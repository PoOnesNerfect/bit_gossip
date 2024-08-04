//! general-use graph data structure and its builder.
//!
//! [Graph] is an enum of two variants `SeqGraph` and `ParaGraph`,
//! and can be built using [GraphBuilder::build].
//!
//! When building with [GraphBuilder], it will automatically choose
//! the best implementation based on the number of threads available.
//!
//! You can also manually choose the implementation by calling the [GraphBuilder::multi_threaded] method.
//!
//! If you also want, you can use either [ParaGraph](parallel::ParaGraph) or [SeqGraph](sequential::SeqGraph) directly.
//!
//! # Examples
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
//! use bit_gossip::Graph;
//!
//! // Initialize a builder with 12 nodes
//! let mut builder = Graph::builder(12);
//!
//! // Connect the nodes
//! for i in 0..12u16 {
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
//! ## Large Graphs
//!
//! In this example, let's create a 100x100 grid graph.
//!
//! ```rust
//! use bit_gossip::Graph;
//!
//! // Initialize a builder with 10000 nodes
//! let mut builder = Graph::builder(10000);
//!
//! // Connect the nodes
//! for y in 0..100u16 {
//!     for x in 0..100 {
//!         let node = y * 100 + x;
//!
//!         if x < 99 {
//!             builder.connect(node, node + 1);
//!         }
//!         if y < 99 {
//!             builder.connect(node, node + 100);
//!         }
//!     }
//! }
//!
//! // Build the graph
//! // This may take a few seconds
//! let graph = builder.build();
//!
//! // Check the shortest path from 0 to 9900
//! // This is fast
//! let mut curr = 0;
//! let dest = 9900;
//!
//! let mut count = 0;
//!
//! while curr != dest {
//!     let prev = curr;
//!     curr = graph.next_node(curr, dest).unwrap();
//!     println!("{prev} -> {curr}");
//!
//!     count += 1;
//!     if curr == dest {
//!         println!("we've reached node '{dest}' in {count} hops!");
//!         break;
//!     }
//! }
//! ```

#[cfg(feature = "parallel")]
pub mod parallel;
pub mod sequential;

/// Unweighted Undirected graph that can be used to find shortest paths between nodes.
///
/// All shortest paths between all nodes are already precomputed.
///
/// This graph is read-only.
///
/// If you want to resize the graph, or add/remove edges, you can
/// convert it into a builder by calling `.into_builder()`.`
///
/// To see a basic use case examples, check the [graph](crate::graph) module documentation.
#[derive(Debug)]
pub enum Graph<NodeId: U16orU32 = u16> {
    Sequential(sequential::SeqGraph<NodeId>),
    #[cfg(feature = "parallel")]
    Parallel(parallel::ParaGraph<NodeId>),
}

impl<NodeId: U16orU32> Graph<NodeId> {
    /// Create a new GraphBuilder with the given number of nodes.
    ///
    /// Panics if the number of nodes exceeds the limit of the NodeId type.
    ///
    /// Default NodeId is u16, which can hold up to 65536 nodes.
    /// If you need more nodes, you can specify u32 as the NodeId type, like `Graph::<u32>::builder(100_000)`
    #[inline]
    pub fn builder(nodes_len: usize) -> GraphBuilder<NodeId> {
        assert!(
            nodes_len <= NodeId::MAX_NODES,
            "Number of nodes exceeds the limit; Specify `u32` as the NodeId type, like `Graph::<u32>::builder(100_000)`"
        );

        GraphBuilder::new(nodes_len)
    }

    /// Converts this graph into a builder.
    pub fn into_builder(self) -> GraphBuilder<NodeId> {
        let nodes_len = match &self {
            Graph::Sequential(ref builder) => builder.nodes_len(),
            #[cfg(feature = "parallel")]
            Graph::Parallel(ref builder) => builder.nodes_len(),
        };

        let inner = match self {
            Graph::Sequential(graph) => GraphBuilderEnum::Sequential(graph.into_builder()),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => GraphBuilderEnum::Parallel(graph.into_builder()),
        };

        let multi_threaded = match inner {
            GraphBuilderEnum::Sequential(_) => Some(false),
            #[cfg(feature = "parallel")]
            GraphBuilderEnum::Parallel(_) => Some(true),
            GraphBuilderEnum::None => unreachable!(),
        };

        GraphBuilder {
            inner,
            multi_threaded,
            nodes_len,
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
        match self {
            Graph::Sequential(graph) => NextNodesIter::Sequential(graph.next_nodes(curr, dest)),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => NextNodesIter::Parallel(graph.next_nodes(curr, dest)),
        }
    }

    /// Given a current node and a destination node,
    /// return a path from the current node to the destination node.
    ///
    /// The path is a list of node IDs, starting with the next node (not current node!) and ending at the destination node.
    #[inline]
    pub fn path_to(&self, curr: NodeId, dest: NodeId) -> PathIter<'_, NodeId> {
        match self {
            Graph::Sequential(graph) => PathIter::Sequential(graph.path_to(curr, dest)),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => PathIter::Parallel(graph.path_to(curr, dest)),
        }
    }

