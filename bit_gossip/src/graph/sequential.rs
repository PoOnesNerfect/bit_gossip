use super::U16orU32;
use crate::{bitvec::BitVec, edge_id};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct SeqGraph<NodeId: U16orU32 = u16> {
    pub nodes: Nodes<NodeId>,
    pub edges: HashMap<(NodeId, NodeId), BitVec>,
}

impl<NodeId: U16orU32> SeqGraph<NodeId> {
    /// Create a new SeqGraphBuilder with the given number of nodes.
    ///
    /// Default NodeId is u16, which can hold up to 65536 nodes.
    /// If you need more nodes, you can specify u32 as the NodeId type, like `SeqGraph::<u32>::builder(100_000)`
    #[inline]
    pub fn builder(nodes_len: usize) -> SeqGraphBuilder<NodeId> {
        debug_assert!(
            nodes_len <= NodeId::MAX_NODES,
            "Number of nodes exceeds the limit; Specify `u32` as the NodeId type, like `SeqGraph::<u32>::builder(100_000)`"
        );

        SeqGraphBuilder::new(nodes_len.min(NodeId::MAX_NODES))
    }

    /// Converts this graph into a builder.
    ///
    /// This is useful if you want to update the graph,
    /// like resizing nodes or adding/removing edges.
    ///
    /// Then you can build the graph again.
    #[inline]
    pub fn into_builder(self) -> SeqGraphBuilder<NodeId> {
        SeqGraphBuilder {
            edge_masks: Edges {
                inner: self.edges.iter().map(|(k, _)| (*k, BitVec::ZERO)).collect(),
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
    /// You can use [neighbor_to_with](Self::neighbor_to_with) to filter matching neighbors,
    /// or [neighbors_to](Self::neighbors_to) to get all neighboring nodes.
    #[inline]
    pub fn neighbor_to(&self, curr: NodeId, dest: NodeId) -> Option<NodeId> {
        self.neighbors_to(curr, dest).next()
    }

    /// Given a current node and a destination node, and a filter function,
    /// return the neighboring node that is the shortest path to the destination node.
    ///
    /// Same as `self.neighbors_to(curr, dest).find(f)`
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
    pub fn neighbor_to_with(
        &self,
        curr: NodeId,
        dest: NodeId,
        f: impl Fn(NodeId) -> bool,
    ) -> Option<NodeId> {
        self.neighbors_to(curr, dest).find(|&n| f(n))
    }

    /// Given a current node and a destination node,
    /// return all neighboring nodes that are shortest paths to the destination node.
    ///
    /// The nodes will be returned in the same order for the same inputs. However, the ordering of the nodes is not guaranteed.
    #[inline]
    pub fn neighbors_to(&self, curr: NodeId, dest: NodeId) -> NeighborsToIter<'_, NodeId> {
        NeighborsToIter {
            graph: self,
            neighbors: self.nodes.neighbors(curr).iter(),
            curr,
            dest,
        }
    }

    /// Given a current node and a destination node,
    /// return a path from the current node to the destination node.
    ///
    /// The path is a list of node IDs, starting with current node and ending at the destination node.
    ///
    /// This is same as calling `.neighbor_to` repeatedly until the destination node is reached.
    ///
    /// If there is no path, the list will be empty.
    #[inline]
    pub fn path_to(&self, curr: NodeId, dest: NodeId) -> PathIter<'_, NodeId> {
        PathIter {
            map: self,
            curr,
            dest,
            init: false,
        }
    }

    /// Check if there is a path from the current node to the destination node.
    #[inline]
    pub fn path_exists(&self, curr: NodeId, dest: NodeId) -> bool {
        self.neighbor_to(curr, dest).is_some()
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
#[derive(Debug)]
pub struct PathIter<'a, NodeId: U16orU32> {
    map: &'a SeqGraph<NodeId>,
    curr: NodeId,
    dest: NodeId,
    init: bool,
}

impl<NodeId: U16orU32> Iterator for PathIter<'_, NodeId> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.dest {
            return None;
        }

        if !self.init {
            self.init = true;
            return Some(self.curr);
        }

        let Some(next) = self.map.neighbor_to(self.curr, self.dest) else {
            return None;
        };

