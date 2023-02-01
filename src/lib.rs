use std::collections::HashMap;

use pyo3::prelude::*;
use rayon::{iter::IterBridge, prelude::*};

type NodeIndex = u32;
type FiltrationTime = f64;
type EdgeMap = HashMap<NodeIndex, HashMap<NodeIndex, FiltrationTime>>;
type TimedTwoPath = ((NodeIndex, NodeIndex, NodeIndex), FiltrationTime);

#[derive(Debug)]
enum TwoPathType {
    DoubleEdge(NodeIndex, NodeIndex),          // (i,j) where i → j → i
    Triangle(NodeIndex, NodeIndex, NodeIndex), // (i, j, k) where i → j → k, i → k
    Bridge((NodeIndex, NodeIndex), NodeIndex), // ((i, k), j) where i → j → k, i ↛ k
    LongSquare(NodeIndex, (NodeIndex, NodeIndex), NodeIndex), // (i, (j, l), k) i → j → k, i → l → k, i ↛ k
}

// TODO: Implement these columns as a grpphati.Column
#[pyclass]
struct GrpphatiRsColumn {
    two_path: TwoPathType,
    entrance_time: FiltrationTime,
}

#[pymethods]
impl GrpphatiRsColumn {
    fn dimension(&self) -> usize {
        2
    }
}

// impl IntoPy<PyObject> for GrpphatiRsColumn {
//     fn into_py(self, py: Python<'_>) -> PyObject {
//         let columns = py.import("grpphati.columns").unwrap();
//         match self.two_path {
//             TwoPathType::Bridge(_, _) => panic!("Cannot conver bridge to GrPPHATI Column"),
//             TwoPathType::DoubleEdge(i, j) => {
//                 let col_cls = columns.getattr("DoubleEdgeCol").unwrap();
//                 let args = ((i, j), self.entrance_time);
//                 col_cls.call1(args).unwrap().into_py(py)
//             }
//             TwoPathType::Triangle(i, j, k) => {
//                 let col_cls = columns.getattr("DirectedTriangleCol").unwrap();
//                 let args = ((i, j, k), self.entrance_time);
//                 col_cls.call1(args).unwrap().into_py(py)
//             }
//             TwoPathType::LongSquare(i, midpoints, j) => {
//                 let col_cls = columns.getattr("LongSquareCol").unwrap();
//                 let args = (i, midpoints, j, self.entrance_time);
//                 col_cls.call1(args).unwrap().into_py(py)
//             }
//         }
//     }
// }

#[derive(Default)]
struct TwoPathFold {
    // Columns that are ready to be put in the basis
    cols: Vec<GrpphatiRsColumn>,
    // Bridges indexed by their endpoints, together with entrance times
    bridges: HashMap<(NodeIndex, NodeIndex), Vec<(NodeIndex, FiltrationTime)>>,
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn get_two_cells(edge_map: EdgeMap) -> Vec<GrpphatiRsColumn> {
    let two_path_iter = enumerate_two_paths(&edge_map);
    let mut two_path_fold = split_off_bridges(&edge_map, two_path_iter);
    // Add columns arising from bridges
    let bridge_cols = two_path_fold
        .bridges
        .into_iter()
        .par_bridge()
        .flat_map(|(endpoints, bridges)| build_bridge_columns(&edge_map, endpoints, bridges));
    two_path_fold.cols.par_extend(bridge_cols);
    two_path_fold.cols
}

fn enumerate_two_paths<'a>(
    edge_map: &'a EdgeMap,
) -> IterBridge<impl Iterator<Item = TimedTwoPath> + Send + 'a> {
    edge_map
        .iter()
        .flat_map(move |(&source, dists_from_source)| {
            dists_from_source
                .iter()
                .flat_map(move |(midpoint, first_hop)| {
                    edge_map
                        .get(midpoint)
                        .unwrap()
                        .iter()
                        .map(move |(&endpoint, &second_hop)| {
                            ((source, *midpoint, endpoint), first_hop.max(second_hop))
                        })
                })
        })
        .par_bridge()
}

