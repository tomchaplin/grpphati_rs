use pyo3::prelude::*;

mod columns;
mod compute_mapping;
mod homology;
mod sparsifiers;

use columns::GrpphatiRsColumn;
use compute_mapping::compute_map_py;
use homology::get_rph_two_cells;
use sparsifiers::{RustIteratorSparsifier, RustListSparsifier, RustParallelListSparsifier};

type NodeIndex = u32;
type FiltrationTime = f64;

// TODO: Provide python method which orchestrates entire pipeline
//    build_columns -> build_map        |--> run phimaker with cylinder
//                 |-> sparsify columns |/

/// A Python module implemented in Rust.
#[pymodule]
fn grpphati_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_rph_two_cells, m)?)?;
    m.add_function(wrap_pyfunction!(compute_map_py, m)?)?;
    m.add_class::<GrpphatiRsColumn>()?;
    m.add_class::<RustListSparsifier>()?;
    m.add_class::<RustParallelListSparsifier>()?;
    m.add_class::<RustIteratorSparsifier>()?;
    Ok(())
}
