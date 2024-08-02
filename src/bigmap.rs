use crate::{bitvec::BitVec, edge_id};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct BigMap {
    pub nodes: Nodes,
    pub edges: HashMap<(u16, u16), BitVec>,
}

impl BigMap {
    pub fn builder(nodes_len: usize) -> BigMapBuilder {
        BigMapBuilder::new(nodes_len)
    }

    pub fn into_builder(self) -> BigMapBuilder {
        BigMapBuilder {
            edge_masks: Edges {
                inner: self.edges.iter().map(|(k, _)| (*k, BitVec::ZERO)).collect(),
            },
            edges: Edges { inner: self.edges },
            nodes: self.nodes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BigMapBuilder {
    pub nodes: Nodes,

    /// key: edge_id
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    pub edges: Edges,

    /// key: edge_id
    /// value: for each edge, bit is set to 1 if the node is computed
    pub edge_masks: Edges,
}

impl BigMapBuilder {
    pub fn new(nodes_len: usize) -> Self {
        Self {
            nodes: Nodes::new(nodes_len),
            edges: Edges::new(),
            edge_masks: Edges::new(),
        }
    }

    /// Add a edge between node_a and node_b
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
            let edge = BitVec::one(val as usize);
            self.edges.inner.insert(ab, edge);
        }

        if let Some(edge) = self.edge_masks.inner.get_mut(&ab) {
            edge.set_bit(a as usize, true);
            edge.set_bit(b as usize, true);
        } else {
            let mut edge = BitVec::one(a.max(b) as usize);
            edge.set_bit(a.min(b) as usize, true);
            self.edge_masks.inner.insert(ab, edge);
        }
    }

    pub fn disconnect(&mut self, a: u16, b: u16) {
        // if the edge doesn't exist, return
        self.nodes.disconnect(a, b);

        let ab = edge_id(a, b);

        if self.edge_masks.inner.remove(&ab).is_some() {
            self.edges.inner.remove(&ab);
        }
    }

    pub fn build(self) -> BigMap {
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
            .cloned()
            .enumerate()
            .map(|(i, e)| (e, BitVec::one(i)))
            .collect();

        let mut active_neighbors_mask = BitVec::ZERO;

        // each rooom's bit is set to 1 if all its edges are done computed
        let mut done_nodes = BitVec::ZERO;

        let full_mask = BitVec::ones(nodes.len());

        let mut neighbor_upserts: Vec<(BitVec, BitVec, BitVec)> = Vec::new();

        for (a, a_neighbors) in nodes.inner.iter().enumerate() {
            // setup
            let a_neighbors = a_neighbors.iter_ones().collect::<Vec<_>>();

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
            for (i, b) in a_neighbors.iter().cloned().enumerate() {
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

                    let (upsert, computed, _) = &mut neighbor_upserts[j];
                    if should_set {
                        upsert.set_bit(b, true);
                    }
                    computed.set_bit(b, true);
                }
            }

            // apply computed values
            for (b, upserts) in a_neighbors.into_iter().zip(neighbor_upserts.drain(..)) {
                let ab = edge_id(a as u16, b as u16);

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

                let a_neighbors = nodes.neighbors(a as u16).iter_ones().collect::<Vec<_>>();

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
                for (i, b) in a_neighbors.iter().copied().enumerate() {
                    let mask = edge_masks.get(edge_id(a as u16, b as u16)).unwrap();
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
                    // neighbors' bits to gossip from edge a->b to other edges
                    let mut neighbors_mask = neighbors_at_depth[b].0.clone();

                    neighbors_mask.set_bit(a, false);

                    // if no neighbors to gossip at this depth, skip
                    if neighbors_mask.is_zero() {
                        continue;
                    }

                    a_active_neighbors_mask.set_bit(b, true);

                    let ab = edge_id(a as u16, b as u16);

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
                        if (a > b) == (a > c) {
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
                        let ab = edge_id(a as u16, b as u16);

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
                done_nodes.set_bit(*a, true);
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
                    new_neighbors.bitor_assign(nodes.neighbors(b as u16));
                }

                // new neighbors at this depth without the previous neighbors
                new_neighbors.bitand_not_assign(&prev_neighbors);
                *a_neighbors_at_depth = new_neighbors;
            }

            active_neighbors_mask.clear();
        }

        BigMap {
            nodes,
            edges: edges.inner,
        }
    }
}

/// index: the node_index
/// value: the bit places of connected nodes are 1
#[derive(Debug, Clone)]
pub struct Nodes {
    pub inner: Vec<BitVec>,
}

impl Nodes {
    pub fn new(nodes_len: usize) -> Self {
        Self {
            inner: vec![BitVec::ZERO; nodes_len],
        }
    }

    /// Get the neighboring nodes
    #[inline]
    pub fn neighbors(&self, node: u16) -> &BitVec {
        &self.inner[node as usize]
    }

    /// Add a edge between node_a and node_b
    pub fn connect(&mut self, a: u16, b: u16) {
        if a == b {
            return;
        }

        self.inner[a as usize].set_bit(b as usize, true);
        self.inner[b as usize].set_bit(a as usize, true);
    }

    /// Remove a edge between node_a and node_b
    pub fn disconnect(&mut self, a: u16, b: u16) {
        if a == b {
            return;
        }

        self.inner[a as usize].set_bit(b as usize, false);
        self.inner[b as usize].set_bit(a as usize, false);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug, Clone)]
pub struct Edges {
    /// key: edge_id
    /// value: for each bit, if this edge is the shortest path
    /// to that bit location's node, bit is set to 1
    inner: HashMap<(u16, u16), BitVec>,
}

impl Edges {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, edge_id: (u16, u16)) -> Option<&BitVec> {
        self.inner.get(&edge_id)
    }

    #[inline]
    pub fn insert(&mut self, edge_id: (u16, u16), val: BitVec) {
        if let Some(bits) = self.inner.get_mut(&edge_id) {
            bits.bitor_assign(&val);
        } else {
            self.inner.insert(edge_id, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_map() {
        pub const NODES_X_LEN: usize = 32;
        pub const NODES_Y_LEN: usize = 32;
        pub const NODES_LEN: usize = NODES_X_LEN * NODES_Y_LEN;

        let mut builder = BigMapBuilder::new(NODES_LEN);

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
