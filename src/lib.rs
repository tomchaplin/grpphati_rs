use lophat::{
    algorithms::{LockFreeAlgorithm, RVDecomposition},
    columns::VecColumn,
    options::LoPhatOptions,
    utils::{anti_transpose, PersistenceDiagram},
};
use pyo3::prelude::*;

mod columns;
mod compute_mapping;
mod homology;
mod sparsifiers;

use columns::GrpphatiRsColumn;
use compute_mapping::compute_map_py;
use homology::{get_dflag_two_cells, get_rph_two_cells};
use sparsifiers::{RustIteratorSparsifier, RustListSparsifier, RustParallelListSparsifier};

type NodeIndex = u32;
type FiltrationTime = f64;

#[pyfunction]
pub fn sparsify_and_decompose(cols: Vec<GrpphatiRsColumn>) -> PersistenceDiagram {
    let mut sparsifier = RustParallelListSparsifier::new(2);
    let sparse_cols: Vec<_> = sparsifier.sparsify(&cols).map(VecColumn::from).collect();
    println!("Sparsified");
    let width = sparse_cols.len();
    let at = anti_transpose(&sparse_cols);
    println!("Anti-transposed");
    let mut options = LoPhatOptions::default();
    options.min_chunk_len = 10000;
    let decomp = LockFreeAlgorithm::decompose(at.into_iter(), Some(options));
    println!("Decomposed");
    let diagram = decomp.diagram();
    println!("Got diagram");
    let diagram = diagram.anti_transpose(width);
    diagram
}

// TODO: Provide python method which orchestrates entire pipeline
//    build_columns -> build_map        |--> run phimaker with cylinder
//                 |-> sparsify columns |/

/// A Python module implemented in Rust.
#[pymodule]
fn grpphati_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_rph_two_cells, m)?)?;
    m.add_function(wrap_pyfunction!(get_dflag_two_cells, m)?)?;
    m.add_function(wrap_pyfunction!(compute_map_py, m)?)?;
    m.add_function(wrap_pyfunction!(sparsify_and_decompose, m)?)?;
    m.add_class::<GrpphatiRsColumn>()?;
    m.add_class::<RustListSparsifier>()?;
    m.add_class::<RustParallelListSparsifier>()?;
    m.add_class::<RustIteratorSparsifier>()?;
    Ok(())
}
