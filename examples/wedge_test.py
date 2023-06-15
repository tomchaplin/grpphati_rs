import time
import random
import networkx as nx
import numpy as np
from math import log

# from grpphati.homologies import RegularPathHomology
# from grpphati.backends import PHATBackend, PersuitBackend
# from grpphati.filtrations import ShortestPathFiltration
# from grpphati.homologies import Homology
# from grpphati.optimisations import component_appendage_empty
# from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
# from grpphati.sparsifiers import ListSparsifier, GeneratorSparsifier
from joblib import Parallel, delayed

from grpphati_rs import RustRegularPathHomology, RustPreferredSparsifier
from grpphati.filtrations import ShortestPathFiltration
from grpphati.optimisations import component_appendage_empty, all_optimisations
from grpphati.backends import LoPHATBackend
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
from grpphati.truncations import cone_time

GrPPH_rs = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=LoPHATBackend(sparsifier=RustPreferredSparsifier(2), with_reps=False),
    optimisation_strat=component_appendage_empty,
    truncation_strat=cone_time,
)


def do_job():
    G = nx.DiGraph()
    N = 50
    for j in range(100):
        G.add_edges_from([((i, j), ((i + 1) % N, j)) for i in range(N)])
    G = nx.convert_node_labels_to_integers(G)
    res = GrPPH_rs(G)
    return res


res = do_job()
print(res.barcode)
