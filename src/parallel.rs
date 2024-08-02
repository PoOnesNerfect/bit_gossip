use crate::{
    bitvec::{AtomicBitVec, BitVec},
    edge_id,
};
use rayon::prelude::*;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug)]
pub struct ParaMap {
    pub nodes: Nodes,
    pub edges: HashMap<(u16, u16), AtomicBitVec>,
}

impl ParaMap {
    /// Create a new ParaMapBuilder with the given number of nodes.
    pub fn builder(nodes_len: usize) -> ParaMapBuilder {
        ParaMapBuilder::new(nodes_len)
    }

    /// Convert this ParaMap into a ParaMapBuilder.
    pub fn into_builder(self) -> ParaMapBuilder {
        ParaMapBuilder {
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
    /// return the neighboring node of current that is the shortest path to the destination node.
    ///
    /// In case there are multiple neighboring nodes that lead to the shortest path,
    /// the first one found will be returned. The same node will be returned for the same input.
    /// However, the order of the nodes is not guaranteed.
    pub fn next_node(&self, curr: u16, dest: u16) -> Option<u16> {
        if curr == dest {
            return None;
        }

        self.nodes
            .neighbors(curr)
            .iter()
            .copied()
            .find(|&neighbor| self.edges[&edge_id(curr, neighbor)].get_bit(dest as usize))
    }

    /// Given a current node and a destination node,
    /// return all neighboring nodes of current that are shortest paths to the destination node.
    ///
    /// The nodes will be returned in the same order for the same inputs. However, the ordering of the nodes is not guaranteed.
    pub fn next_nodes(&self, curr: u16, dest: u16) -> Vec<u16> {
        if curr == dest {
            return vec![];
        }

        self.nodes
            .neighbors(curr)
            .iter()
            .copied()
            .filter(|&neighbor| self.edges[&edge_id(curr, neighbor)].get_bit(dest as usize))
            .collect()
    }

    /// Given a current node and a destination node,
    /// return a path from the current node to the destination node.
    /// The path is a list of node IDs, starting from the current node and ending at the destination node.
    pub fn path_to(&self, curr: u16, dest: u16) -> PathIter {
        PathIter {
            map: self,
            curr,
            dest,
            done: false,
        }
    }

    /// Return a list of all neighboring nodes of the given node.
    pub fn neighbors(&self, node: u16) -> &[u16] {
        self.nodes.neighbors(node)
    }
}

#[derive(Debug)]
pub struct PathIter<'a> {
    map: &'a ParaMap,
    curr: u16,
    dest: u16,
    done: bool,
}

impl Iterator for PathIter<'_> {
    type Item = u16;

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

#[derive(Debug)]
pub struct ParaMapBuilder {
    /// key: node_id
    /// value: neighbors of node
    pub nodes: Nodes,

    /// key: edge_id
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    pub edges: Edges,

    /// key: edge_id
    /// value: for each edge, bit is set to 1 if the node with the bit location is computed for this edge
    pub edge_masks: Edges,
}

impl ParaMapBuilder {
    /// Create a new ParaMapBuilder with the given number of nodes.
    pub fn new(nodes_len: usize) -> Self {
        Self {
            nodes: Nodes::new(nodes_len),
            edges: Edges::new(),
            edge_masks: Edges::new(),
        }
    }

    /// Add an edge between node_a and node_b
    pub fn connect(&mut self, a: u16, b: u16) {
        self.nodes.connect(a, b);

        // edge value is flipped to b -> a, which means from node b's perspective, this edge is:
        // - gets further away from b
        // - shortest path to a
        // - gets further away from all other nodes
        let val = if a > b { a } else { b };

        let ab = edge_id(a, b);

        if let Some(edge) = self.edges.inner.get_mut(&ab) {
            edge.set_bit(val as usize, true);
        } else {
            let edge = AtomicBitVec::one(val as usize, self.nodes.len());

            self.edges.inner.insert(ab, edge);
        }

        if let Some(edge) = self.edge_masks.inner.get_mut(&ab) {
            edge.set_bit(a as usize, true);
            edge.set_bit(b as usize, true);
        } else {
            let edge = AtomicBitVec::zeros(self.nodes.len());
            edge.set_bit(a as usize, true);
            edge.set_bit(b as usize, true);

            self.edge_masks.inner.insert(ab, edge);
        }
    }

