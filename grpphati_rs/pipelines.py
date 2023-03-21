from grpphati_rs import RustRegularPathHomology, RustPreferredSparsifier
from grpphati.filtrations import ShortestPathFiltration
from grpphati.optimisations import component_appendage_empty, all_optimisations
from grpphati.backends import LoPHATBackend
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline

GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=LoPHATBackend(sparsifier=RustPreferredSparsifier(2)),
    optimisation_strat=component_appendage_empty,
)

GrPPH_par_wedge_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=LoPHATBackend(sparsifier=RustPreferredSparsifier(2)),
    optimisation_strat=all_optimisations,
)
