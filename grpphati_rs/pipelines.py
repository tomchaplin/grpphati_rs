from grpphati_rs import RustRegularPathHomology, RustGeneratorSparsifier
from grpphati.filtrations import ShortestPathFiltration
from grpphati.optimisations import component_appendage_empty, all_optimisations
from grpphati.backends import PersuitBackend
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline

GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=PersuitBackend(
        in_parallel=False, sparsifier=RustGeneratorSparsifier(return_dimension=False)
    ),
    optimisation_strat=component_appendage_empty,
)

GrPPH_par_wedge_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=PersuitBackend(
        in_parallel=False, sparsifier=RustGeneratorSparsifier(return_dimension=False)
    ),
    optimisation_strat=all_optimisations,
)
