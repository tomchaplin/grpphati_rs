use pyo3::prelude::*;

use rayon::{iter::IterBridge, prelude::*};

use std::{cmp::Ordering, collections::HashMap};

use crate::{
    columns::{ColumnType, GrpphatiRsColumn},
    FiltrationTime, NodeIndex,
};

type EdgeMap = HashMap<NodeIndex, HashMap<NodeIndex, FiltrationTime>>;
type UnstructuredTwoPathWithTime = ((NodeIndex, NodeIndex, NodeIndex), FiltrationTime);

#[derive(Debug)]
pub enum TwoPathType {
    DoubleEdge(NodeIndex, NodeIndex),          // (i,j) where i → j → i
    Triangle(NodeIndex, NodeIndex, NodeIndex), // (i, j, k) where i → j → k, i → k
    Bridge((NodeIndex, NodeIndex), NodeIndex), // ((i, k), j) where i → j → k, i ↛ k
}

struct TwoPathWithTime {
    two_path: TwoPathType,
    entrance_time: FiltrationTime,
}

#[derive(Default)]
struct TwoPathFold {
    // Columns that are ready to be put in the basis
    cols: Vec<GrpphatiRsColumn>,
    // Bridges indexed by their endpoints, together with entrance times
    bridges: HashMap<(NodeIndex, NodeIndex), Vec<(NodeIndex, FiltrationTime)>>,
}

fn compare_columns(col_a: &GrpphatiRsColumn, col_b: &GrpphatiRsColumn) -> Ordering {
    let t_a = col_a
        .entrance_time
        .expect("Produced columns should have an entrance time");
    let t_b = col_b
        .entrance_time
        .expect("Produced columns should have an entrance time");
    t_a.partial_cmp(&t_b)
        .expect("Neither filtration time should be NaN")
}

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn get_rph_two_cells(edge_map: EdgeMap) -> Vec<GrpphatiRsColumn> {
    let two_path_iter = enumerate_two_paths(&edge_map);
    let mut two_path_fold = split_off_bridges(&edge_map, two_path_iter);
    // Add columns arising from bridges
    let sorted_bridges = two_path_fold
        .bridges
        .into_iter()
        .par_bridge()
        .map(|(endpoints, bridges)| (endpoints, sort_bridges(bridges)));
    // TODO: Make this neater and in paralell?
    let bridge_cols = sorted_bridges
        .map(|(endpoints, bridges)| build_bridge_columns(&edge_map, endpoints, bridges));
    let (long_square_cols, triangle_cols): (Vec<_>, Vec<_>) = bridge_cols.unzip();
    let long_square_cols: Vec<_> = long_square_cols.into_iter().flatten().collect();
    let triangle_cols: Vec<_> = triangle_cols.into_iter().flatten().collect();
    two_path_fold.cols.extend(triangle_cols);
    two_path_fold.cols.extend(long_square_cols);
    println!("Computed 2-cells");
    two_path_fold.cols.sort_unstable_by(compare_columns);
    println!("Sorted 2-cells");
    two_path_fold.cols
}

#[pyfunction]
pub fn get_dflag_two_cells(edge_map: EdgeMap) -> Vec<GrpphatiRsColumn> {
    let two_path_iter = enumerate_two_paths(&edge_map);
    let mut cols: Vec<_> = two_path_iter
        .filter_map(|(path, path_time)| {
            if path.0 == path.2 {
                return None;
            }
            let ac_time = edge_time(&edge_map, (&path.0, &path.2));
            let entrance_time = path_time.max(ac_time);
            if entrance_time.is_infinite() {
                return None;
            }
            Some(GrpphatiRsColumn {
                col_type: ColumnType::Triangle(path.0, path.1, path.2),
                entrance_time: Some(entrance_time),
            })
        })
        .collect();
    cols.sort_unstable_by(compare_columns);
    cols
}

fn enumerate_two_paths<'a>(
    edge_map: &'a EdgeMap,
) -> IterBridge<impl Iterator<Item = UnstructuredTwoPathWithTime> + Send + 'a> {
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
    two_paths: IterBridge<impl Iterator<Item = UnstructuredTwoPathWithTime> + Send>,
) -> TwoPathFold {
    // Split off two paths that automatically lead to columns
    let typed_paths = two_paths.map(|(two_path, path_time)| {
        if two_path.0 == two_path.2 {
            TwoPathWithTime {
                two_path: TwoPathType::DoubleEdge(two_path.0, two_path.1),
                entrance_time: path_time,
            }
        } else if edge_time(edge_map, (&two_path.0, &two_path.2)) <= path_time {
            TwoPathWithTime {
                two_path: TwoPathType::Triangle(two_path.0, two_path.1, two_path.2),
                entrance_time: path_time,
            }
        } else {
            TwoPathWithTime {
                two_path: TwoPathType::Bridge((two_path.0, two_path.2), two_path.1),
                entrance_time: path_time,
            }
        }
    });
    // In parallel build up the bridges hashmap and cols vector
    // The paths get split across threads and folded in each thread
    let folded = typed_paths.fold(
        || TwoPathFold::default(),
        |mut accum: TwoPathFold, timed_path: TwoPathWithTime| {
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
                    col_type: other
                        .try_into()
                        .expect("Could not convert two path into column"),
                    entrance_time: Some(timed_path.entrance_time),
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

fn sort_bridges(mut bridges: Vec<(NodeIndex, FiltrationTime)>) -> Vec<(NodeIndex, FiltrationTime)> {
    bridges.sort_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap());
    bridges
}

fn build_bridge_columns(
    edge_map: &EdgeMap,
    endpoints: (NodeIndex, NodeIndex),
    bridges: Vec<(NodeIndex, FiltrationTime)>,
) -> (Vec<GrpphatiRsColumn>, Vec<GrpphatiRsColumn>) {
    let mut bridge_iter = bridges.into_iter();
    let first_bridge = bridge_iter.next().expect("Found empty bridge vector");
    let collapse_time = edge_time(edge_map, (&endpoints.0, &endpoints.1));
    let collapsing_col = GrpphatiRsColumn {
        col_type: ColumnType::Triangle(endpoints.0, first_bridge.0, endpoints.1),
        entrance_time: Some(collapse_time),
    };
    let mut ls_columns = vec![];
    for (bridge, time) in bridge_iter {
        ls_columns.push(GrpphatiRsColumn {
            col_type: ColumnType::LongSquare(endpoints.0, (first_bridge.0, bridge), endpoints.1),
            entrance_time: Some(time),
        })
    }
    (ls_columns, vec![collapsing_col])
}

fn edge_time(edge_map: &EdgeMap, edge: (&NodeIndex, &NodeIndex)) -> FiltrationTime {
    *edge_map
        .get(edge.0)
        .and_then(|dist_map| dist_map.get(edge.1))
        .unwrap_or(&FiltrationTime::INFINITY)
}
