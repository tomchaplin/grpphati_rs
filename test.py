import time
import random
import networkx as nx
import numpy as np
from math import log
from grpphati_rs import RustRegularPathHomology
from grpphati_rs.grpphati_rs import RustListSparsifier
from grpphati.homologies import RegularPathHomology
from grpphati.backends import PHATBackend, PersuitBackend
from grpphati.filtrations import ShortestPathFiltration
from grpphati.homologies import Homology
from grpphati.optimisations import component_appendage_empty
from grpphati.pipelines.grounded import GrPPH, make_grounded_pipeline
from grpphati.sparsifiers import ListSparsifier, GeneratorSparsifier
import phat

def _non_trivial_dict(sp_iter):
    return {
        source: {
            target: distance
            for target, distance in distances.items()
            if target != source and np.isfinite(distance)
        }
        for source, distances in sp_iter
    }

log_deltas = []

N = 100
G = nx.DiGraph()
for i in range(N):
    G.add_node(i)
for i in range(N):
    G.add_edge(i, (i+1)%N, weight=random.random())

pipeline = make_grounded_pipeline(
        ShortestPathFiltration,
        RustRegularPathHomology,
        backend = PHATBackend(
                              sparsifier = ListSparsifier(return_dimension = True)),
        optimisation_strat = None)

old_pipeline = make_grounded_pipeline(
        ShortestPathFiltration,
        RegularPathHomology,
        backend = PHATBackend(
                              sparsifier = ListSparsifier(return_dimension = True)),
        optimisation_strat = None)

cells1 = RustRegularPathHomology.get_cells([0,1,2],ShortestPathFiltration(G))
cells2 = RegularPathHomology.get_cells([0,1,2],ShortestPathFiltration(G))
print(len(cells1))
print(len(cells2))


tic1 = time.time()
print("Start")
res1 = pipeline(G)
print("End")
tic2 = time.time()
print("Start")
res2 = old_pipeline(G)
print("End")
tic3 = time.time()
print(tic2 - tic1)
print(tic3 - tic2)

