pub mod prim;
pub use prim::*;

pub mod bigmap;
pub use bigmap::{BigMap, BigMapBuilder};

#[cfg(feature = "parallel")]
pub mod parallel;
#[cfg(feature = "parallel")]
pub use parallel::{ParaMap, ParaMapBuilder};

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