        self.curr = next;

        Some(next)
    }
}

/// An iterator that returns neighboring nodes that are shortest paths to the destination node.
#[derive(Debug)]
pub struct NeighborsToIter<'a, NodeId: U16orU32> {
    graph: &'a SeqGraph<NodeId>,
    curr: NodeId,
    dest: NodeId,
    neighbors: std::slice::Iter<'a, NodeId>,
}

impl<NodeId: U16orU32> Iterator for NeighborsToIter<'_, NodeId> {
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

/// A builder for creating a [SeqGraph].
#[derive(Debug, Clone)]
pub struct SeqGraphBuilder<NodeId: U16orU32> {
    /// key: node_id
    ///
    /// value: neighbors of node
    pub nodes: Nodes<NodeId>,

    /// key: edge_id
    ///
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    pub edges: Edges<NodeId>,

    /// key: edge_id
    ///
    /// value: for each edge, bit is set to 1 if the node is computed
    pub edge_masks: Edges<NodeId>,
}

impl<NodeId: U16orU32> SeqGraphBuilder<NodeId> {
    /// Create a new SeqGraphBuilder with the given number of nodes.
    #[inline]
    pub fn new(nodes_len: usize) -> Self {
        Self {
            nodes: Nodes::new(nodes_len),
            edges: Edges::new(),
            edge_masks: Edges::new(),
        }
    }

    /// Resize the graph to the given number of nodes.
    ///
    /// All edges that are connected to nodes that are removed will also be removed.
    pub fn resize(&mut self, nodes_len: usize) {
        let should_truncate = nodes_len < self.nodes.len();

        self.nodes.resize(nodes_len);

        if should_truncate {
            self.edges.truncate(nodes_len);
            self.edge_masks.truncate(nodes_len);
        }
    }

