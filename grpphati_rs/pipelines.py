from grpphati_rs import RustRegularPathHomology, RustPreferredSparsifier
from grpphati_rs.grpphati_rs import sparsify_and_decompose
from grpphati.filtrations import ShortestPathFiltration
from grpphati.optimisations import component_appendage_empty, all_optimisations
from grpphati.backends import LoPHATBackend, Backend
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
from grpphati.truncations import cone_time
from grpphati.results import Result

from lophat import LoPhatOptions, compute_pairings


class GrpphatiRsBackend(Backend):
    def __init__(self):
        pass

    def compute_ph(self, cols) -> Result:
        cols.sort(key=lambda col: (col.dimension(), col.get_entrance_time()))
        diagram = sparsify_and_decompose(cols)
        result = Result.empty()
        result.add_paired(diagram.paired, cols, reps=None)
        result.add_unpaired_raw(diagram.unpaired, cols, reps=None)
        return result


GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=GrpphatiRsBackend(),
    optimisation_strat=component_appendage_empty,
    truncation_strat=cone_time,
)

GrPPH_par_wedge_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=GrpphatiRsBackend(),
    optimisation_strat=all_optimisations,
    truncation_strat=cone_time,
)
