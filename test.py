import time
import random
import networkx as nx
import numpy as np
from math import log
from grpphati_rs import RustRegularPathHomology, RustGeneratorSparsifier
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


# log_deltas = []
#
# N = 150
# G = nx.DiGraph()
# for i in range(N):
#     G.add_node(i)
# for i in range(N):
#     G.add_edge(i, (i + 1) % N, weight=random.random())
#
best_pipeline = make_grounded_pipeline(
    ShortestPathFiltration,
    RustRegularPathHomology,
    backend=PersuitBackend(sparsifier=RustGeneratorSparsifier(return_dimension=False)),
    optimisation_strat=None,
)

old_pipeline = make_grounded_pipeline(
    ShortestPathFiltration,
    RegularPathHomology,
    backend=PersuitBackend(sparsifier=GeneratorSparsifier(return_dimension=False)),
    optimisation_strat=None,
)
from data.paul_analysis import assemble_dash_data

tic0 = time.time()
d2 = assemble_dash_data(data_root="./data/all", pipeline=old_pipeline)
print("Done")
tic1 = time.time()
d0 = assemble_dash_data(data_root="./data/all", pipeline=best_pipeline)
print("Done")
tic2 = time.time()
print(tic1 - tic0)
print(tic2 - tic1)

# cells1 = RustRegularPathHomology.get_cells([0, 1, 2], ShortestPathFiltration(G))
# cells2 = RegularPathHomology.get_cells([0, 1, 2], ShortestPathFiltration(G))
# cells1_py = [cell.to_grpphati_column() for cell in cells1]
# cells1_py.sort(key=lambda c: (c.get_entrance_time(), c.dimension()))
# cells2.sort(key=lambda c: (c.get_entrance_time(), c.dimension()))
# print(set(cells1_py) == set(cells2))

## tic1 = time.time()
##
## print("Start")
## res1 = pipeline(G)
## print("End")
##
## tic2 = time.time()
##
## print("Start")
## res2 = old_pipeline(G)
## print("End")
##
## tic3 = time.time()

## print("Start")
## res3 = third_pipeline(G)
## print("End")

##  tic4 = time.time()
##
##  print(tic2 - tic1)
##  print(tic3 - tic2)
##  print(tic4 - tic3)
