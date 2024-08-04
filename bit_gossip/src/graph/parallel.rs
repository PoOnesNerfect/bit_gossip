use super::U16orU32;
use crate::{
    bitvec::{AtomicBitVec, BitVec},
    edge_id,
};
use rayon::prelude::*;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug)]
pub struct ParaGraph<NodeId: U16orU32 = u16> {
    pub nodes: Nodes<NodeId>,
    pub edges: HashMap<(NodeId, NodeId), AtomicBitVec>,
}

impl<NodeId: U16orU32> ParaGraph<NodeId> {
    /// Create a new ParaGraphBuilder with the given number of nodes.
    ///
    /// Default NodeId is u16, which can hold up to 65536 nodes.
    /// If you need more nodes, you can specify u32 as the NodeId type, like `ParaGraph::<u32>::builder(100_000)`
    #[inline]
    pub fn builder(nodes_len: usize) -> ParaGraphBuilder<NodeId> {
        assert!(
            nodes_len <= NodeId::MAX_NODES,
            "Number of nodes exceeds the limit; Specify `u32` as the NodeId type, like `ParaGraph::<u32>::builder(100_000)`"
        );

        ParaGraphBuilder::new(nodes_len.min(NodeId::MAX_NODES))
    }

    /// Convert this ParaGraph into a ParaGraphBuilder.
    #[inline]
    pub fn into_builder(self) -> ParaGraphBuilder<NodeId> {
        ParaGraphBuilder {
            edge_masks: Edges {
                inner: self
                    .edges
                    .iter()
                    .map(|(k, _)| (*k, AtomicBitVec::zeros(self.nodes.len())))
                    .collect(),
            },
            edges: Edges { inner: self.edges },
            nodes: self.nodes,
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
    /// If you would like to have some custom behavior when choosing the next node,
    /// you can use the `next_node_with` method, or the `next_nodes` method to get all neighboring nodes.
    #[inline]
    pub fn next_node(&self, curr: NodeId, dest: NodeId) -> Option<NodeId> {
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
        curr: NodeId,
        dest: NodeId,
        f: impl Fn(NodeId) -> bool,
    ) -> Option<NodeId> {
        self.next_nodes(curr, dest).find(|&n| f(n))
    }

    /// Given a current node and a destination node,
    /// return all neighboring nodes of current that are shortest paths to the destination node.
    ///
    /// The nodes will be returned in the same order for the same inputs. However, the ordering of the nodes is not guaranteed.
    #[inline]
    pub fn next_nodes(&self, curr: NodeId, dest: NodeId) -> NextNodesIter<'_, NodeId> {
        NextNodesIter {
            graph: self,
            neighbors: self.nodes.neighbors(curr).iter(),
            curr,
            dest,
        }
    }

    /// Given a current node and a destination node,
    /// return a path from the current node to the destination node.
    ///
    /// The path is a list of node IDs, starting with the next node (not current node!) and ending at the destination node.
    #[inline]
    pub fn path_to(&self, curr: NodeId, dest: NodeId) -> PathIter<'_, NodeId> {
        PathIter {
            map: self,
            curr,
            dest,
        }
    }

    /// Check if there is a path from the current node to the destination node.
    #[inline]
    pub fn path_exists(&self, curr: NodeId, dest: NodeId) -> bool {
        self.next_node(curr, dest).is_some()
    }

    /// Return a list of all neighboring nodes of the given node.
    #[inline]
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
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

/// An iterator that returns a path from the current node to the destination node.
///
/// Current node is not included in the path.
#[derive(Debug)]
pub struct PathIter<'a, NodeId: U16orU32> {
    map: &'a ParaGraph<NodeId>,
    curr: NodeId,
    dest: NodeId,
}

impl<NodeId: U16orU32> Iterator for PathIter<'_, NodeId> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.dest {
            return None;
        }

        let Some(next) = self.map.next_node(self.curr, self.dest) else {
            return None;
        };

        self.curr = next;

        Some(next)
    }
}

#[derive(Debug)]
pub struct NextNodesIter<'a, NodeId: U16orU32> {
    graph: &'a ParaGraph<NodeId>,
    curr: NodeId,
    dest: NodeId,
    neighbors: std::slice::Iter<'a, NodeId>,
}

impl<NodeId: U16orU32> Iterator for NextNodesIter<'_, NodeId> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.dest {
            return None;
        }

        while let Some(&neighbor) = self.neighbors.next() {
            let bit = self
                .graph
                .edges
                .get(&edge_id(self.curr, neighbor))?
                .get_bit(self.dest.as_usize());
            let bit = if self.curr > neighbor { !bit } else { bit };

            if bit {
                return Some(neighbor);
            }
        }

        None
    }
}