    /// Check if there is a path from the current node to the destination node.
    #[inline]
    pub fn path_exists(&self, curr: NodeId, dest: NodeId) -> bool {
        match self {
            Graph::Sequential(graph) => graph.path_exists(curr, dest),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => graph.path_exists(curr, dest),
        }
    }

    /// Return a list of all neighboring nodes of the given node.
    #[inline]
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        match self {
            Graph::Sequential(graph) => graph.neighbors(node),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => graph.neighbors(node),
        }
    }

    /// Return the number of nodes in the graph.
    #[inline]
    pub fn nodes_len(&self) -> usize {
        match self {
            Graph::Sequential(graph) => graph.nodes_len(),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => graph.nodes_len(),
        }
    }

    /// Return the number of edges in the graph.
    #[inline]
    pub fn edges_len(&self) -> usize {
        match self {
            Graph::Sequential(graph) => graph.edges_len(),
            #[cfg(feature = "parallel")]
            Graph::Parallel(graph) => graph.edges_len(),
        }
    }
}

/// An iterator that returns a path from the current node to the destination node.
///
/// Current node is not included in the path.
#[derive(Debug)]
pub enum PathIter<'a, NodeId: U16orU32> {
    Sequential(sequential::PathIter<'a, NodeId>),
    #[cfg(feature = "parallel")]
    Parallel(parallel::PathIter<'a, NodeId>),
}

impl<NodeId: U16orU32> Iterator for PathIter<'_, NodeId> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PathIter::Sequential(iter) => iter.next(),
            #[cfg(feature = "parallel")]
            PathIter::Parallel(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
pub enum NextNodesIter<'a, NodeId: U16orU32> {
    Sequential(sequential::NextNodesIter<'a, NodeId>),
    #[cfg(feature = "parallel")]
    Parallel(parallel::NextNodesIter<'a, NodeId>),
}

impl<NodeId: U16orU32> Iterator for NextNodesIter<'_, NodeId> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NextNodesIter::Sequential(iter) => iter.next(),
            #[cfg(feature = "parallel")]
            NextNodesIter::Parallel(iter) => iter.next(),
        }
    }
}

/// A builder for creating a new graph and all shortest paths.
#[derive(Debug)]
pub struct GraphBuilder<NodeId: U16orU32 = u16> {
    inner: GraphBuilderEnum<NodeId>,
    multi_threaded: Option<bool>,
    nodes_len: usize,
}

#[derive(Debug)]
enum GraphBuilderEnum<NodeId: U16orU32> {
    Sequential(sequential::SeqGraphBuilder<NodeId>),
    #[cfg(feature = "parallel")]
    Parallel(parallel::ParaGraphBuilder<NodeId>),
    None,
}

impl<NodeId: U16orU32> GraphBuilderEnum<NodeId> {
    #[inline]
    fn is_none(&self) -> bool {
        matches!(self, GraphBuilderEnum::None)
    }

    #[allow(unused_variables)]
    fn set_builder(&mut self, nodes_len: usize, multi_threaded: Option<bool>) {
        #[cfg(feature = "parallel")]
        let builder = {
            let multi_threaded = multi_threaded.unwrap_or_else(|| {
                let available_parallelism = std::thread::available_parallelism()
                    .map(|e| e.get())
                    .unwrap_or(1);
                available_parallelism > 1
            });

            if multi_threaded {
                GraphBuilderEnum::Parallel(parallel::ParaGraphBuilder::new(nodes_len))
            } else {
                GraphBuilderEnum::Sequential(sequential::SeqGraphBuilder::new(nodes_len))
            }
        };

        #[cfg(not(feature = "parallel"))]
        let builder = GraphBuilderEnum::Sequential(sequential::SeqGraphBuilder::new(nodes_len));

        *self = builder;
    }
}

impl<NodeId: U16orU32> GraphBuilder<NodeId> {
    #[inline]
    pub fn new(nodes_len: usize) -> Self {
        GraphBuilder {
            inner: GraphBuilderEnum::None,
            multi_threaded: None,
            nodes_len,
        }
    }

    #[cfg(feature = "parallel")]
    #[inline]
    pub fn multi_threaded(mut self, multi_threaded: bool) -> Self {
        self.multi_threaded = Some(multi_threaded);
        self
    }

    /// Add an edge between node_a and node_b
    #[inline]
    pub fn connect(&mut self, a: NodeId, b: NodeId) {
        if self.inner.is_none() {
            self.inner.set_builder(self.nodes_len, self.multi_threaded);
        }

        match &mut self.inner {
            GraphBuilderEnum::Sequential(builder) => builder.connect(a, b),
            #[cfg(feature = "parallel")]
            GraphBuilderEnum::Parallel(builder) => builder.connect(a, b),
            GraphBuilderEnum::None => unreachable!(),
        }
    }

