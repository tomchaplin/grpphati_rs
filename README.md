# GrPPHATI_rs (WIP)

Rust implementation of regular path homology for GrPPHATI.

## Known issues

- Currently slower than GrPPHATI default implementation.
This is not due to PyO3 overhead but instead the cells are not produced in the best order for the persistence calculation.
