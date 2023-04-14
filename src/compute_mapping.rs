// Placeholder

use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use dashmap::DashMap;

use pyo3::prelude::*;

use crate::{
    columns::{ColumnType, GrpphatiRsColumn},
    NodeIndex,
};

use rayon::prelude::*;

type NodeMapping = HashMap<NodeIndex, NodeIndex>;

#[pyfunction]
#[pyo3(name = "compute_rph_map")]
pub fn compute_map_py(
    domain_cells: Vec<GrpphatiRsColumn>,
    codomain_cells: Vec<GrpphatiRsColumn>,
    vertex_map: NodeMapping,
) -> Vec<Vec<usize>> {
    compute_map(&domain_cells, &codomain_cells, vertex_map)
}

pub fn compute_map(
    domain_cells: &Vec<GrpphatiRsColumn>,
    codomain_cells: &Vec<GrpphatiRsColumn>,
    vertex_map: NodeMapping,
) -> Vec<Vec<usize>> {
    let index = build_index(codomain_cells);
    domain_cells
        .par_iter()
        .map(|col| match col.col_type {
            ColumnType::Triangle(i, j, k) => {
                let fi = *vertex_map.get(&i).unwrap();
                let fj = *vertex_map.get(&j).unwrap();
                let fk = *vertex_map.get(&k).unwrap();
                let image_set = compute_two_path_image(&index, (fi, fj, fk));
                image_set.into_iter().sorted().collect()
            }
            ColumnType::LongSquare(s, mids, t) => {
                let fs = *vertex_map.get(&s).unwrap();
                let fu = *vertex_map.get(&mids.0).unwrap();
                let fv = *vertex_map.get(&mids.1).unwrap();
                let ft = *vertex_map.get(&t).unwrap();
                let path_1 = (fs, fu, ft);
                let path_2 = (fs, fv, ft);
                let im_1 = compute_two_path_image(&index, path_1);
                let im_2 = compute_two_path_image(&index, path_2);
                let final_im = im_1.symmetric_difference(&im_2);
                final_im.into_iter().cloned().sorted().collect()
            }
            ColumnType::DoubleEdge(i, j) => {
                let fi = *vertex_map.get(&i).unwrap();
                let fj = *vertex_map.get(&j).unwrap();
                let image_set = compute_two_path_image(&index, (fi, fj, fi));
                image_set.into_iter().sorted().collect()
            }
            ColumnType::Edge(i, j) => {
                let fi = *vertex_map.get(&i).unwrap();
                let fj = *vertex_map.get(&j).unwrap();
                if fi == fj {
                    vec![]
                } else {
                    let im_idx = index.edges.get(&(fi, fj)).unwrap().value().clone();
                    vec![im_idx]
                }
            }
            ColumnType::Node(i) => {
                let fi = *vertex_map.get(&i).unwrap();
                let im_idx = index.nodes.get(&fi).unwrap().value().clone();
                vec![im_idx]
            }
        })
        .collect()
}

// Remember to sort output before returning vector
fn compute_two_path_image(
    index: &CodomainIndex,
    image_path: (NodeIndex, NodeIndex, NodeIndex),
) -> HashSet<usize> {
    if image_path.0 == image_path.2 {
        if image_path.0 == image_path.1 {
            // Path is collapsed to nothing
            return HashSet::default();
        }
        // Image is a double edge
        let im_idx = index
            .double_edges
            .get(&(image_path.0, image_path.1))
            .unwrap()
            .value()
            .clone();
        return HashSet::from([im_idx]);
    }
    if image_path.0 == image_path.1 || image_path.1 == image_path.2 {
        return HashSet::default();
    }
    // Image is a two-path with all distinct vertices
    // Must be combination of long square and directed triangles
    if index.triangles.contains_key(&image_path) {
        let im_idx = index.triangles.get(&image_path).unwrap().value().clone();
        return HashSet::from([im_idx]);
    }
    // Image must be contained in a long square
    // We fetch the index of that long square and the index of the triangle corresponding
    // to the other half of the long square
    let ls_idx = index.long_squares.get(&image_path).unwrap().value().clone();
    let base_node = index
        .bases
        .get(&(image_path.0, image_path.2))
        .unwrap()
        .value()
        .clone();
    let base_idx = index
        .triangles
        .get(&(image_path.0, base_node, image_path.2))
        .unwrap()
        .value()
        .clone();
    HashSet::from([ls_idx, base_idx])
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
    // For a pair (s, t) long_squares[(s, m, t)] is the index of long square smt - s(base)t
    long_squares: DashMap<(NodeIndex, NodeIndex, NodeIndex), usize>,
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
                index.long_squares.insert((s, mid.1, t), idx);
            }
        });
    index
}
