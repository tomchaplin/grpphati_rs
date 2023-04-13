use crate::homology::TwoPathType;
use crate::{FiltrationTime, NodeIndex};
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// TODO: This would be better implemented as a trait, does this play well with PyO3?
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ColumnType {
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

#[pyclass]
#[derive(Clone, Debug)]
pub struct GrpphatiRsColumn {
    pub col_type: ColumnType,
    pub entrance_time: Option<FiltrationTime>,
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

    pub fn dimension(&self) -> usize {
        match self.col_type {
            ColumnType::DoubleEdge(_, _) => 2,
            ColumnType::Triangle(_, _, _) => 2,
            ColumnType::LongSquare(_, _, _) => 2,
            ColumnType::Edge(_, _) => 1,
            ColumnType::Node(_) => 0,
        }
    }

    pub fn get_entrance_time(&self) -> FiltrationTime {
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

    pub fn boundary(&self) -> Vec<GrpphatiRsColumn> {
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

    pub fn to_grpphati_column(&self) -> PyObject {
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