/// A builder for creating a ParaGraph.
#[derive(Debug)]
pub struct ParaGraphBuilder<NodeId: U16orU32> {
    /// key: node_id
    /// value: neighbors of node
    pub nodes: Nodes<NodeId>,

    /// key: edge_id
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    pub edges: Edges<NodeId>,

    /// key: edge_id
    /// value: for each edge, bit is set to 1 if the node with the bit location is computed for this edge
    pub edge_masks: Edges<NodeId>,
}

impl<NodeId: U16orU32> ParaGraphBuilder<NodeId> {
    /// Create a new ParaGraphBuilder with the given number of nodes.
    #[inline]
    pub fn new(nodes_len: usize) -> Self {
        Self {
            nodes: Nodes::new(nodes_len),
            edges: Edges::new(),
            edge_masks: Edges::new(),
        }
    }

    /// Add an edge between node_a and node_b
    pub fn connect(&mut self, a: NodeId, b: NodeId) {
        self.nodes.connect(a, b);

        // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
        // - gets further away from b
        // - shortest path to a
        // - gets further away from all other nodes
        let val = if a > b { a } else { b };

        let ab = edge_id(a, b);

        if let Some(edge) = self.edges.inner.get_mut(&ab) {
            edge.set_bit(val.as_usize(), true);
        } else {
            let edge = AtomicBitVec::one(val.as_usize(), self.nodes.len());

            self.edges.inner.insert(ab, edge);
        }

        if let Some(edge) = self.edge_masks.inner.get_mut(&ab) {
            edge.set_bit(a.as_usize(), true);
            edge.set_bit(b.as_usize(), true);
        } else {
            let edge = AtomicBitVec::zeros(self.nodes.len());
            edge.set_bit(a.as_usize(), true);
            edge.set_bit(b.as_usize(), true);

            self.edge_masks.inner.insert(ab, edge);
        }
    }

    /// Remove an edge between node_a and node_b
    pub fn disconnect(&mut self, a: NodeId, b: NodeId) {
        // if the edge doesn't exist, return
        self.nodes.disconnect(a, b);

        let ab = edge_id(a, b);

        if self.edge_masks.inner.remove(&ab).is_some() {
            self.edges.inner.remove(&ab);
        }
    }

