//! graph implementations using primitive data types.
//!
//! If you know the maximum number of nodes is less than or equal to 16, 32, 64, or 128, use the corresponding graph type.
//! If you think the number of nodes will exceed 128, use the general [Graph](crate::graph::Graph) implementation.
//!
//! Computing paths with these is over 3x faster than using the general [Graph](crate::graph::Graph) implementation.
//! See [Benchmarks](https://github.com/PoOnesNerfect/bit_gossip#benchmarks) for more details.
//!
//! <br>
//!
//! **Panics** in debug mode if the number of nodes exceeds the maximum number of nodes for the graph type.
//!
//! In release mode, it will saturate at the maximum number of nodes.
//!
//! # Example
//!
//! ## Basic Usage
//!
//! ```sh
//! 0 -- 1 -- 2 -- 3
//! |         |    |
//! 4 -- 5 -- 6 -- 7
//! |         |    |
//! 8 -- 9 -- 10 - 11
//! ```
//!
//! ```
//! use bit_gossip::Graph16;
//!
//! // Initialize a builder with 12 nodes
//! let mut builder = Graph16::builder(12);
//!
//! // Connect the nodes
//! for i in 0..12u8 {
//!     if i % 4 != 3 {
//!         builder.connect(i, i + 1);
//!     }
//!     if i < 8 {
//!         builder.connect(i, i + 4);
//!     }
//! }
//! builder.disconnect(1, 5);
//! builder.disconnect(5, 9);
//!
//! // Build the graph
//! let graph = builder.build();
//!
//! // Check the shortest path from 0 to 9
//! assert_eq!(graph.next_node(0, 9), Some(4));
//! assert_eq!(graph.next_node(4, 9), Some(8));
//! assert_eq!(graph.next_node(8, 9), Some(9));
//!
//! // Both 1 and 4 can reach 11 in the shortest path.
//! assert_eq!(graph.next_nodes(0, 11).collect::<Vec<_>>(), vec![1, 4]);
//!
//! // Get the path from 0 to 5
//! assert_eq!(graph.path_to(0, 5).collect::<Vec<_>>(), vec![4, 5]);
//! ```
//!
//! ## Do not exceed the maximum number of nodes for the graph type.
//!
//! ```should_panic
//! use bit_gossip::Graph16;
//!
//! // This will panic in debug mode,
//! // and saturate at 16 in release mode
//! let mut builder = Graph16::builder(17);
//! ```

use crate::edge_id;
use paste::paste;
use std::{collections::HashMap, fmt::Debug};

