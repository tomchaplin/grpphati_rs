from grpphati.homologies import Homology
from grpphati.filtrations import Filtration
from grpphati_rs.grpphati_rs import get_rph_two_cells, GrpphatiRsColumn, compute_rph_map


class RustRegularPathHomology(Homology):
    @classmethod
    def get_zero_cells(cls, filtration: Filtration):
        return [
            GrpphatiRsColumn("Node", [node], time)
            for node, time in filtration.node_iter()
        ]

    @classmethod
    def get_one_cells(cls, filtration: Filtration):
        return [
            GrpphatiRsColumn("Edge", [edge[0], edge[1]], time)
            for edge, time in filtration.edge_iter()
        ]

    @classmethod
    def get_two_cells(cls, filtration: Filtration):
        return get_rph_two_cells(filtration.edge_dict())

    @classmethod
    def compute_map(cls, domain, codomain, domain_node_list, vertex_map=lambda x: x):
        collected_map = {node: vertex_map(node) for node in domain_node_list}
        return compute_rph_map(domain, codomain, collected_map)

    @staticmethod
    def get_relabelled_inclusion(domain_G, codomain_G, label_attribute="original"):
        def inclusion(x):
            target_original = domain_G.nodes[x][label_attribute]
            for y, data in codomain_G.nodes(data=True):
                if data[label_attribute] == target_original:
                    return y

            raise ValueError(
                f"Node with original label {target_original} does not exist in codomain"
            )

        return inclusion
