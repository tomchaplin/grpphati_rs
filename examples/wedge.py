import networkx as nx
from grpphati_rs import GrPPH_rs


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
