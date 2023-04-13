// Placeholder

use dashmap::DashMap;

use crate::{
    columns::{ColumnType, GrpphatiRsColumn},
    NodeIndex,
};

use rayon::prelude::*;

type NodeMapping = DashMap<NodeIndex, NodeIndex>;

#[allow(dead_code)]
pub fn compute_map(
    domain_cells: &Vec<GrpphatiRsColumn>,
    codomain_cells: &Vec<GrpphatiRsColumn>,
    vertex_map: NodeMapping,
) -> Vec<Vec<usize>> {
    let index = build_index(codomain_cells);
    domain_cells
        .par_iter()
        .map(|col| match col.col_type {
            ColumnType::Triangle(_, _, _) => todo!(),
            ColumnType::LongSquare(_, _, _) => todo!(),
            ColumnType::DoubleEdge(i, j) => {
                let fi = *vertex_map.get(&i).unwrap().value();
                let fj = *vertex_map.get(&j).unwrap().value();
                let im_idx = index.double_edges.get(&(fi, fj)).unwrap().value().clone();
                vec![im_idx]
            }
            ColumnType::Edge(i, j) => {
                let fi = *vertex_map.get(&i).unwrap().value();
                let fj = *vertex_map.get(&j).unwrap().value();
                let im_idx = index.edges.get(&(fi, fj)).unwrap().value().clone();
                vec![im_idx]
            }
            ColumnType::Node(i) => {
                let fi = *vertex_map.get(&i).unwrap().value();
                let im_idx = index.nodes.get(&fi).unwrap().value().clone();
                vec![im_idx]
            }
        })
        .collect()
}

#[derive(Default)]
struct CodomainIndex {
    // Indexes of low-dim columns
    nodes: DashMap<NodeIndex, usize>,
    edges: DashMap<(NodeIndex, NodeIndex), usize>,
    // Indexes of double edges
    double_edges: DashMap<(NodeIndex, NodeIndex), usize>,
    // For a pair (s, t) bases[(s, t)] is the starting midpoint of all long square
    bases: DashMap<(NodeIndex, NodeIndex), NodeIndex>,
    // For a pair (s, t) long_squares[((s, t), m)] is the index of long square s -> t with midpoints (base[(s, t)], m)
    long_squares: DashMap<((NodeIndex, NodeIndex), NodeIndex), usize>,
    // Stores the index of a triangle a -> b -> c in triangles[(a,b,c)]
    triangles: DashMap<(NodeIndex, NodeIndex, NodeIndex), usize>,
}

fn build_index(codomain_cells: &Vec<GrpphatiRsColumn>) -> CodomainIndex {
    let index = CodomainIndex::default();
    codomain_cells
        .iter()
        .enumerate()
        .par_bridge()
        .for_each(|(idx, col)| match col.col_type {
            crate::columns::ColumnType::Edge(i, j) => {
                index.edges.insert((i, j), idx);
            }
            crate::columns::ColumnType::Node(node) => {
                index.nodes.insert(node, idx);
            }
            crate::columns::ColumnType::DoubleEdge(i, j) => {
                index.double_edges.insert((i, j), idx);
            }
            crate::columns::ColumnType::Triangle(i, j, k) => {
                index.triangles.insert((i, j, k), idx);
            }
            crate::columns::ColumnType::LongSquare(s, mid, t) => {
                index.bases.insert((s, t), mid.0);
                index.long_squares.insert(((s, t), mid.1), idx);
            }
        });
    index
}
