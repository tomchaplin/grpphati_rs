use pyo3::prelude::*;

mod columns;
mod homology;
mod sparsifiers;

use columns::GrpphatiRsColumn;
use sparsifiers::{RustIteratorSparsifier, RustListSparsifier, RustParallelListSparsifier};

use crate::homology::get_rph_two_cells;

type NodeIndex = u32;
type FiltrationTime = f64;

/// A Python module implemented in Rust.
#[pymodule]
fn grpphati_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_rph_two_cells, m)?)?;
    m.add_class::<GrpphatiRsColumn>()?;
    m.add_class::<RustListSparsifier>()?;
    m.add_class::<RustParallelListSparsifier>()?;
    m.add_class::<RustIteratorSparsifier>()?;
    Ok(())
}