fn split_off_bridges(
    edge_map: &EdgeMap,
    two_paths: IterBridge<impl Iterator<Item = TimedTwoPath> + Send>,
) -> TwoPathFold {
    // Split off two paths that automatically lead to columns
    let typed_paths = two_paths.map(|(two_path, path_time)| {
        if two_path.0 == two_path.2 {
            GrpphatiRsColumn {
                two_path: TwoPathType::DoubleEdge(two_path.0, two_path.1),
                entrance_time: path_time,
            }
        } else if edge_time(edge_map, (&two_path.0, &two_path.2)) <= path_time {
            GrpphatiRsColumn {
                two_path: TwoPathType::Triangle(two_path.0, two_path.1, two_path.2),
                entrance_time: path_time,
            }
        } else {
            GrpphatiRsColumn {
                two_path: TwoPathType::Bridge((two_path.0, two_path.2), two_path.1),
                entrance_time: path_time,
            }
        }
    });
    // In parallel build up the bridges hashmap and cols vector
    // The paths get split across threads and folded in each thread
    let folded = typed_paths.fold(
        || TwoPathFold::default(),
        |mut accum: TwoPathFold, timed_path: GrpphatiRsColumn| {
            match timed_path.two_path {
                TwoPathType::Bridge(endpoints, j) => {
                    if let Some(bridges) = accum.bridges.get_mut(&endpoints) {
                        bridges.push((j, timed_path.entrance_time));
                    } else {
                        accum
                            .bridges
                            .insert(endpoints, vec![(j, timed_path.entrance_time)]);
                    };
                }
                other => accum.cols.push(GrpphatiRsColumn {
                    two_path: other,
                    entrance_time: timed_path.entrance_time,
                }),
            };
            accum
        },
    );
    // Do a final reduce to join the folds made be each thread
    let reduced = folded.reduce(
        || TwoPathFold::default(),
        |mut accum: TwoPathFold, next_fold: TwoPathFold| {
            accum.cols.extend(next_fold.cols.into_iter());
            for (endpoints, fold_bridges) in next_fold.bridges.into_iter() {
                if let Some(accum_bridges) = accum.bridges.get_mut(&endpoints) {
                    accum_bridges.extend(fold_bridges);
                } else {
                    accum.bridges.insert(endpoints, fold_bridges);
                }
            }
            accum
        },
    );
    reduced
}

fn build_bridge_columns(
    edge_map: &EdgeMap,
    endpoints: (NodeIndex, NodeIndex),
    mut bridges: Vec<(NodeIndex, FiltrationTime)>,
) -> Vec<GrpphatiRsColumn> {
    // Sort bridges by filtration time
    bridges.sort_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap());
    let mut bridge_iter = bridges.into_iter();
    let first_bridge = bridge_iter.next().expect("Found empty bridge vector");
    // First add the collapsing directed triangle
    let collapse_time = edge_time(edge_map, (&endpoints.0, &endpoints.1));
    let mut columns = vec![GrpphatiRsColumn {
        two_path: TwoPathType::Triangle(endpoints.0, first_bridge.0, endpoints.1),
        entrance_time: collapse_time,
    }];
    for (bridge, time) in bridge_iter {
        columns.push(GrpphatiRsColumn {
            two_path: TwoPathType::LongSquare(endpoints.0, (first_bridge.0, bridge), endpoints.1),
            entrance_time: time,
        })
    }
    columns
}

fn edge_time(edge_map: &EdgeMap, edge: (&NodeIndex, &NodeIndex)) -> FiltrationTime {
    *edge_map
        .get(edge.0)
        .and_then(|dist_map| dist_map.get(edge.1))
        .unwrap_or(&FiltrationTime::INFINITY)
}

/// A Python module implemented in Rust.
#[pymodule]
fn grpphati_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_two_cells, m)?)?;
    Ok(())
}
