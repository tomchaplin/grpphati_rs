# GrPPHATI_rs (WIP)

Rust implementation of various [GrPPHATI](https://github.com/tomchaplin/grpphati) classes, including:
* `RustRegularPathHomology` - an implementation of regular path homology producing a basis in parallel, using custom column types.
* `RustGeneratorSparsifier` - a lazy sparsifier written in Rust, meant to work with columns produced by `RustRegularPathHomology`.
* `RustPreferredSparsifier` - a non-lazy sparsifier written in Rust, meant to work with columns produced by `RustRegularPathHomology`.
Sparsifies columns of each dimension in parallel.

## Usage

Note `RustRegularPathHomology` does not produce columns in the same order as `RegularPathHomology`.
This leads to considerable slow down when using `PHATBackend`.
As such, the recommended usage is with `LoPHATBackend`, available in `grpphati`.
Eirene has not been tested so far.
It is also recommended to use the provided parallel sparsifier.
Good pipelines are provided in `grpphati_rs.GrPPH_rs` and `grpphai_rs.GrPPH_par_wedge_rs`.

For example usage, please consult `examples/disjoint.py` in the repository.

## Known issues

- Graphs used with `RustRegularPathHomology` must be integer indexed.