    /// Remove an edge between node_a and node
    pub fn disconnect(&mut self, a: u16, b: u16) {
        // if the edge doesn't exist, return
        self.nodes.disconnect(a, b);

        let ab = edge_id(a, b);

        if self.edge_masks.inner.remove(&ab).is_some() {
            self.edges.inner.remove(&ab);
        }
    }

    /// Build the ParaMap from the current state of the builder.
    pub fn build(self) -> ParaMap {
        let Self {
            nodes,
            edges,
            edge_masks,
            ..
        } = self;

        let chunk_size = 16;

        // (neighbors at current depth, neighbors at previous depths)
        let neighbors_at_depth: Vec<(AtomicBitVec, AtomicBitVec)> = nodes
            .inner
            .par_iter()
            .enumerate()
            .map(|(i, e)| {
                let neighbors = AtomicBitVec::zeros(nodes.len());
                for n in e {
                    neighbors.set_bit(*n as usize, true);
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

                    // for each edge in this node
                    // set the bit value for a and b as 1
                    for (i, b) in a_neighbors.iter().cloned().enumerate() {
                        let b = b as usize;

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
                            let c = c as usize;

                            // if both b and c are in the same corner (tl or br)
                            // flip the bit
                            let should_set = if (a > b) == (a > c) { !val } else { val };

                            let (upsert, computed) = &mut neighbor_upserts[j];
                            if should_set {
                                upsert.set_bit(b, true);
                            }
                            computed.set_bit(b, true);
                        }
                    }

                    // apply computed values
                    for (b, upserts) in a_neighbors.iter().zip(neighbor_upserts.drain(..)) {
                        let ab = edge_id(a as u16, *b);

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

                        let a_neighbors = nodes.neighbors(a as u16);

                        let mut neighbor_upserts: Vec<(BitVec, BitVec)> =
                            vec![(BitVec::ZERO, BitVec::ZERO); a_neighbors.len()];

                        // collect all nodes that need to update their neighbors to next depth
                        let mut a_active_neighbors_mask = BitVec::ZERO;

                        // are all edges computed for this node?
                        let mut all_edges_done = true;

                        // get all neighbors' masks
                        // so we can just reuse it

                        let mut neighbor_masks = Vec::with_capacity(a_neighbors.len());

                        for b in a_neighbors.iter().copied() {
                            let mask = edge_masks.get(edge_id(a as u16, b as u16)).unwrap();
                            neighbor_masks.push(mask);

                            if !mask.eq(&full_mask) {
                                all_edges_done = false;
                            }
                        }

                        if all_edges_done {
                            done_nodes.set_bit(a, true);

                            continue;
                        }

                        for (i, b) in a_neighbors.iter().copied().enumerate() {
                            let b = b as usize;

                            // neighbors' bits to gossip from edge a->b to other edges
                            let mut neighbors_mask = neighbors_at_depth[b].0.into_bitvec();

                            neighbors_mask.set_bit(a, false);

                            // if no neighbors to gossip at this depth, skip
                            if neighbors_mask.is_zero() {
                                continue;
                            }

                            a_active_neighbors_mask.set_bit(b, true);

                            let ab = edge_id(a as u16, b as u16);

                            let val = edges.get(ab).unwrap().into_bitvec();

                            // gossip to other edges about its neighbors at current depth
                            for (j, c) in a_neighbors.iter().copied().enumerate() {
                                // skip if same neighbor
                                if i == j {
                                    continue;
                                }
                                let c = c as usize;

                                let mask_ac = neighbor_masks[j];
                                if mask_ac.eq(&full_mask) {
                                    continue;
                                }
                                all_edges_done = false;

                                let mut compute_mask = neighbors_mask.clone();
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
                        if all_edges_done || a_active_neighbors_mask.is_zero() {
                            done_nodes.set_bit(a, true);
                        } else {
                            for (b, upserts) in
                                a_neighbors.iter().copied().zip(neighbor_upserts.drain(..))
                            {
                                let ab = edge_id(a as u16, b as u16);

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
                            for c in nodes.neighbors(b as u16) {
                                new_neighbors.set_bit(*c as usize, true);
                            }
                        }

                        // new neighbors at this depth without the previous neighbors
                        new_neighbors.bitand_not_assign_atomic(prev_neighbors);
                        a_neighbors_at_depth.assign_from(&new_neighbors);
                    }
                });

            active_neighbors_mask.clear();
        }

        ParaMap {
            nodes,
            edges: edges.inner,
        }
    }
}

/// index: the node_id
/// value: neighbors of node
#[derive(Debug, Clone)]
pub struct Nodes {
    pub inner: Vec<Vec<u16>>,
}

impl Nodes {
    pub fn new(nodes_len: usize) -> Self {
        Self {
            inner: vec![vec![]; nodes_len],
        }
    }

    pub fn resize(&mut self, nodes_len: usize) {
        self.inner.resize(nodes_len, vec![]);

        if nodes_len < self.inner.len() {
            let nodes_len = nodes_len as u16;

            for neighbors in self.inner.iter_mut() {
                neighbors.retain(|&i| i < nodes_len);
            }
        }
    }

    /// Get the neighboring nodes
    #[inline]
    pub fn neighbors(&self, node: u16) -> &[u16] {
        &self.inner[node as usize]
    }

    /// Add a edge between node_a and node_b
    pub fn connect(&mut self, a: u16, b: u16) {
        if a == b {
            return;
        }

        if !self.inner[a as usize].contains(&b) {
            self.inner[a as usize].push(b);
        }

        self.inner[b as usize].push(a);
    }

    /// Remove a edge between node_a and node_b
    pub fn disconnect(&mut self, a: u16, b: u16) {
        if a == b {
            return;
        }

        if let Some(index) = self.inner[a as usize].iter().position(|&x| x == b) {
            self.inner[a as usize].swap_remove(index);
        }
        if let Some(index) = self.inner[b as usize].iter().position(|&x| x == a) {
            self.inner[b as usize].swap_remove(index);
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug)]
pub struct Edges {
    /// key: edge_id
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    inner: HashMap<(u16, u16), AtomicBitVec>,
}

impl Edges {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, edge_id: (u16, u16)) -> Option<&AtomicBitVec> {
        self.inner.get(&edge_id)
    }

    #[inline]
    pub fn insert(&mut self, edge_id: (u16, u16), val: BitVec, nodes_len: usize) {
        if let Some(bits) = self.inner.get_mut(&edge_id) {
            bits.bitor_assign(&val);
        } else {
            self.inner
                .insert(edge_id, AtomicBitVec::from_bitvec(&val, nodes_len));
        }
    }

    #[inline]
    pub fn update(&self, edge_id: (u16, u16), val: BitVec) {
        if let Some(bits) = self.inner.get(&edge_id) {
            bits.bitor_assign(&val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_para_map() {
        pub const NODES_X_LEN: usize = 80;
        pub const NODES_Y_LEN: usize = 80;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let now = std::time::Instant::now();

        let mut builder = ParaMapBuilder::new(NODES_LEN);

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

        println!("Setup Time: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        let _map = builder.build();
        println!("Build Time: {:?}", now.elapsed());

        // std::thread::sleep(std::time::Duration::from_secs(20));
        // drop(map);
    }
}
