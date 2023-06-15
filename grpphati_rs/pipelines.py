from grpphati_rs import RustRegularPathHomology, RustPreferredSparsifier
from grpphati.filtrations import ShortestPathFiltration
from grpphati.optimisations import component_appendage_empty, all_optimisations
from grpphati.backends import LoPHATBackend
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
from grpphati.truncations import cone_time

# We don't add reps because GrpphatiRsColumn is not pickleable

GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=LoPHATBackend(sparsifier=RustPreferredSparsifier(2), with_reps=False),
    optimisation_strat=component_appendage_empty,
    truncation_strat=cone_time,
)

GrPPH_par_wedge_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=LoPHATBackend(sparsifier=RustPreferredSparsifier(2), with_reps=False),
    optimisation_strat=all_optimisations,
    truncation_strat=cone_time,
)
