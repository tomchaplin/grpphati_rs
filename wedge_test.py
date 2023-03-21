import time
import random
import networkx as nx
import numpy as np
from math import log
from grpphati_rs import (
    RustRegularPathHomology,
    RustGeneratorSparsifier,
    GrPPH_par_wedge_rs,
    GrPPH_rs,
)
from grpphati_rs.grpphati_rs import RustListSparsifier
from grpphati.homologies import RegularPathHomology
from grpphati.backends import PHATBackend, PersuitBackend
from grpphati.filtrations import ShortestPathFiltration
from grpphati.homologies import Homology
from grpphati.optimisations import component_appendage_empty
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
from grpphati.sparsifiers import ListSparsifier, GeneratorSparsifier
from joblib import Parallel, delayed


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
