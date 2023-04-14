import sys
import os

sys.path.append(os.getcwd())

from grpphati.filtrations import ShortestPathFiltration
from grpphati_rs import RustRegularPathHomology

import networkx as nx


def build_grounded_cols(G):
    filtration = ShortestPathFiltration(G)
    grounded_filtration = filtration.ground(G)
    # Build boundary matrix
    cols = RustRegularPathHomology.get_cells(
        [0, 1], grounded_filtration
    ) + RustRegularPathHomology.get_cells([2], filtration)
    cols.sort(key=lambda col: (col.get_entrance_time(), col.dimension()))
    return cols


identity = lambda x: x

G1 = nx.DiGraph()
G1.add_edges_from(
    [
        (0, 1, {"weight": 2}),
        (1, 3, {"weight": 2}),
        (0, 2, {"weight": 3}),
        (2, 3, {"weight": 3}),
    ]
)

G2 = nx.DiGraph()
G2.add_edges_from(
    [
        (0, 1, {"weight": 1}),
        (1, 3, {"weight": 1}),
        (0, 2, {"weight": 0.2}),
        (2, 3, {"weight": 0.2}),
    ]
)

domain_1 = build_grounded_cols(G1)
codomain_1 = build_grounded_cols(G2)

cell_map_1 = RustRegularPathHomology.compute_map(
    domain_1, codomain_1, list(G1.nodes), identity
)


print("--> G6")
N = 10
G6_1 = nx.relabel_nodes(
    nx.complete_graph(N, create_using=nx.DiGraph), lambda x: (x, 1) if x > 0 else x
)
G6_2 = nx.relabel_nodes(
    nx.complete_graph(N, create_using=nx.DiGraph), lambda x: (x, 2) if x > 0 else x
)
G6 = nx.compose(G6_1, G6_2)

# Convert to integer node labels for grpphati_rs compatability
G6_1 = nx.convert_node_labels_to_integers(G6_1, label_attribute="original")
G6 = nx.convert_node_labels_to_integers(G6, label_attribute="original")
# Now that we have relabelled, the inclusion is non-trivial
inclusion = RustRegularPathHomology.get_relabelled_inclusion(
    G6_1, G6, label_attribute="original"
)


domain_2 = build_grounded_cols(G6_1)
codomain_2 = build_grounded_cols(G6)
print("Got cells")
cell_map_2 = RustRegularPathHomology.compute_map(
    domain_2, codomain_2, list(G6_1.nodes), inclusion
)
print("Got map")
