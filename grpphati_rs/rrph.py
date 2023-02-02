from grpphati.homologies import Homology
from grpphati.filtrations import Filtration
from grpphati_rs.grpphati_rs import get_rph_two_cells, GrpphatiRsColumn

class RustRegularPathHomology(Homology):
    @classmethod
    def get_zero_cells(cls, filtration: Filtration):
        return [
                GrpphatiRsColumn("Node", [node], time) for node, time in filtration.node_iter()
        ]

    @classmethod
    def get_one_cells(cls, filtration: Filtration):
        return [
                GrpphatiRsColumn("Edge", [edge[0], edge[1]], time) for edge, time in filtration.edge_iter()
        ]

    @classmethod
    def get_two_cells(cls, filtration: Filtration):
        return get_rph_two_cells(filtration.edge_dict())
