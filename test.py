import time
import networkx as nx
import numpy as np
from math import log
from grpphati_rs import get_two_cells
from grpphati.homologies import RegularPathHomology
from grpphati.filtrations import ShortestPathFiltration

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

for N in range(100,401,100):
    G = nx.DiGraph()
    for i in range(N):
        G.add_node(i)
    for i in range(N):
        G.add_edge(i, (i+1)%N, weight=0.3)
    #sp_lengths = _non_trivial_dict(nx.all_pairs_dijkstra_path_length(G))
    filtration = ShortestPathFiltration(G)
    #tic1 = time.time()
    #res1 = RegularPathHomology.get_two_cells(filtration)
    tic2 = time.time()
    res2 = get_two_cells(filtration.distances)
    tic3 = time.time()
    #print(tic2 - tic1)
    delta = tic3 - tic2
    log_deltas.append(log(delta))
    print(delta)
    G.clear()

print(log_deltas)

