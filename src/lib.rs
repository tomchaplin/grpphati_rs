use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use pyo3::prelude::*;
use rayon::{iter::IterBridge, prelude::*};

type NodeIndex = u32;
type FiltrationTime = f64;
type EdgeMap = HashMap<NodeIndex, HashMap<NodeIndex, FiltrationTime>>;
type UnstructuredTwoPathWithTime = ((NodeIndex, NodeIndex, NodeIndex), FiltrationTime);

#[derive(Debug)]
enum TwoPathType {
    DoubleEdge(NodeIndex, NodeIndex),          // (i,j) where i → j → i
    Triangle(NodeIndex, NodeIndex, NodeIndex), // (i, j, k) where i → j → k, i → k
    Bridge((NodeIndex, NodeIndex), NodeIndex), // ((i, k), j) where i → j → k, i ↛ k
}

struct TwoPathWithTime {
    two_path: TwoPathType,
    entrance_time: FiltrationTime,
}

// TODO: This would be better implemented as a trait, does this play well with PyO3?
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum ColumnType {
    DoubleEdge(NodeIndex, NodeIndex),          // (i,j) where i → j → i
    Triangle(NodeIndex, NodeIndex, NodeIndex), // (i, j, k) where i → j → k, i → k
    LongSquare(NodeIndex, (NodeIndex, NodeIndex), NodeIndex), // (i, (j, l), k) i → j → k, i → l → k, i ↛ k
    Edge(NodeIndex, NodeIndex),
    Node(NodeIndex),
}

impl TryFrom<TwoPathType> for ColumnType {
    type Error = &'static str;
    fn try_from(value: TwoPathType) -> Result<Self, Self::Error> {
        match value {
            TwoPathType::DoubleEdge(i, j) => Ok(ColumnType::DoubleEdge(i, j)),
            TwoPathType::Triangle(i, j, k) => Ok(ColumnType::Triangle(i, j, k)),
            TwoPathType::Bridge(_, _) => Err("Cannot convert Bridge path into Column"),
        }
    }
}

// TODO: Implement these columns as a grpphati.Column
#[pyclass]
#[derive(Clone)]
struct GrpphatiRsColumn {
    col_type: ColumnType,
    entrance_time: Option<FiltrationTime>,
}

#[pymethods]
impl GrpphatiRsColumn {
    #[new]
    fn new(
        col_type_str: &str,
        data: Vec<NodeIndex>,
        entrance_time: Option<FiltrationTime>,
    ) -> Self {
        // TODO: Deal with incorrectly supplied data
        let col_type = match col_type_str {
            "DoubleEdge" => ColumnType::DoubleEdge(data[0], data[1]),
            "Triangle" => ColumnType::Triangle(data[0], data[1], data[2]),
            "LongSquare" => ColumnType::LongSquare(data[0], (data[1], data[2]), data[3]),
            "Edge" => ColumnType::Edge(data[0], data[1]),
            "Node" => ColumnType::Node(data[0]),
            _ => panic!(),
        };
        Self {
            col_type,
            entrance_time,
        }
    }

    fn dimension(&self) -> usize {
        match self.col_type {
            ColumnType::DoubleEdge(_, _) => 2,
            ColumnType::Triangle(_, _, _) => 2,
            ColumnType::LongSquare(_, _, _) => 2,
            ColumnType::Edge(_, _) => 1,
            ColumnType::Node(_) => 0,
        }
    }

    fn get_entrance_time(&self) -> FiltrationTime {
        self.entrance_time
            .expect("Column does not have an entrance time")
    }

    // TODO: Make this more informative
    fn __repr__(&self) -> String {
        match self.col_type {
            ColumnType::DoubleEdge(i, j) => format!("DoubleEdge({i},{j})"),
            ColumnType::Triangle(i, j, k) => format!("Triangle({i},{j},{k})"),
            ColumnType::LongSquare(i, (e0, e1), k) => format!("LongSquare({i},{e0},{e1},{k})"),
            ColumnType::Edge(i, j) => format!("Edge({i},{j})"),
            ColumnType::Node(i) => format!("Node({i})"),
        }
    }

    fn __eq__(&self, other: &PyAny) -> bool {
        other
            .extract()
            .and_then(|other_col: GrpphatiRsColumn| Ok(other_col.col_type == self.col_type))
            .unwrap_or(false)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.col_type.hash(&mut hasher);
        let output = hasher.finish();
        output
    }

    // Defining this magic method becuase __eq__ is not supported by PyO3
    fn __richcmp__(&self, other: &PyAny, cmp_op: pyo3::basic::CompareOp) -> bool {
        match cmp_op {
            pyo3::pyclass::CompareOp::Lt => todo!(),
            pyo3::pyclass::CompareOp::Le => todo!(),
            pyo3::pyclass::CompareOp::Eq => self.__eq__(other),
            pyo3::pyclass::CompareOp::Ne => todo!(),
            pyo3::pyclass::CompareOp::Gt => todo!(),
            pyo3::pyclass::CompareOp::Ge => todo!(),
        }
    }