    /// Add a edge between node_a and node_b
    #[inline]
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
            let edge = BitVec::one(val.as_usize());
            self.edges.inner.insert(ab, edge);
        }

        if let Some(edge) = self.edge_masks.inner.get_mut(&ab) {
            edge.set_bit(a.as_usize(), true);
            edge.set_bit(b.as_usize(), true);
        } else {
            let mut edge = BitVec::one(a.max(b).as_usize());
            edge.set_bit(a.min(b).as_usize(), true);
            self.edge_masks.inner.insert(ab, edge);
        }
    }

    #[inline]
    pub fn disconnect(&mut self, a: NodeId, b: NodeId) {
        // if the edge doesn't exist, return
        self.nodes.disconnect(a, b);

        let ab = edge_id(a, b);

        if self.edge_masks.inner.remove(&ab).is_some() {
            self.edges.inner.remove(&ab);
        }
    }

    #[inline]
    pub fn build(self) -> SeqGraph<NodeId> {
        let Self {
            nodes,
            mut edges,
            mut edge_masks,
            ..
        } = self;

        // (neighbors at current depth, neighbors at previous depths)
        let mut neighbors_at_depth: Vec<(BitVec, BitVec)> = nodes
            .inner
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let mut neighbors = BitVec::ZERO;
                for n in e {
                    neighbors.set_bit(n.as_usize(), true);
                }
                (neighbors, BitVec::one(i))
            })
            .collect();

        let mut active_neighbors_mask = BitVec::ZERO;

        // each rooom's bit is set to 1 if all its edges are done computed
        let mut done_nodes = BitVec::ZERO;

        let full_mask = BitVec::ones(nodes.len());

        let mut neighbor_upserts: Vec<(BitVec, BitVec, BitVec)> = Vec::new();

        for (a, a_neighbors) in nodes.inner.iter().enumerate() {
            // setup
            // clear upserts
            neighbor_upserts.iter_mut().for_each(|(e1, e2, e3)| {
                e1.clear();
                e2.clear();
                e3.clear();
            });
            if neighbor_upserts.len() < a_neighbors.len() {
                neighbor_upserts.resize(
                    a_neighbors.len(),
                    (BitVec::ZERO, BitVec::ZERO, BitVec::ZERO),
                );
            }

            // for each edge in this node
            // set the bit value for a and b as 1
            for (i, b) in a_neighbors.iter().enumerate() {
                let b = b.as_usize();

                let mut val = true;

                // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
                // - gets further away from b
                // - shortest path to a
                // - gets further away from all other nodes
                if a > b {
                    val = false;
                }

                // for all other edges in this node, set the value for this node bit as 0
                for (j, c) in a_neighbors.iter().enumerate() {
                    if i == j {
                        continue;
                    }

                    // if both b and c are in the same corner (tl or br)
                    // flip the bit
                    let should_set = if (a > b) == (a > c.as_usize()) {
                        !val
                    } else {
                        val
                    };

                    let (upsert, computed, _) = &mut neighbor_upserts[j];
                    if should_set {
                        upsert.set_bit(b, true);
                    }
                    computed.set_bit(b, true);
                }
            }

            let a = NodeId::from_usize(a);

            // apply computed values
            for (b, upserts) in a_neighbors.into_iter().zip(neighbor_upserts.drain(..)) {
                let ab = edge_id(a, *b);

                let (upsert, computed, _) = upserts;

                if !computed.is_zero() {
                    if !upsert.is_zero() {
                        edges.insert(ab, upsert);
                    }
                    edge_masks.insert(ab, computed);
                }
            }
        }

        let mut set_done_list = Vec::new();

        loop {
            // iterate through all undone nodes
            for a in done_nodes.iter_zeros() {
                if a >= nodes.len() {
                    break;
                }

                let a_usize = a;
                let a = NodeId::from_usize(a);

                let a_neighbors = nodes.neighbors(a);

                // clear upserts
                neighbor_upserts.iter_mut().for_each(|(e1, e2, e3)| {
                    e1.clear();
                    e2.clear();
                    e3.clear();
                });
                if neighbor_upserts.len() < a_neighbors.len() {
                    neighbor_upserts.resize(
                        a_neighbors.len(),
                        (BitVec::ZERO, BitVec::ZERO, BitVec::ZERO),
                    );
                }

                // collect all nodes that need to update their neighbors to next depth
                let mut a_active_neighbors_mask = BitVec::ZERO;

                // are all edges computed for this node?
                let mut all_edges_done = true;

                // get all neighbors' masks
                // so we can just reuse it
                for (i, b) in a_neighbors.iter().enumerate() {
                    let mask = edge_masks.get(edge_id(a, *b)).unwrap();
                    neighbor_upserts[i].2 = mask.clone();

                    if !mask.eq(&full_mask) {
                        all_edges_done = false;
                    }
                }

                if all_edges_done {
                    set_done_list.push(a);

                    continue;
                }

                for (i, b) in a_neighbors.iter().copied().enumerate() {
                    let b_usize = b.as_usize();

                    // neighbors' bits to gossip from edge a->b to other edges
                    let mut neighbors_mask = neighbors_at_depth[b_usize].0.clone();

                    neighbors_mask.set_bit(a_usize, false);

                    // if no neighbors to gossip at this depth, skip
                    if neighbors_mask.is_zero() {
                        continue;
                    }

                    a_active_neighbors_mask.set_bit(b_usize, true);

                    let ab = edge_id(a, b);

                    let val = edges.get(ab).unwrap();

                    // gossip to other edges about its neighbors at current depth
                    for (j, c) in a_neighbors.iter().copied().enumerate() {
                        // skip if same neighbor
                        if i == j {
                            continue;
                        }

                        let mask_ac = &neighbor_upserts[j].2;
                        if mask_ac.eq(&full_mask) {
                            continue;
                        }
                        all_edges_done = false;

                        let mut compute_mask = neighbors_mask.clone();
                        // dont set bits that are already computed
                        compute_mask.bitand_not_assign(&mask_ac);

                        // if all bits are already computed, skip
                        if compute_mask.is_zero() {
                            continue;
                        }

                        let (upsert, computed, _) = &mut neighbor_upserts[j];

                        // if both b and c are in the same corner (tl or br)
                        // flip the bit
                        if (a_usize > b_usize) == (a_usize > c.as_usize()) {
                            upsert.bitor_not_and_assign(val, &compute_mask);
                        } else {
                            upsert.bitor_and_assign(val, &compute_mask);
                        };

                        computed.bitor_assign(&compute_mask);
                    }
                }

                // if all edges are computed or none of a's neighbors are active,
                // then a is done
                if all_edges_done || a_active_neighbors_mask.is_zero() {
                    set_done_list.push(a);
                } else {
                    for (b, upserts) in a_neighbors.iter().copied().zip(neighbor_upserts.drain(..))
                    {
                        let ab = edge_id(a, b);

                        let (upsert, computed, _) = upserts;

                        if !computed.is_zero() {
                            if !upsert.is_zero() {
                                edges.insert(ab, upsert);
                            }
                            edge_masks.insert(ab, computed);
                        }
                    }
                }

                active_neighbors_mask.bitor_assign(&a_active_neighbors_mask);
            }

            for a in &set_done_list {
                done_nodes.set_bit(a.as_usize(), true);
            }
            set_done_list.clear();

            if done_nodes.eq(&full_mask) {
                break;
            }

            for a in active_neighbors_mask.iter_ones() {
                let (a_neighbors_at_depth, prev_neighbors) = &mut neighbors_at_depth[a];

                if a_neighbors_at_depth.is_zero() {
                    continue;
                }

                // add previous neighbors to prev neighbors
                prev_neighbors.bitor_assign(&a_neighbors_at_depth);

                let mut new_neighbors = BitVec::ZERO;
                for b in a_neighbors_at_depth.iter_ones() {
                    for c in nodes.neighbors(NodeId::from_usize(b)) {
                        new_neighbors.set_bit(c.as_usize(), true);
                    }
                }

                // new neighbors at this depth without the previous neighbors
                new_neighbors.bitand_not_assign(&prev_neighbors);
                *a_neighbors_at_depth = new_neighbors;
            }

            active_neighbors_mask.clear();
        }

        SeqGraph {
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

    #[inline]
    pub fn resize(&mut self, nodes_len: usize) {
        let prev_len = self.inner.len();
        self.inner.resize(nodes_len, vec![]);

        if nodes_len < prev_len {
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
    #[inline]
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
    #[inline]
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
#[derive(Debug, Clone)]
pub struct Edges<NodeId: U16orU32> {
    /// key: edge_id
    ///
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    inner: HashMap<(NodeId, NodeId), BitVec>,
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
    pub fn get(&self, edge_id: (NodeId, NodeId)) -> Option<&BitVec> {
        self.inner.get(&edge_id)
    }

    /// Insert a new edge with its shortest paths.
    ///
    /// If the edge already exists, the shortest paths will be merged.
    #[inline]
    pub fn insert(&mut self, edge_id: (NodeId, NodeId), val: BitVec) {
        if let Some(bits) = self.inner.get_mut(&edge_id) {
            bits.bitor_assign(&val);
        } else {
            self.inner.insert(edge_id, val);
        }
    }

    /// Truncate the edges to the given length of nodes.
    pub fn truncate(&mut self, nodes_len: usize) {
        let keys_to_remove = self
            .inner
            .keys()
            .filter(|&(a, b)| a.as_usize() >= nodes_len || b.as_usize() >= nodes_len)
            .cloned()
            .collect::<Vec<_>>();

        for key in keys_to_remove {
            self.inner.remove(&key);
        }

        for edge in self.inner.values_mut() {
            edge.truncate(nodes_len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_seq_graph() {
        pub const NODES_X_LEN: usize = 100;
        pub const NODES_Y_LEN: usize = 200;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = SeqGraphBuilder::new(NODES_LEN);

        // place a edge between every adjacent node
        for y in 0..NODES_Y_LEN {
            for x in 0..NODES_X_LEN {
                let node_id = y * NODES_X_LEN + x;

                if x > 0 {
                    let a = (node_id - 1) as u16;
                    let b = node_id as u16;
                    builder.connect(a, b);
                }

                if y > 0 {
                    let a = node_id as u16;
                    let b = (node_id - NODES_X_LEN) as u16;
                    builder.connect(a, b);
                }
            }
        }

        let now = std::time::Instant::now();
        let _map = builder.build();
        println!("Time: {:?}", now.elapsed());

        // std::thread::sleep(std::time::Duration::from_secs(20));
        // drop(map);
    }
}
