# GrPPHATI_rs (WIP)

Rust implementation of various [GrPPHATI](https://github.com/tomchaplin/grpphati) classes, including:
* `RustRegularPathHomology` - an implementation of regular path homology producing a basis in parallel, using custom column types.
* `RustGeneratorSparsifier` - a lazy sparsifier written in Rust, meant to work with columns produced by `RustRegularPathHomology`.

## Usage

Note `RustRegularPathHomology` does not produce columns in the same order as `RegularPathHomology`.
This leads to considerable slow down when using `PHATBackend`.
As such, the recommended usage is with `PersuitBackend`, available in `grpphati[persuit]`.
Eirene has not been tested so far.
It is also recommended to use the provided sparsifier.
A good pipeline is

```python
from grpphati.filtrations import ShortestPathFiltration
from grpphati_rs import RustRegularPathHomology, RustGeneratorSparsifier
from grpphati.backends import PersuitBackend
from grpphati.optimisations import all_optimisations

GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=PersuitBackend(sparsifier=RustGeneratorSparsifier(return_dimension=False)),
    optimisation_strat=all_optimisations,
)
```


## Known issues

- Currently slower than GrPPHATI default implementation when using PHAT.
This is not due to PyO3 overhead but instead the cells are not produced in the best order for the persistence calculation.