// macros were about 2x faster than using generics
macro_rules! impl_prim {
    ($node_bits:ty, $node_id:ty, $num:expr) => {
        paste! {
            #[doc = "Graph implementation using `" $node_bits "` as the node bits storage."]
            ///
            #[doc = "Number of nodes must be equal or lower than " $num "."]
            ///
            /// <br>
            ///
            /// **panics** in debug mode if given number of nodes exceeds
            #[doc = $num "."]
            ///
            /// In release mode, it will saturate at the maximum number of nodes.
            #[derive(Debug, Clone)]
            pub struct [< Graph $num >] {
                pub nodes: [<Nodes $num>],
                pub edges: HashMap<($node_id, $node_id), $node_bits>,
            }

            impl [< Graph $num >] {
                pub fn builder(nodes_len: usize) -> [<Graph $num Builder>] {
                    debug_assert!(nodes_len <= $num, "Number of nodes must be equal or lower than {}", $num);

                    [<Graph $num Builder>]::new(nodes_len.min($num))
                }

                pub fn into_builder(self) -> [<Graph $num Builder>] {
                    [<Graph $num Builder>] {
                        nodes: self.nodes,
                        edge_masks: [<Edges $num>] { inner: self.edges.iter().map(|(k, _)| (*k, 0)).collect() },
                        edges: [<Edges $num>] { inner: self.edges },
                    }
                }

                /// Given a current node and a destination node,
                /// return the first neighboring node that is the shortest path to the destination node.
                ///
                /// This operation is very fast as all paths for all nodes are precomputed.
                ///
                /// `None` is returned when:
                /// - `curr` and `dest` are the same node
                /// - `curr` has no path to `dest`
                ///
                /// **Note:** In case there are multiple neighboring nodes that lead to the destination node,
                /// the first one found will be returned. The same node will be returned for the same input.
                /// However, the order of the nodes is not guaranteed.
                ///
                /// You can use [next_node_with](Self::next_node_with) to filter matching neighbors,
                /// or [next_nodes](Self::next_nodes) to get all neighboring nodes.
                #[inline]
                pub fn next_node(&self, curr: $node_id, dest: $node_id) -> Option<$node_id> {
                    self.next_nodes(curr, dest).next()
                }

                /// Given a current node and a destination node, and a filter function,
                /// return the neighboring node of current that is the shortest path to the destination node.
                ///
                /// Same as `self.next_nodes(curr, dest).find(f)`
                ///
                /// This may be useful if you want some custom behavior when choosing the next node.
                ///
                /// **Ex)** In a game, you might want to randomize which path to take when there are multiple shortest paths.
                ///
                /// `None` is returned when:
                /// - `curr` and `dest` are the same node
                /// - `curr` has no path to `dest`
                /// - The filter function returns `false` for all neighboring nodes
                #[inline]
                pub fn next_node_with(
                    &self,
                    curr: $node_id,
                    dest: $node_id,
                    f: impl Fn($node_id) -> bool,
                ) -> Option<$node_id> {
                    self.next_nodes(curr, dest).find(|&n| f(n))
                }

                /// Given a current node and a destination node,
                /// return all neighboring nodes of current that are shortest paths to the destination node.
                ///
                /// The nodes will be returned in the same order for the same inputs. However, the ordering of the nodes is not guaranteed.
                #[inline]
                pub fn next_nodes(&self, curr: $node_id, dest: $node_id) -> [<NextNodesIter $num>]<'_> {
                    [<NextNodesIter $num>] {
                        graph: self,
                        neighbors: self.nodes.neighbors(curr),
                        curr,
                        dest,
                    }
                }

                /// Given a current node and a destination node,
                /// return a path from the current node to the destination node.
                ///
                /// The path is a list of node IDs, starting with the next node (not current node!) and ending at the destination node.
                #[inline]
                pub fn path_to(&self, curr: $node_id, dest: $node_id) -> [<PathIter $num>]<'_> {
                    [<PathIter $num>] {
                        map: self,
                        curr,
                        dest,
                        done: false,
                    }
                }

                /// Return a list of all neighboring nodes of the given node.
                #[inline]
                pub fn neighbors(&self, node: $node_id) -> impl Iterator<Item = $node_id> + '_  {
                    self.nodes.neighbors(node)
                }

                /// Return the number of nodes in this graph.
                #[inline]
                pub fn nodes_len(&self) -> usize {
                    self.nodes.len()
                }

                /// Return the number of edges in this graph.
                #[inline]
                pub fn edges_len(&self) -> usize {
                    self.edges.len()
                }
            }

            /// Iterator that returns a path from the current node to the destination node.
            ///
            /// Current node is not included in the path.
            #[derive(Debug)]
            pub struct [<PathIter $num>]<'a> {
                map: &'a [<Graph $num>],
                curr: $node_id,
                dest: $node_id,
                done: bool,
            }

            impl Iterator for [<PathIter $num>]<'_> {
                type Item = $node_id;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.done || self.curr == self.dest {
                        return None;
                    }

                    let Some(next) = self.map.next_node(self.curr, self.dest) else {
                        self.done = true;
                        return None;
                    };

                    self.curr = next;

                    Some(next)
                }
            }

            /// Iterator that iterates neighboring nodes which are the shortest paths to the destination node.
            #[derive(Debug)]
            pub struct [<NextNodesIter $num>]<'a> {
                graph: &'a [<Graph $num>],
                curr: $node_id,
                dest: $node_id,
                neighbors: [<NodeBits $num Iter>],
            }

            impl Iterator for [<NextNodesIter $num>]<'_> {
                type Item = $node_id;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.curr == self.dest {
                        return None;
                    }

                    while let Some(neighbor) = self.neighbors.next() {
                        let bit = self.graph.edges.get(&edge_id(self.curr, neighbor))? & 1 << self.dest > 0;
                        let bit = if self.curr > neighbor { !bit } else { bit };

                        if bit {
                            return Some(neighbor);
                        }
                    }

                    None
                }
            }

            #[doc = "Builder for the Graph" $num]
            #[derive(Debug, Clone)]
            pub struct [<Graph $num Builder>] {
                pub nodes: [<Nodes $num>],

                /// key: edge_id
                /// value: for each bit, if this edge is the shortest path
                /// to that bit location's node, bit is set to 1
                pub edges: [<Edges $num>],

                /// key: edge_id
                /// value: for each edge, bit is set to 1 if the node is computed for this edge
                pub edge_masks: [<Edges $num>],
            }

            impl [<Graph $num Builder>] {
                pub fn new(nodes_len: usize) -> Self {
                    Self {
                        nodes: [<Nodes $num>]::new(nodes_len),
                        edges: [<Edges $num>]::new(),
                        edge_masks: [<Edges $num>]::new(),
                    }
                }

                /// Add a edge between node_a and node_b
                pub fn connect(&mut self, a: $node_id, b: $node_id) {
                    // if the edge already exists, return
                    if !self.nodes.connect(a, b) {
                        return;
                    }

                    let a_bit = 1 << a;
                    let b_bit = 1 << b;

                    let mut val = b_bit;

                    // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
                    // - gets further away from b
                    // - shortest path to a
                    // - gets further away from all other nodes
                    if a > b {
                        val = a_bit;
                    }

                    let ab = edge_id(a, b);

                    self.edges.insert(ab, val);
                    self.edge_masks.insert(ab, a_bit | b_bit);
                }

                pub fn disconnect(&mut self, a: $node_id, b: $node_id) {
                    // if the edge doesn't exist, return
                    if self.nodes.disconnect(a, b) {
                        return;
                    }

                    let ab = edge_id(a, b);

                    if self.edges.inner.remove(&ab).is_some() {
                        self.edge_masks.inner.remove(&ab);
                    }
                }

                pub fn process(&mut self) {
                    let mut this = std::mem::replace(self, Self::new(1));
                    this = this.build().into_builder();
                    let _ = std::mem::replace(self, this);
                }

                pub fn build(self) -> [< Graph $num >] {
                    let Self {
                        nodes,
                        mut edges,
                        mut edge_masks,
                    } = self;

                    // (neighbors at current depth, neighbors at previous depths)
                    let mut neighbors_at_depth: Vec<($node_bits, $node_bits)> =
                        nodes.inner.iter().enumerate().map(|(i, e)| (*e, 1 << i)).collect();

                    let mut active_neighbors_mask: $node_bits = 0;

                    // each rooom's bit is set to 1 if all its edges are done computed
                    let mut done_mask: $node_bits = 0;

                    // Temporary storage for upserts
                    // so we don't have to allocate every iteration
                    // (edge_val, mask, computed_mask)
                    let mut upserts: Vec<($node_bits, $node_bits, $node_bits)> = Vec::new();

                    let last_node_bit = 1 << (nodes.inner.len() - 1);
                    let full_mask: $node_bits = last_node_bit | (last_node_bit - 1);

                    // setup
                    for (a, a_neighbors) in &nodes {
                        let a_neighbors_len = a_neighbors.len() as usize;

                        // clear upserts
                        if upserts.len() < a_neighbors_len {
                            upserts.resize(a_neighbors_len, (0, 0, 0));
                        } else {
                            upserts.fill((0, 0, 0));
                        }

                        // for each edge in this node
                        // set the value for a and b's node as 1
                        for (i, b) in a_neighbors.enumerate() {
                            let a_bit = 1 << a;
                            let b_bit = 1 << b;

                            let mut val = b_bit;

                            // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
                            // - gets further away from b
                            // - shortest path to a
                            // - gets further away from all other nodes
                            if a > b {
                                val = a_bit;
                            }

                            // for all other edges in this node, set the value for this node bit as 0
                            for (j, c) in a_neighbors.clone().enumerate() {
                                if i == j {
                                    continue;
                                }

                                // if both b and c are in the same corner (tl or br)
                                // flip the bit
                                let upsert = if (a > b) == (a > c) {
                                    !val & b_bit
                                } else {
                                    val & b_bit
                                };

                                let vals = &mut upserts[j];
                                vals.0 |= upsert;
                                vals.1 |= b_bit;
                            }
                        }

                        // apply computed values
                        for (i, b) in a_neighbors.enumerate() {
                            let ab = edge_id(a, b);

                            let (upsert, computed, _) = upserts[i];

                            if computed != 0 {
                                if upsert != 0 {
                                    edges.insert(ab, upsert);
                                }
                                edge_masks.insert(ab, computed);
                            }
                        }
                    }

                    'outer: while done_mask != full_mask {
                        // iterate through all undone nodes
                        for a in [<node_bits_ $num _iter>](full_mask ^ done_mask) {
                            let a_bit = 1 << a;
                            let a_neighbors = nodes.neighbors(a);
                            let a_neighbors_len = a_neighbors.len() as usize;

                            // clear upserts
                            if upserts.len() < a_neighbors_len {
                                upserts.resize(a_neighbors_len, (0, 0, 0));
                            } else {
                                upserts.fill((0, 0, 0));
                            }

                            // collect all nodes that need to update their neighbors to next depth
                            let mut a_active_neighbors_mask = 0;

                            // are all edges computed for this node?
                            let mut all_edges_done = true;

                            // get all neighbors' masks
                            // so we can just reuse it
                            for (i, b) in a_neighbors.enumerate() {
                                let mask = edge_masks.get(edge_id(a, b)).unwrap();
                                upserts[i].2 = mask;

                                if mask != full_mask {
                                    all_edges_done = false;
                                }
                            }

                            if all_edges_done {
                                done_mask |= a_bit;

                                continue;
                            }

                            for (i, b) in a_neighbors.enumerate() {
                                let ab = edge_id(a, b);

                                // neighbors' bits to gossip from edge a->b to other edges
                                let neighbors_mask = neighbors_at_depth.get(b as usize).unwrap().0 & !a_bit;

                                // if no neighbors to gossip at this depth, skip
                                if neighbors_mask == 0 {
                                    continue;
                                }

                                a_active_neighbors_mask |= 1 << b;

                                let val = edges.get(ab).unwrap();

                                // gossip to other edges about its neighbors at current depth
                                for (j, c) in a_neighbors.enumerate() {
                                    // skip if same neighbor
                                    if i == j {
                                        continue;
                                    }

                                    let mask_ac = upserts[j].2;
                                    if mask_ac == full_mask {
                                        continue;
                                    }
                                    all_edges_done = false;

                                    // dont set bits that are already computed
                                    let compute_mask = neighbors_mask & !mask_ac;

                                    // if all bits are already computed, skip
                                    if compute_mask == 0 {
                                        continue;
                                    }

                                    // if both b and c are in the same corner (tl or br)
                                    // flip the bit
                                    let upsert = if (a > b) == (a > c) { !val } else { val } & compute_mask;

                                    let vals = &mut upserts[j];
                                    vals.0 |= upsert;
                                    vals.1 |= compute_mask;
                                }
                            }

                            // if all edges are computed or none of a's neighbors are active,
                            // then a is done
                            if all_edges_done || a_active_neighbors_mask == 0 {
                                done_mask |= a_bit;
                            } else {
                                for (i, b) in a_neighbors.enumerate() {
                                    let ab = edge_id(a, b);

                                    let (upsert, computed, _) = upserts[i];

                                    if computed != 0 {
                                        if upsert != 0 {
                                            edges.insert(ab, upsert);
                                        }
                                        edge_masks.insert(ab, computed);
                                    }
                                }
                            }

                            // if all nodes are done, return true
                            if done_mask == full_mask {
                                break 'outer;
                            }

                            active_neighbors_mask |= a_active_neighbors_mask;
                        }

                        // iterate through active neighbors that were colleted this iteration
                        // and get the next layer of neighbors for each node.
                        // if new_neighbors is 0, then all neighbors are computed.
                        for a in [<node_bits_ $num _iter>](active_neighbors_mask) {
                            let a_usize = a as usize;
                            let (a_neighbors_at_depth, mut prev_neighbors) = neighbors_at_depth[a_usize];

                            if a_neighbors_at_depth == 0 {
                                continue;
                            }

                            let mut new_neighbors = 0;
                            for b in [<node_bits_ $num _iter>](a_neighbors_at_depth) {
                                new_neighbors |= nodes.neighbors(b).node_bits;
                            }

                            // add previous neighbors to computed
                            prev_neighbors |= a_neighbors_at_depth;

                            // new neighbors at this depth without the previous neighbors
                            new_neighbors &= !prev_neighbors;
                            neighbors_at_depth[a_usize] = (new_neighbors, prev_neighbors);
                        }

                        active_neighbors_mask = 0;
                    }

                    [< Graph $num >] {
                        nodes,
                        edges: edges.inner,
                    }
                }
            }

            /// Map of nodes and their neighbors.
            ///
            /// index: node_id
            ///
            #[doc = "value: " $node_bits " with neighbors' bit locations set to `true`"]
            #[derive(Debug, Clone)]
            pub struct [<Nodes $num>] {
                pub inner: Vec<$node_bits>,
            }

            impl [<Nodes $num>] {
                pub fn new(nodes_len: usize) -> Self {
                    Self {
                        inner: vec![0; nodes_len],
                    }
                }

                /// Get the neighboring nodes
                #[inline]
                pub fn neighbors(&self, node: $node_id) -> [<NodeBits $num Iter>] {
                    [<node_bits_ $num _iter>](self.inner[node as usize])
                }

                /// Add a edge between node_a and node_b
                /// If the edge was not added, return false
                pub fn connect(&mut self, a: $node_id, b: $node_id) -> bool {
                    if a == b {
                        return false;
                    }

                    let b_bit = 1 << b;

                    self.inner[a as usize] |= b_bit;
                    self.inner[b as usize] |= 1 << a;

                    true
                }

                /// Remove a edge between node_a and node_b
                /// If the edge was not removed, return false
                pub fn disconnect(&mut self, a: $node_id, b: $node_id) -> bool {
                    if a == b {
                        return false;
                    }

                    let b_bit = 1 << b;

                    self.inner[a as usize] &= !b_bit;
                    self.inner[b as usize] &= !(1 << a);

                    true
                }

                #[inline]
                pub fn edge_count(&self, node: $node_id) -> u32 {
                    self.inner[node as usize].count_ones()
                }

                #[inline]
                pub fn len(&self) -> usize {
                    self.inner.len()
                }
            }

            /// Map of edges and bits indicating if the edge is the shortest path to the node.
            #[derive(Debug, Clone)]
            pub struct [<Edges $num>] {
                /// key: edge_id
                /// value: for each bit, if this edge is the shortest path
                /// to that bit location's node, bit is set to 1
                inner: HashMap<($node_id, $node_id), $node_bits>,
            }

            impl [<Edges $num>] {
                fn new() -> Self {
                    Self {
                        inner: HashMap::new(),
                    }
                }

                #[inline]
                pub fn get(&self, edge_id: ($node_id, $node_id)) -> Option<$node_bits> {
                    self.inner.get(&edge_id).cloned()
                }

                #[inline]
                pub fn insert(&mut self, edge_id: ($node_id, $node_id), val: $node_bits) {
                    if let Some(edge) = self.inner.get_mut(&edge_id) {
                        *edge |= val;
                    } else {
                        self.inner.insert(edge_id, val);
                    }
                }
            }

            impl<'a> IntoIterator for &'a [<Nodes $num>] {
                type Item = ($node_id, [<NodeBits $num Iter>]);
                type IntoIter = [<Neighbors $num Iter>]<'a>;

                fn into_iter(self) -> Self::IntoIter {
                    [<Neighbors $num Iter>] {
                        neighbors: self,
                        node: 0,
                    }
                }
            }

            /// Iterator that iterates through all nodes and their neighbors.
            pub struct [<Neighbors $num Iter>]<'a> {
                neighbors: &'a [<Nodes $num>],
                node: $node_id,
            }

            impl<'a> Iterator for [<Neighbors $num Iter>]<'a> {
                type Item = ($node_id, [<NodeBits $num Iter>]);

                fn next(&mut self) -> Option<Self::Item> {
                    let node = self.node;

                    if node as usize >= self.neighbors.len() {
                        return None;
                    }

                    self.node += 1;
                    self.neighbors
                        .inner
                        .get(node as usize)
                        .map(|connected| (node, [<node_bits_ $num _iter>](*connected)))
                }
            }

            fn [<node_bits_ $num _iter>](node_bits: $node_bits) -> [<NodeBits $num Iter>] {
                [<NodeBits $num Iter>] { node_bits }
            }

            /// Given a value with bits set to 1 at existing nodes' indices,
            /// iterate through the node indices
            #[derive(Clone, Copy)]
            pub struct [<NodeBits $num Iter>] {
                node_bits: $node_bits,
            }

            impl Debug for [<NodeBits $num Iter>] {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:016b}", self.node_bits)
                }
            }

            impl [<NodeBits $num Iter>] {
                pub fn without(self, node: $node_id) -> Self {
                    Self {
                        node_bits: self.node_bits & !(1 << node),
                    }
                }

                #[inline]
                pub fn len(&self) -> u32 {
                    self.node_bits.count_ones()
                }
            }

            impl Iterator for [<NodeBits $num Iter>] {
                type Item = $node_id;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.node_bits == 0 {
                        return None;
                    }

                    // index of the next connected edge
                    let node = self.node_bits.trailing_zeros();

                    // remove the connected edge from the node_bits
                    self.node_bits &= !(1 << node);

                    Some(node as $node_id)
                }
            }
        }
    };
}
impl_prim!(u16, u8, 16);
impl_prim!(u32, u8, 32);
impl_prim!(u64, u8, 64);
impl_prim!(u128, u8, 128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_16() {
        pub const NODES_X_LEN: usize = 4;
        pub const NODES_Y_LEN: usize = 4;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = Graph16Builder::new(NODES_LEN);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = (node_id - 1) as u8;
                    let b = node_id as u8;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id as u8;
                    let b = (node_id - NODES_X_LEN) as u8;
                    builder.connect(a, b);
                }
            }
        }

        let now = std::time::Instant::now();
        let _graph = builder.build();
        println!("Time: {:?}", now.elapsed());
    }

    #[test]
    fn test_graph_32() {
        pub const NODES_X_LEN: usize = 4;
        pub const NODES_Y_LEN: usize = 8;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = Graph32Builder::new(NODES_LEN);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = (node_id - 1) as u8;
                    let b = node_id as u8;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id as u8;
                    let b = (node_id - NODES_X_LEN) as u8;
                    builder.connect(a, b);
                }
            }
        }

        let now = std::time::Instant::now();
        let _graph = builder.build();
        println!("Time: {:?}", now.elapsed());
    }

    #[test]
    fn test_graph_64() {
        pub const NODES_X_LEN: usize = 8;
        pub const NODES_Y_LEN: usize = 8;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = Graph64Builder::new(NODES_LEN);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = (node_id - 1) as u8;
                    let b = node_id as u8;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id as u8;
                    let b = (node_id - NODES_X_LEN) as u8;
                    builder.connect(a, b);
                }
            }
        }

        let now = std::time::Instant::now();
        let _graph = builder.build();
        println!("Time: {:?}", now.elapsed());
    }

    #[test]
    fn test_graph_128() {
        pub const NODES_X_LEN: usize = 8;
        pub const NODES_Y_LEN: usize = 16;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = Graph128Builder::new(NODES_LEN);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = (node_id - 1) as u8;
                    let b = node_id as u8;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id as u8;
                    let b = (node_id - NODES_X_LEN) as u8;
                    builder.connect(a, b);
                }
            }
        }

        let now = std::time::Instant::now();
        let _graph = builder.build();
        println!("Time: {:?}", now.elapsed());
    }
}