    /// Remove an edge between node_a and node_b
    #[inline]
    pub fn disconnect(&mut self, a: NodeId, b: NodeId) {
        if self.inner.is_none() {
            self.inner.set_builder(self.nodes_len, self.multi_threaded);
        }

        match &mut self.inner {
            GraphBuilderEnum::Sequential(builder) => builder.disconnect(a, b),
            #[cfg(feature = "parallel")]
            GraphBuilderEnum::Parallel(builder) => builder.disconnect(a, b),
            GraphBuilderEnum::None => unreachable!(),
        }
    }

    #[inline]
    pub fn build(self) -> Graph<NodeId> {
        let mut builder = self.inner;
        if builder.is_none() {
            builder.set_builder(self.nodes_len, self.multi_threaded);
        }

        match builder {
            GraphBuilderEnum::Sequential(builder) => Graph::Sequential(builder.build()),
            #[cfg(feature = "parallel")]
            GraphBuilderEnum::Parallel(builder) => Graph::Parallel(builder.build()),
            GraphBuilderEnum::None => unreachable!(),
        }
    }

    /// Return the number of nodes in this graph.
    #[inline]
    pub fn nodes_len(&self) -> usize {
        match self {
            GraphBuilder {
                inner: GraphBuilderEnum::Sequential(builder),
                ..
            } => builder.nodes_len(),
            #[cfg(feature = "parallel")]
            GraphBuilder {
                inner: GraphBuilderEnum::Parallel(builder),
                ..
            } => builder.nodes_len(),
            GraphBuilder {
                inner: GraphBuilderEnum::None,
                nodes_len,
                ..
            } => *nodes_len,
        }
    }

    /// Return the number of edges in this graph.
    #[inline]
    pub fn edges_len(&self) -> usize {
        match self {
            GraphBuilder {
                inner: GraphBuilderEnum::Sequential(builder),
                ..
            } => builder.edges_len(),
            #[cfg(feature = "parallel")]
            GraphBuilder {
                inner: GraphBuilderEnum::Parallel(builder),
                ..
            } => builder.edges_len(),
            GraphBuilder {
                inner: GraphBuilderEnum::None,
                ..
            } => 0,
        }
    }

    /// Return the neighbors of the given node.
    #[inline]
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        match self {
            GraphBuilder {
                inner: GraphBuilderEnum::Sequential(builder),
                ..
            } => builder.neighbors(node),
            #[cfg(feature = "parallel")]
            GraphBuilder {
                inner: GraphBuilderEnum::Parallel(builder),
                ..
            } => builder.neighbors(node),
            GraphBuilder {
                inner: GraphBuilderEnum::None,
                ..
            } => &[],
        }
    }
}

/// Either u16 or u32.
pub trait U16orU32: sealed::Sealed {
    /// Maximum number of nodes that can be stored
    const MAX_NODES: usize;

    /// Cast type as usize.
    /// For internal uses, we can assume this is safe.
    fn as_usize(self) -> usize;

    /// Convert usize to NodeId.
    fn from_usize(value: usize) -> Self;
}

mod sealed {
    use std::fmt;

    use super::*;

    pub trait Sealed:
        Ord + Eq + Clone + Copy + std::hash::Hash + Send + Sync + fmt::Display + fmt::Debug
    {
    }
    impl Sealed for u16 {}
    impl Sealed for u32 {}

    impl U16orU32 for u16 {
        const MAX_NODES: usize = 1 << 16;

        #[inline]
        fn as_usize(self) -> usize {
            self as usize
        }

        #[inline]
        fn from_usize(value: usize) -> Self {
            value as u16
        }
    }

    impl U16orU32 for u32 {
        const MAX_NODES: usize = 1 << 32;

        #[inline]
        fn as_usize(self) -> usize {
            self as usize
        }

        #[inline]
        fn from_usize(value: usize) -> Self {
            value as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        // Initialize a builder with 10000 nodes
        let mut builder = Graph::builder(10000);

        // Connect the nodes
        for y in 0..100u16 {
            for x in 0..100 {
                let node = y * 100 + x;

                if x < 99 {
                    builder.connect(node, node + 1);
                }
                if y < 99 {
                    builder.connect(node, node + 100);
                }
            }
        }

        // Build the graph
        // This may take a few seconds
        let graph = builder.build();

        // Check the shortest path from 0 to 9900
        // This is fast
        let mut curr = 0;
        let dest = 9900;

        let mut hops = 0;

        while curr != dest {
            let prev = curr;
            curr = graph.next_node(curr, dest).unwrap();
            println!("{prev} -> {curr}");

            hops += 1;
            if curr == dest {
                println!("we've reached node '{dest}' in {hops} hops!");
                break;
            }
        }
    }
}
