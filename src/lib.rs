//! Pathfinding library for calculating all node pairs' shortest paths in an unweighted undirected graph.
//!
//! See [prim] and [graph] modules for more information.

pub mod prim;
pub use prim::{
    Graph128, Graph128Builder, Graph16, Graph16Builder, Graph32, Graph32Builder, Graph64,
    Graph64Builder,
};

pub mod graph;
pub use graph::{Graph, GraphBuilder};

pub mod bitvec;

/// Given two node IDs, return a tuple of the two IDs in ascending order.
#[inline]
pub fn edge_id<T: Ord>(node_a_index: T, node_b_index: T) -> (T, T) {
    if node_a_index > node_b_index {
        (node_b_index, node_a_index)
    } else {
        (node_a_index, node_b_index)
    }
}