    fn boundary(&self) -> Vec<GrpphatiRsColumn> {
        match self.col_type {
            ColumnType::DoubleEdge(i, j) => vec![
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(i, j),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(j, i),
                    entrance_time: None,
                },
            ],
            ColumnType::Triangle(i, j, k) => vec![
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(i, j),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(j, k),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(i, k),
                    entrance_time: None,
                },
            ],
            ColumnType::LongSquare(start, midpoints, end) => vec![
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(start, midpoints.0),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(start, midpoints.1),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(midpoints.0, end),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Edge(midpoints.1, end),
                    entrance_time: None,
                },
            ],
            ColumnType::Edge(i, j) => vec![
                GrpphatiRsColumn {
                    col_type: ColumnType::Node(i),
                    entrance_time: None,
                },
                GrpphatiRsColumn {
                    col_type: ColumnType::Node(j),
                    entrance_time: None,
                },
            ],
            ColumnType::Node(_) => vec![],
        }
    }

    fn to_grpphati_column(&self) -> PyObject {
        //TODO: Do I need to obtain GIL?
        Python::with_gil(|py| {
            let columns = py.import("grpphati.columns").unwrap();
            match self.col_type {
                ColumnType::DoubleEdge(i, j) => {
                    let col_cls = columns.getattr("DoubleEdgeCol").unwrap();
                    let args = ((i, j), self.entrance_time);
                    col_cls.call1(args).unwrap().into_py(py)
                }
                ColumnType::Triangle(i, j, k) => {
                    let col_cls = columns.getattr("DirectedTriangleCol").unwrap();
                    let args = ((i, j, k), self.entrance_time);
                    col_cls.call1(args).unwrap().into_py(py)
                }
                ColumnType::LongSquare(i, midpoints, j) => {
                    let col_cls = columns.getattr("LongSquareCol").unwrap();
                    let args = (i, midpoints, j, self.entrance_time);
                    col_cls.call1(args).unwrap().into_py(py)
                }
                ColumnType::Edge(i, j) => {
                    let col_cls = columns.getattr("EdgeCol").unwrap();
                    let args = ((i, j), self.entrance_time);
                    col_cls.call1(args).unwrap().into_py(py)
                }
                ColumnType::Node(i) => {
                    let col_cls = columns.getattr("NodeCol").unwrap();
                    let args = (i, self.entrance_time);
                    col_cls.call1(args).unwrap().into_py(py)
                }
            }
        })
    }
}

#[pyclass]
struct RustListSparsifier {}

#[pymethods]
impl RustListSparsifier {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn __call__(&mut self, cols: Vec<GrpphatiRsColumn>) -> Vec<(usize, Vec<usize>)> {
        let mut sparse_cols = vec![];
        let mut col2idx_map: HashMap<ColumnType, usize> = HashMap::new();
        for col in cols {
            let bdry = col.boundary();
            let mut sparse_bdry = vec![];
            for row in bdry {
                let idx = col2idx_map.get(&row.col_type).unwrap();
                sparse_bdry.push(*idx);
            }
            sparse_bdry.sort();
            let insertion_idx = sparse_cols.len();
            sparse_cols.push((col.dimension(), sparse_bdry));
            col2idx_map.insert(col.col_type, insertion_idx);
        }
        sparse_cols
    }
}

#[derive(Default)]
struct TwoPathFold {
    // Columns that are ready to be put in the basis
    cols: Vec<GrpphatiRsColumn>,
    // Bridges indexed by their endpoints, together with entrance times
    bridges: HashMap<(NodeIndex, NodeIndex), Vec<(NodeIndex, FiltrationTime)>>,
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn get_rph_two_cells(edge_map: EdgeMap) -> Vec<GrpphatiRsColumn> {
    let two_path_iter = enumerate_two_paths(&edge_map);
    let mut two_path_fold = split_off_bridges(&edge_map, two_path_iter);
    // Add columns arising from bridges
    let bridge_cols = two_path_fold
        .bridges
        .into_iter()
        .par_bridge()
        .flat_map(|(endpoints, bridges)| build_bridge_columns(&edge_map, endpoints, bridges));
    two_path_fold.cols.par_extend(bridge_cols);
    println!("Computed cells");
    two_path_fold.cols
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
        col_type: ColumnType::Triangle(endpoints.0, first_bridge.0, endpoints.1),
        entrance_time: Some(collapse_time),
    }];
    for (bridge, time) in bridge_iter {
        columns.push(GrpphatiRsColumn {
            col_type: ColumnType::LongSquare(endpoints.0, (first_bridge.0, bridge), endpoints.1),
            entrance_time: Some(time),
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
    m.add_function(wrap_pyfunction!(get_rph_two_cells, m)?)?;
    m.add_class::<GrpphatiRsColumn>()?;
    m.add_class::<RustListSparsifier>()?;
    Ok(())
}
