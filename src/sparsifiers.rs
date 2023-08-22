use rayon::prelude::*;
use std::{collections::HashMap, sync::Mutex};

use dashmap::DashMap;
use pyo3::prelude::*;

use crate::columns::{ColumnType, GrpphatiRsColumn};

#[pyclass]
pub struct RustListSparsifier {}

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

#[pyclass]
pub struct RustParallelListSparsifier {
    max_dim: usize,
}

impl RustParallelListSparsifier {
    pub fn sparsify(
        &mut self,
        cols: &Vec<GrpphatiRsColumn>,
    ) -> impl Iterator<Item = (usize, Vec<usize>)> {
        let mut sparse_cols: Vec<Mutex<(usize, Vec<usize>)>> = Vec::with_capacity(cols.len());
        // Build up output
        for _ in 0..cols.len() {
            sparse_cols.push(Mutex::new((0, vec![])));
        }
        let col2idx_map: DashMap<ColumnType, usize> = DashMap::new();
        for working_dim in 0..=self.max_dim {
            // Build boundaries
            cols.iter()
                .enumerate()
                .par_bridge()
                .filter(|(_col_idx, col)| col.dimension() == working_dim)
                .for_each(|(col_idx, col)| {
                    let bdry = col.boundary();
                    let mut sparse_bdry = vec![];
                    for row in bdry {
                        let idx = col2idx_map.get(&row.col_type).unwrap();
                        sparse_bdry.push(*idx);
                    }
                    sparse_bdry.sort();
                    let dimension = col.dimension();
                    *sparse_cols[col_idx].lock().unwrap() = (dimension, sparse_bdry);
                });
            // Insert into col2idx_map
            if working_dim == self.max_dim {
                continue;
            }
            cols.iter()
                .enumerate()
                .par_bridge()
                .filter(|(_col_idx, col)| col.dimension() == working_dim)
                .for_each(|(col_idx, col)| {
                    col2idx_map.insert(col.col_type, col_idx);
                })
        }
        sparse_cols
            .into_iter()
            .map(|outer| outer.into_inner().unwrap())
    }
}

#[pymethods]
impl RustParallelListSparsifier {
    #[new]
    pub fn new(max_dim: usize) -> Self {
        Self { max_dim }
    }

    fn __call__(&mut self, cols: Vec<GrpphatiRsColumn>) -> Vec<(usize, Vec<usize>)> {
        println!("Sparsified");
        self.sparsify(&cols).collect()
    }
}

#[pyclass]
pub struct RustIteratorSparsifier {
    col2idx_map: HashMap<ColumnType, usize>,
    current_idx: usize,
    cols: std::vec::IntoIter<GrpphatiRsColumn>,
}

#[pymethods]
impl RustIteratorSparsifier {
    #[new]
    fn new(cols: Vec<GrpphatiRsColumn>) -> Self {
        Self {
            col2idx_map: HashMap::new(),
            current_idx: 0,
            cols: cols.into_iter(),
        }
    }
    fn get_next(&mut self) -> Option<(usize, Vec<usize>)> {
        let col = self.cols.next()?;
        let bdry = col.boundary();
        let mut sparse_bdry = vec![];
        for row in bdry {
            let idx = self.col2idx_map.get(&row.col_type).unwrap();
            sparse_bdry.push(*idx);
        }
        sparse_bdry.sort();
        self.col2idx_map.insert(col.col_type, self.current_idx);
        self.current_idx += 1;
        Some((col.dimension(), sparse_bdry))
    }
}