    /// Build the ParaGraph from the current state of the builder.
    pub fn build(self) -> ParaGraph<NodeId> {
        let Self {
            nodes,
            edges,
            edge_masks,
            ..
        } = self;

        let chunk_size = 8;

        // (neighbors at current depth, neighbors at previous depths)
        let neighbors_at_depth: Vec<(AtomicBitVec, AtomicBitVec)> = nodes
            .inner
            .par_iter()
            .enumerate()
            .map(|(i, e)| {
                let neighbors = AtomicBitVec::zeros(nodes.len());
                for n in e {
                    neighbors.set_bit(n.as_usize(), true);
                }
                (neighbors, AtomicBitVec::one(i, nodes.len()))
            })
            .collect();

        let active_neighbors_mask = AtomicBitVec::zeros(nodes.len());

        // each rooom's bit is set to 1 if all its edges are done computed
        let done_nodes = AtomicBitVec::zeros(nodes.len());

        let full_mask = BitVec::ones(nodes.len());

        nodes
            .inner
            .par_iter()
            .enumerate()
            .chunks(chunk_size)
            .for_each(|nodes| {
                for (a, a_neighbors) in nodes {
                    // setup
                    let mut neighbor_upserts: Vec<(BitVec, BitVec)> =
                        vec![(BitVec::ZERO, BitVec::ZERO); a_neighbors.len()];

                    let a = NodeId::from_usize(a);

                    // for each edge in this node
                    // set the bit value for a and b as 1
                    for (i, b) in a_neighbors.iter().cloned().enumerate() {
                        let b_usize = b.as_usize();

                        let mut val = true;

                        // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
                        // - gets further away from b
                        // - shortest path to a
                        // - gets further away from all other nodes
                        if a > b {
                            val = false;
                        }

                        // for all other edges in this node, set the value for this node bit as 0
                        for (j, c) in a_neighbors.iter().cloned().enumerate() {
                            if i == j {
                                continue;
                            }

                            // if both b and c are in the same corner (tl or br)
                            // flip the bit
                            let should_set = if (a > b) == (a > c) { !val } else { val };

                            let (upsert, computed) = &mut neighbor_upserts[j];
                            if should_set {
                                upsert.set_bit(b_usize, true);
                            }
                            computed.set_bit(b_usize, true);
                        }
                    }

                    // apply computed values
                    for (b, upserts) in a_neighbors.iter().zip(neighbor_upserts.drain(..)) {
                        let ab = edge_id(a, *b);

                        let (upsert, computed) = upserts;

                        if !computed.is_zero() {
                            if !upsert.is_zero() {
                                edges.update(ab, upsert);
                            }
                            edge_masks.update(ab, computed);
                        }
                    }
                }
            });

        loop {
            // iterate through all undone nodes
            done_nodes
                .iter_zeros()
                .chunks(chunk_size)
                .par_bridge()
                .for_each(|e| {
                    for a in e {
                        if a >= nodes.len() {
                            break;
                        }

                        let a_usize = a;
                        let a = NodeId::from_usize(a);

                        let a_neighbors = nodes.neighbors(a);

                        let mut neighbor_upserts: Vec<(BitVec, BitVec)> =
                            vec![(BitVec::ZERO, BitVec::ZERO); a_neighbors.len()];

                        // collect all nodes that need to update their neighbors to next depth
                        let mut a_active_neighbors_mask = BitVec::ZERO;

                        // get all neighbors' masks
                        // so we can just reuse it
                        let mut a_neighbor_masks = Vec::with_capacity(a_neighbors.len());

                        for b in a_neighbors.iter().copied() {
                            let mask = edge_masks.get(edge_id(a, b)).unwrap();

                            if mask.eq(&full_mask) {
                                a_neighbor_masks.push(None);
                            } else {
                                a_neighbor_masks.push(Some(mask));
                            }
                        }

                        // if all edges are computed, skip
                        if a_neighbor_masks.iter().all(Option::is_none) {
                            done_nodes.set_bit(a_usize, true);

                            continue;
                        }

                        for (i, b) in a_neighbors.iter().copied().enumerate() {
                            let b_usize = b.as_usize();

                            // b's neighbors' bits to gossip from edge a->b to other edges
                            let mut b_neighbor_mask_at_d =
                                neighbors_at_depth[b_usize].0.into_bitvec();

                            b_neighbor_mask_at_d.set_bit(a_usize, false);

                            // if no neighbors to gossip at this depth, skip
                            if b_neighbor_mask_at_d.is_zero() {
                                continue;
                            }

                            a_active_neighbors_mask.set_bit(b_usize, true);

                            let ab = edge_id(a, b);

                            let val = edges.get(ab).unwrap().into_bitvec();

                            // gossip to other edges about its neighbors at current depth
                            for (j, c) in a_neighbors.iter().copied().enumerate() {
                                // skip if same neighbor
                                if i == j {
                                    continue;
                                }

                                let Some(mask_ac) = a_neighbor_masks[j] else {
                                    continue;
                                };

                                let mut compute_mask = b_neighbor_mask_at_d.clone();
                                // dont set bits that are already computed
                                compute_mask.bitand_not_assign(&mask_ac.into_bitvec());

                                // if all bits are already computed, skip
                                if compute_mask.is_zero() {
                                    continue;
                                }

                                let (upsert, computed) = &mut neighbor_upserts[j];

                                // if both b and c are in the same corner (tl or br)
                                // flip the bit
                                if (a > b) == (a > c) {
                                    upsert.bitor_not_and_assign(&val, &compute_mask);
                                } else {
                                    upsert.bitor_and_assign(&val, &compute_mask);
                                };

                                computed.bitor_assign(&compute_mask);
                            }
                        }

                        // if all edges are computed or none of a's neighbors are active,
                        // then a is done
                        if a_active_neighbors_mask.is_zero() {
                            done_nodes.set_bit(a_usize, true);
                        } else {
                            for (b, upserts) in
                                a_neighbors.iter().copied().zip(neighbor_upserts.drain(..))
                            {
                                let ab = edge_id(a, b);

                                let (upsert, computed) = upserts;

                                if !computed.is_zero() {
                                    if !upsert.is_zero() {
                                        edges.update(ab, upsert);
                                    }
                                    edge_masks.update(ab, computed);
                                }
                            }
                        }

                        active_neighbors_mask.bitor_assign(&a_active_neighbors_mask);
                    }
                });

            if done_nodes.eq(&full_mask) {
                break;
            }

            active_neighbors_mask
                .iter_ones()
                .chunks(chunk_size)
                .par_bridge()
                .for_each(|e| {
                    for a in e {
                        let (a_neighbors_at_depth, prev_neighbors) = &neighbors_at_depth[a];

                        if a_neighbors_at_depth.is_zero() {
                            continue;
                        }

                        // add previous neighbors to prev neighbors
                        prev_neighbors.bitor_assign_atomic(&a_neighbors_at_depth);

                        let mut new_neighbors = BitVec::ZERO;
                        for b in a_neighbors_at_depth.iter_ones() {
                            for c in nodes.neighbors(NodeId::from_usize(b)) {
                                new_neighbors.set_bit(c.as_usize(), true);
                            }
                        }

                        // new neighbors at this depth without the previous neighbors
                        new_neighbors.bitand_not_assign_atomic(prev_neighbors);
                        a_neighbors_at_depth.assign_from(&new_neighbors);
                    }
                });

            active_neighbors_mask.clear();
        }

        ParaGraph {
            nodes,
            edges: edges.inner,
        }
    }

    /// Return the number of nodes in this graph.
    #[inline]
    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    /// Return the number of edges in this graph.
    #[inline]
    pub fn edges_len(&self) -> usize {
        self.edges.inner.len()
    }

    /// Return the neighbors of the given node.
    #[inline]
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        self.nodes.neighbors(node)
    }
}

/// Map of nodes and their neighbors.
///
/// index: node_id
///
/// value: neighbors of node
#[derive(Debug, Clone)]
pub struct Nodes<NodeId: U16orU32> {
    pub inner: Vec<Vec<NodeId>>,
}

impl<NodeId: U16orU32> Nodes<NodeId> {
    #[inline]
    pub fn new(nodes_len: usize) -> Self {
        Self {
            inner: vec![vec![]; nodes_len],
        }
    }

    pub fn resize(&mut self, nodes_len: usize) {
        self.inner.resize(nodes_len, vec![]);

        if nodes_len < self.inner.len() {
            let nodes_len = NodeId::from_usize(nodes_len);

            for neighbors in self.inner.iter_mut() {
                neighbors.retain(|&i| i < nodes_len);
            }
        }
    }

    /// Get the neighboring nodes
    #[inline]
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        &self.inner[node.as_usize()]
    }

    /// Add a edge between node_a and node_b
    pub fn connect(&mut self, a: NodeId, b: NodeId) {
        if a == b {
            return;
        }

        if !self.inner[a.as_usize()].contains(&b) {
            self.inner[a.as_usize()].push(b);
        }

        self.inner[b.as_usize()].push(a);
    }

    /// Remove a edge between node_a and node_b
    pub fn disconnect(&mut self, a: NodeId, b: NodeId) {
        if a == b {
            return;
        }

        if let Some(index) = self.inner[a.as_usize()].iter().position(|&x| x == b) {
            self.inner[a.as_usize()].swap_remove(index);
        }
        if let Some(index) = self.inner[b.as_usize()].iter().position(|&x| x == a) {
            self.inner[b.as_usize()].swap_remove(index);
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Map of edges and their shortest paths to other nodes.
///
/// key: edge_id
///
/// value: for each bit, if this edge is the shortest path
/// to that bit location's node, bit is set to 1
#[derive(Debug)]
pub struct Edges<NodeId: U16orU32> {
    /// key: edge_id
    ///
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    inner: HashMap<(NodeId, NodeId), AtomicBitVec>,
}

impl<NodeId: U16orU32> Edges<NodeId> {
    #[inline]
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Return the shortest-paths-indicating bit vector.
    #[inline]
    pub fn get(&self, edge_id: (NodeId, NodeId)) -> Option<&AtomicBitVec> {
        self.inner.get(&edge_id)
    }

    /// Insert a new edge with its shortest paths.
    ///
    /// If the edge already exists, the shortest paths will be merged.
    #[inline]
    pub fn insert(&mut self, edge_id: (NodeId, NodeId), val: BitVec, nodes_len: usize) {
        if let Some(bits) = self.inner.get_mut(&edge_id) {
            bits.bitor_assign(&val);
        } else {
            self.inner
                .insert(edge_id, AtomicBitVec::from_bitvec(&val, nodes_len));
        }
    }

    /// Update the shortest paths for the given edge.
    #[inline]
    pub fn update(&self, edge_id: (NodeId, NodeId), val: BitVec) {
        if let Some(bits) = self.inner.get(&edge_id) {
            bits.bitor_assign(&val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_para_graph() {
        type NodeId = u32;

        pub const NODES_X_LEN: NodeId = 200;
        pub const NODES_Y_LEN: NodeId = 100;
        pub const NODES_LEN: NodeId = NODES_X_LEN * NODES_Y_LEN;

        let now = std::time::Instant::now();

        let mut builder = ParaGraphBuilder::new(NODES_LEN as usize);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = node_id - 1;
                    let b = node_id;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id;
                    let b = node_id - NODES_X_LEN;
                    builder.connect(a, b);
                }
            }
        }

        println!("Setup Time: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        let _graph = builder.build();
        println!("Build Time: {:?}", now.elapsed());
    }
}
