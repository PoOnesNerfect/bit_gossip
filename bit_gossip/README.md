# bit_gossip

[<img alt="github" src="https://img.shields.io/badge/github-poonesnerfect/bit_gossip-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/poonesnerfect/bit_gossip)
[<img alt="crates.io" src="https://img.shields.io/crates/v/bit_gossip.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/bit_gossip)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-bit_gossip-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/bit_gossip)

**bit_gossip**, named after its implementation technique, is a simple pathfinding library for calculating all node pairs' shortest paths in an unweighted undirected graph.

Once the computation is complete, you can retrieve the shortest path between any two nodes in near constant time; I'm talking less than a microsecond!

This library is for you if your game:

- has decent number of nodes/tiles (~1000), and
- has hundreds or thousands of entities that need to find paths to a moving target.

If you have a static map of large number of nodes (>10,000),
you can use this library to compute all paths during the loading phase.

Also, computation is fast enough to be used not only in static maps but also in dynamically changing maps in games.

- Computing all paths for 100 nodes takes only a few hundred µs on a modern CPU.
- Computing all paths for 1000 nodes takes less than 50 ms on a modern CPU.
- Computing all paths for 10000 nodes takes less a few seconds.

See [benchmarks](#benchmarks) for more details.

## Table of Contents

<details>
<summary>Click to expand</summary>

<br>

<!--toc:start-->

- [bit_gossip](#bitgossip)
  - [Table of Contents](#table-of-contents)
  - [Basic Usage](#basic-usage)
    - [Small Graphs](#small-graphs)
    - [Large Graphs](#large-graphs)
    - [Updating the Graph](#updating-the-graph)
  - [Graph Types](#graph-types)
  - [How It Works](#how-it-works)
    - [Graph Representation](#graph-representation)
    - [Building the Graph](#building-the-graph)
      - [First Iteration](#first-iteration)
      - [Second Iteration](#second-iteration)
    - [Path Retrieval](#path-retrieval)
  - [Benchmarks](#benchmarks)
    - [Processing Time](#processing-time)
    - [Memory Usage](#memory-usage)
  - [Features](#features)
  - [Examples](#examples)
    - [Astar Recording](#astar-recording)
    - [Bit Gossip Recording](#bit-gossip-recording)
  - [Optimization for Large Graphs](#optimization-for-large-graphs)
    - [Partitioning](#partitioning)
    - [Dividing the Map Better](#dividing-the-map-better)
  - [Future Plans](#future-plans)
    - [Node with A Single Neighbor](#node-with-a-single-neighbor)
    - [Delta/Flow of Information](#deltaflow-of-information)
    - [GPU/SIMD Acceleration](#gpusimd-acceleration)
    - [Finding Dangling Nodes](#finding-dangling-nodes)
    - [Moving Away from a Destination](#moving-away-from-a-destination)
  - [What Paper Did I Base This On?](#what-paper-did-i-base-this-on)

<!--toc:end-->

</details>

## Basic Usage

### Small Graphs

For small graphs with less than 128 nodes, use [Graph16], [Graph32], [Graph64], or [Graph128].

In GraphN, like `Graph16`, N denotes the number of nodes that the graph can hold.

```sh
0 -- 1 -- 2 -- 3
|         |    |
4 -- 5 -- 6 -- 7
|         |    |
8 -- 9 -- 10 - 11
```

```rust
use bit_gossip::Graph16;

// Initialize a builder with 12 nodes
let mut builder = Graph16::builder(12);

// Connect the nodes
for i in 0..12u8 {
    if i % 4 != 3 {
        builder.connect(i, i + 1);
    }
    if i < 8 {
        builder.connect(i, i + 4);
    }
}

builder.disconnect(1, 5);
builder.disconnect(5, 9);

// Build the graph
let graph = builder.build();

// Check the shortest path from 0 to 9
assert_eq!(graph.neighbor_to(0, 9), Some(4));
assert_eq!(graph.neighbor_to(4, 9), Some(8));
assert_eq!(graph.neighbor_to(8, 9), Some(9));

// Both 1 and 4 can reach 11 in the shortest path.
assert_eq!(graph.neighbors_to(0, 11).collect::<Vec<_>>(), vec![1, 4]);

// Get the path from 0 to 5
assert_eq!(graph.path_to(0, 5).collect::<Vec<_>>(), vec![0, 4, 5]);
```

### Large Graphs

For graphs with more than 128 nodes, use [Graph], which can hold arbitrary number of nodes.

If the environment allows multi-threading, `Graph` will process paths in parallel for faster computation.

In this example, let's create a 100x100 grid graph.

```rust
use bit_gossip::Graph;

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
    curr = graph.neighbor_to(curr, dest).unwrap();
    println!("{prev} -> {curr}");

    hops += 1;
    if curr == dest {
        println!("we've reached node '{dest}' in {hops} hops!");
        break;
    }
}
```

### Updating the Graph

You cannot update the graph directly, but you can convert the graph back into the builder,
update the number of nodes and add/remove edges, then build the graph again.

I have not tested the resizing of graph yet, so please open an issue if you find any bugs.

```rust
use bit_gossip::Graph;

// Initialize a builder with 10000 nodes
let mut builder = Graph::builder(10000);

... build graph

// Build the graph
// This may take a few seconds
let graph = builder.build();

... do some work

let mut builder  = graph.into_builder();

// Resize the graph to 5000 nodes
builder.resize(5000);

// add/remove edges
builder.disconnect(0, 1);
builder.connect(0, 3000);

... 

/// Build the graph again
let graph = builder.build();
```

## Graph Types

- [Graph16], [Graph32], [Graph64], [Graph128]: Uses primitive data types for bit storage and are highly efficient.
  - In `GraphN`, N denotes the number of nodes that the graph can hold.
  - Use these types for fixed node sizes less than 128 nodes.

- [Graph]: Supports arbitrary node sizes and is optimized for parallel processing.
  - Enum of `SeqGraph` and `ParaGraph`. If environment supports multi-threading, `ParaGraph` is used; otherwise, `SeqGraph` is used.

- [SeqGraph]: Uses `Vec<u32>` or `Vec<u64>` to store bits depending on the target architecture,
  and generally about 3 times slower than primitive data types.
- [ParaGraph]: Uses `Vec<AtomicU32>` or `Vec<AtomicU64>` for bit storage, which is slower but benefits from parallel processing, making it more efficient as the number of nodes increases.

For fixed node sizes less than 128 nodes, prefer primitive-data-type-backed graph types.

The library exposes a generic `Graph` type that automatically selects the parallel or sequential version based on the environment, with manual selection also available.

## How It Works

### Graph Representation

<details>
  <summary>Click to expand</summary>

Each edge in the graph stores N bits of information, where N is the number of nodes in the graph.

In the edge `a->b`, `n`'th bit represents the presence of a shortest path from the node `a` to the node `n` in this edge.

Therefore, the graph can store all shortest paths between all pairs of nodes in the graph in NxM bits, where M is the number of edges in the graph.

Once the graph is built, you can retrieve the shortest path between any two nodes in near constant time, just by checking the bits in the edges.

</details>

### Building the Graph

<details>
  <summary>Click to expand</summary>

We will follow through a simple graph:

```
0 -- 1 -- 2
|    |
3 -- 4
|
5
```

At the start, all the bits in the edges are unset. Every edge will have bits `[ ][ ][ ][ ][ ][ ]` like so.

```
[ ][ ][ ][ ][ ][ ] // 0 -> 1
[ ][ ][ ][ ][ ][ ] // 0 -> 3
[ ][ ][ ][ ][ ][ ] // 1 -> 2
[ ][ ][ ][ ][ ][ ] // 1 -> 4
[ ][ ][ ][ ][ ][ ] // 3 -> 4
[ ][ ][ ][ ][ ][ ] // 3 -> 5
```

#### First Iteration

Let's start with node `0`.

For the edge `0->1`, we set the bits to `[ ][ ][ ][ ][1][ ]` since this edge is part of the shortest path to `1`.

```
[ ][ ][ ][ ][1][ ] // 0 -> 1
```

For `0->3`, we set the bits to `[ ][ ][1][ ][ ][ ]` since this edge is part of the shortest path to `3`.

```
[ ][ ][1][ ][ ][ ] // 0 -> 3
```

Next, we move to node `1`.

For the edge `1->0`, we can simply flip the bits of `0->1`.

```
[ ][ ][ ][ ][1][ ] // 0 -> 1

flips to

[ ][ ][ ][ ][0][ ] // 1 -> 0
```

We set the bit 0 to 1.

```
[ ][ ][ ][ ][0][1] // 1 -> 0

flips to

[ ][ ][ ][ ][1][0] // 0 -> 1
```

From node `1`'s perspective, the edge `1->0` is the shortest path to `0`, and gets further away from `1`. Makes sense, right?

Now, to `1->2`, we set the bit 2 to `1`, as this edge is the shortest path to node `2`.

```
[ ][ ][ ][1][ ][ ] // 1 -> 2
```

Now, we move to node `2`.

For `2->1`, again, flipping `1->2`, we set the bit 1 to `1`.

```
[ ][ ][ ][0][1][ ] // 2 -> 1
```

We repeat this process for rest of the nodes, which we end up with the following bits in edges, in matrix form:

```
[ ][ ][ ][ ][1][0] // 0 -> 1
[ ][ ][1][ ][ ][0] // 0 -> 3
[ ][ ][ ][1][0][ ] // 1 -> 2
[ ][1][ ][ ][0][ ] // 1 -> 4
[ ][1][0][ ][ ][ ] // 3 -> 4
[1][ ][0][ ][ ][ ] // 3 -> 5
```

Do you see a pattern here? Every edge has 2 bits set.

#### Second Iteration

Now, each edge will start share its bits with neighboring edges; this is why the algorithm is called `bit_gossip`.

**Note**: we share the bit only if the neighboring edge does not have the bit already set.

Let's start with edges of node `0`.

```
[ ][ ][ ][ ][1][0] // 0 -> 1
[ ][ ][1][ ][ ][0] // 0 -> 3
```

`0->1` is the shortest path to `1`, which means that all other edges from `0` cannot be the shortest paths to `1`.
If they were, their bit 1 would be already set.

So, we set all other neighboring edges' bit 1 to 0.

```
[ ][ ][ ][ ][1][0] // 0 -> 1
[ ][ ][1][ ][0][0] // 0 -> 3
```

Same with `0->3`. Since we know that `0->3` is the shortest path to `3`, all other neighboring edges cannot be the shortest paths to `3`.
So, we set all other neighboring edges' bit 3 to 0.

```
[ ][ ][0][ ][1][0] // 0 -> 1
[ ][ ][1][ ][0][0] // 0 -> 3
```

Now, let's go to edges of node `1`. I flipped the edge `0->1` to `1->0` for easier visualization.

**node 1:**

```
[ ][ ][1][ ][0][1] // 1 -> 0
[ ][ ][ ][1][0][ ] // 1 -> 2
[ ][1][ ][ ][0][ ] // 1 -> 4
```

Since `1->0` is the shortest path to `0`, we set all neighboring edges bit 0 to `0`.

```
[ ][ ][1][ ][0][1] // 1 -> 0
[ ][ ][ ][1][0][0] // 1 -> 2
[ ][1][ ][ ][0][0] // 1 -> 4
```

Since `1->2` is the shortest path to `2`, we set all neighboring edges bit 2 to `0`.

```
[ ][ ][1][0][0][1] // 1 -> 0
[ ][ ][ ][1][0][0] // 1 -> 2
[ ][1][ ][0][0][0] // 1 -> 4
```

And same for `1->4`.

```
[ ][0][1][0][0][1] // 1 -> 0
[ ][0][ ][1][0][0] // 1 -> 2
[ ][1][ ][0][0][0] // 1 -> 4
```

We repeat this process for rest of the nodes:

**node 3:**

```
[ ][ ][0][ ][1][1] // 3 -> 0
[ ][1][0][ ][ ][ ] // 3 -> 4
[1][ ][0][ ][ ][ ] // 3 -> 5
```

becomes

```
[0][0][0][ ][1][1] // 3 -> 0
[0][1][0][ ][ ][0] // 3 -> 4
[1][0][0][ ][ ][0] // 3 -> 5
```

**node 4:**

```
[ ][0][ ][1][1][1] // 4 -> 1
[1][0][1][ ][ ][1] // 4 -> 3
```

becomes

```
[ ][0][0][1][1][1] // 4 -> 1
[1][0][1][ ][0][1] // 4 -> 3
```

After the gossip session, the edge matrix looks like this:

```
[ ][1][0][1][1][0] // 0 -> 1
[1][1][1][ ][0][0] // 0 -> 3
[ ][0][ ][1][0][0] // 1 -> 2
[ ][1][1][0][0][0] // 1 -> 4
[0][1][0][ ][1][0] // 3 -> 4
[1][0][0][ ][ ][0] // 3 -> 5
```

See how shortest-paths information spreads like wildfire? just like gossiping!

Hence, the name `bit_gossip`.

We repeat this process until either:

1. all bits in all edges are set, or
2. no bits were set in the iteration.

That's it!

Since all operations are bitwise, the computation is extremely fast.

Also, "gossiping" at each node is independent from other nodes, so we can parallelize the computation, which we do in the `Graph` type.

</details>

### Path Retrieval

<details>
  <summary>Click to expand</summary>

After all iterations are done, the matrix will look like this:

```
0 -- 1 -- 2
|    |
3 -- 4
|
5
```

```
[0][1][0][1][1][0] // 0 -> 1
[1][1][1][0][0][0] // 0 -> 3
[0][0][0][1][0][0] // 1 -> 2
[1][1][1][0][0][0] // 1 -> 4
[0][1][0][0][1][0] // 3 -> 4
[1][0][0][0][0][0] // 3 -> 5
```

Now, how do we use this data to retrieve the shortest path between two nodes?

Let's say we want to find the shortest path between node `2` and node `5`.

1. Check the edges of `2` at bit `5`.

Flip `1->2` to view bits of `2->1`.

```
[1][1][1][0][1][1] // 2 -> 1
```

We see that the bit 5 is set, so we move to node `1`.

2. Check the edges of `1` at bit `5`.

```
[1][0][1][0][0][1] // 1 -> 0
[0][0][0][1][0][0] // 1 -> 2
[1][1][1][0][0][0] // 1 -> 4
```

We see that both `1->0` and `1->4` have bit 5 set; at this point, we can either move to `0` or `4`.
Both will give the equal shortest path.

Let's move to `4`.

```
[0][0][0][1][1][1] // 4 -> 1
[1][0][1][1][0][1] // 4 -> 3
```

We see that the bit 5 is set for `4->3`, so we move to node `3`.

3. Check the edges of `3` at bit `5`.

```
[0][0][0][1][1][1] // 3 -> 0
[0][1][0][0][1][0] // 3 -> 4
[1][0][0][0][0][0] // 3 -> 5
```

We see that the bit 5 is set for `3->5`, so we move to node `5`.

4. We have reached node `5`.

Thus, the shortest path between node `2` and node `5` is `2 -> 1 -> 4 -> 3 -> 5`.

Hooraay!

You may think that this is a lot of work to find a path, but in reality, this is extremely fast.

In an actual game, we don't need to retrieve the entire path to the destination.

All we need is, "for the edges of the current node, which one is the shortest path to the destination node?"

After we go to the next node, we repeat the process.

If the destination is changed, again, we simply ask the question, "which neighbor should I go to next?"

</details>

## Benchmarks

The benchmarks below illustrate computation times for different graph sizes and types. They serve to highlight the performance characteristics of various graph representations rather than providing absolute metrics.

Note that different types of graphs will take different amount of time; For example, maze graphs took a bit more than just tile grids.

**Machine Specs:** Apple M3 Pro, 12-core CPU, 18GB RAM

The benchmarks were performed on tile grids where each node is connected to its four neighbors.

Here, `n` denotes the number of nodes, `e` the number of edges, and `(WxH)` the grid dimensions. For a grid of size `WxH`, there are `W*H` nodes and `2*W*H - W - H` edges.

### Processing Time

| Nodes, Edges, Grid Dim     | Graph16 | Graph32 | Graph64 | Graph128 | SeqGraph | ParaGraph |
| :------------------------- | ------: | ------: | ------: | -------: | -------: | --------: |
| 16n, 24e (4x4)             |   ~15µs |   ~15us |   ~15us |    ~15us |    ~45us |    ~500us |
| 32n, 52e (4x8)             |         |   ~45us |   ~40us |    ~48us |   ~150us |      ~1ms |
| 64n, 112e (8x8)            |         |         |  ~120us |   ~140us |   ~450us |    ~1.5ms |
| 128n, 232e (8x16)          |         |         |         |   ~390us |   ~1.5ms |    ~2.5ms |
| 256n, 480e (16x16)         |         |         |         |          |     ~5ms |      ~4ms |
| 512n, 976e (16x32)         |         |         |         |          |    ~18ms |     ~10ms |
| 1024n, 1984e (32x32)       |         |         |         |          |    ~55ms |     ~20ms |
| 2048n, 4000e (32x64)       |         |         |         |          |   ~200ms |     ~64ms |
| 2500n, 4900e (50x50)       |         |         |         |          |   ~400ms |    ~100ms |
| 4900n, 9660e (70x70)       |         |         |         |          |   ~1.70s |    ~300ms |
| 7225n, 14280e (85x85)      |         |         |         |          |    ~3.6s |    ~690ms |
| 10000n, 19800e (100x100)   |         |         |         |          |    ~6.8s |     ~1.3s |
| 20000n, 39700e (100x200)   |         |         |         |          |     ~29s |     ~6.3s |
| 40000n, 79600e (200x200)   |         |         |         |          |    ~140s |      ~27s |
| 102400n, 204160e (320x320) |         |         |         |          |          |     ~991s |

### Memory Usage

Below are the theoretical memory requirements for different graph types based on the number of nodes.

The function for calculating edge bits' memory usage is `n * m / 8` bytes, where `n` is the number of bits of data type
and `m` is the number of edges.

The function for calculating nodes neighbors data memory usage is:

- For `GraphN`, `n * N / 8` bytes, where `N` is the number of bits of data type, and `n` is the number of nodes.
- For `SeqGraph` and `ParaGraph`, `e * 2 * 2` bytes for graph less than 65536 nodes using `u16` for nodeID,
  and `e * 4 * 2` for graph more than 65536 nodes using `u32` for nodeID where `e` is the number of edges.

The value in chart below is the sum of edge bits and node neighbors data memory usage.

It does not account for memory overhead of atomics, hashmap or vector structures.

So in reality, the memory usage will be much higher than the values shown below.

Below chart shows memory usage in bytes `B`.

| Nodes, Edges, Grid Dim     | Graph16 | Graph32 | Graph64 | Graph128 |  SeqGraph | ParaGraph |
| :------------------------- | ------: | ------: | ------: | -------: | --------: | --------: |
| 16n, 24e (4x4)             |   ~80 B |  ~160 B |  ~320 B |   ~640 B |    ~288 B |    ~288 B |
| 32n, 52e (4x8)             |         |  ~336 B |  ~672 B |  ~1.4 KB |    ~624 B |    ~624 B |
| 64n, 112e (8x8)            |         |         | ~1.4 KB |  ~2.8 KB |  ~1.34 KB |  ~1.34 KB |
| 128n, 232e (8x16)          |         |         |         | ~5.75 KB |  ~4.63 KB |  ~4.63 KB |
| 256n, 480e (16x16)         |         |         |         |          |  ~17.3 KB |  ~17.3 KB |
| 512n, 976e (16x32)         |         |         |         |          |  ~66.3 KB |  ~66.3 KB |
| 1024n, 1984e (32x32)       |         |         |         |          | ~261.9 KB | ~261.9 KB |
| 2048n, 4000e (32x64)       |         |         |         |          |     ~1 MB |     ~1 MB |
| 2500n, 4900e (50x50)       |         |         |         |          |  ~1.56 MB |  ~1.56 MB |
| 4900n, 9660e (70x70)       |         |         |         |          |   ~6.0 MB |   ~6.0 MB |
| 7225n, 14280e (85x85)      |         |         |         |          |  ~12.9 MB |  ~12.9 MB |
| 10000n, 19800e (100x100)   |         |         |         |          |  ~24.9 MB |  ~24.9 MB |
| 20000n, 39700e (100x200)   |         |         |         |          |  ~99.4 MB |  ~99.4 MB |
| 40000n, 79600e (200x200)   |         |         |         |          |   ~398 MB |   ~398 MB |
| 102400n, 204160e (320x320) |         |         |         |          |           |  ~2.61 GB |

## Features

- **parallel**: Enable parallelism using Rayon; this feature is enabled by default.

## Examples

I have made a simple maze game using [bevy](https://bevyengine.org/) to compare `bit_gossip` and `astar`.

The game will create a 80x80 grid maze.

You can move the "player" (green dot) using the arrow keys.

Whenever you press the `<space>` key spawns 200 more enemies (red dots) that chase the player.

When the player and enemies collide, the enemies are removed.

**note**: for `bit_gossip`, building a graph initially with 80x80 grid maze takes a few seconds,
and the enemies will not move until built. When built, it will log `graph built in ...s` to the console.

You can run the `bit_gossip` version with:

```sh
cargo run --release -p maze
```

You can run the `astar` version with:

```sh
cargo run --release -p astar_maze
```

This is just to show how `bit_gossip` can be used in a game.

For me, at around 1000 enemies, I start to notice a lag in the `astar` version whenever the player moves.
`bit_gossip` version, however, does not show any lag regardless of the number of enemies.

I went up to more than 50000 enemies, and it shows no degradation in performance.

Still, it is amazing how well `astar` performs even with 1000 enemies chasing the player.
This means that for most games, `astar` is more than enough.

you can change the size by
going into [examples/maze/src/main.rs](https://github.com/PoOnesNerfect/bit_gossip/blob/main/examples/maze/src/main.rs) for `bit_gossip` version,
and [examples/astar_maze/src/main.rs](https://github.com/PoOnesNerfect/bit_gossip/blob/main/examples/astar_maze/src/main.rs) for `astar` version.

Change the values in `MazePlugin::new(50, 50)` to change the maze grid width and height.

### Astar Recording

https://github.com/user-attachments/assets/b1ddf4da-c885-4b86-b114-a2cce7ff7e98

### Bit Gossip Recording

https://github.com/user-attachments/assets/be9378f4-e3c8-452b-b94c-900b7a94b45a

## Optimization for Large Graphs

### Partitioning

Partitioning the graph into smaller graphs can help with memory usage and computation time.

With astar, I believe this is called hierarchical pathfinding?

If you have over 10000 nodes, you should definitely consider partitioning the graph.

### Dividing the Map Better

Instead of making tiles into graphs, think about dividing your map into rooms and doors.

A room is a space where any point is reachable from any other point trivially, like by drawing a straight line.

A door is an edge that connects two rooms.

This way, you can just find a path from room to room, at which point you can just move straight
to the more specific point in the room.

Personally, I think this is a better approach than partitioning the graph.

## Future Plans

When I was developing the algorithm, I found some interesting points about the algorithm
that I could explore further to find more ways to optimize the algorithm.

### Node with A Single Neighbor

When a node only has a sinlge neighbore, that neighbor is the shortest path to all other nodes.

Furthermore, if that neighbor only has two neighbors, that node's only other neighbor is the shortest path to all other nodes.

This can be a good optimization point for maze-like graphs.

### Delta/Flow of Information

I'm not sure what to call this yet, but this is related to partial updates of the graph.

Say, after the graph is built, we want to remove an edge.

How can we update the graph without rebuilding the entire graph? Which nodes should be updated?

One idea that I have not explored far enough is thinking of the graph as a flow of information.

The information lost due to removing that edge is the nodes that were reachable only through that edge.

```
lost bits = (merged bits of all other edges) ^ (bits of the edge to be removed)
```

Does this make sense?

So for the node's neighbors and their neighbors, we only unset the bits that were uniquely set by the edge that was removed.

And, for any node affected, if their neighbor also has the shortest path for any of the lost bits, we don't propagate it.

It seems not too hard... but I need to think about it more, and also about how much compute it will actually save?

### GPU/SIMD Acceleration

I'm not an expert on GPU or SIMD, but maybe it could be possible?

### Finding Dangling Nodes

It should also be trivial to find the nodes that are unreachable from all other nodes.

In a node, any node, merge all edges' bits together, and, if some bit are still 0, that node is unreachable.

### Moving Away from a Destination

It is also very easy to find a path to move away from a destination.

It's just the opposite of finding a path to the destination.

Find an edge with the bit set as 0, and move to that node.
It's uncertain whether it is the furthest path from the destination, but
at least you won't get closer faster than the shortest path.

## What Paper Did I Base This On?

**Short Answer:** None.

Of course, I tried looking into precomputed pathfinding in gamedev communities, but the only similar concept that I could find
was written back in 2003; Richard "superpig" Fine published an [article](https://archive.gamedev.net/archive/reference/articles/article1939.html)
that uses matrices to store the shortest paths between all pairs of nodes, although he does not give detail on how to actually compute the matrix.

Even after starting the project, I had no idea that there was a huge field of mathematics called
All-Pairs Shortest Paths (APSP) that is dedicated to solving this problem.

Only after I finished the initial implementation did it occur to me that
maybe this is a well-known problem with established solutions,
and I just didn't look hard enough.

So, then, I started looking into graph theory and found out about APSP, as well as papers on this topic.
I learned about Floyd-Warshall and Johnson's algorithm, and I found some papers on APSP.

There are some similar thoughts with Floyd-Warshall and Johnson's, but the implementation is quite different
as far as I know.

I tried reading papers on this topic, but either:

1. they were behind paywalls, or
2. I just could not understand them; how can the pseudo code be less readable than my actual code?
3. and none of them have actual code implementation nor real-life benchmarks.

So, at some point, I lost complete interest and gave up on reading papers;
I decided I'm just too dumb for academic life.

So, **long answer:** Also, none.

This implementation is original as far as I know, though there may be some similar concepts;
I also scraped the github repos to find any similar concepts or implementations, and found none.

It is entirely possible that someone already wrote a paper on this implementation, and I just couldn't find it;
if so, please email me at jackyldev@gmail.com

If you could also explain what the paper is in simple terms, I would be grateful, because I do want to understand
what they are saying.

With that being said, here are some papers that I wish I had access to or had the ability to understand:

- [On the All-Pairs-Shortest-Path Problem in Unweighted Undirected Graphs](https://www.sciencedirect.com/science/article/pii/S0022000085710781?via%3Dihub)
- [Original optimal method to solve the all-pairs shortest path problem: Dhouib-matrix-ALL-SPP - ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2666764924000109)
- [Scalable All-pairs Shortest Paths for Huge Graphs on Multi-GPU Clusters](https://dl.acm.org/doi/abs/10.1145/3431379.3460651)
- [All-pairs shortest paths for unweighted undirected graphs in o(mn) time | Proceedings of the seventeenth annual ACM-SIAM symposium on Discrete algorithm](https://dl.acm.org/doi/10.5555/1109557.1109614)

If anyone wants to explain what any of these papers are talking about, please email me at jackyldev@gmail.com

[Graph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/enum.Graph.html
[SeqGraph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/sequential/struct.SeqGraph.html
[ParaGraph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/parallel/struct.ParaGraph.html
[Graph16]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph16.html
[Graph32]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph32.html
[Graph64]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph64.html
[Graph128]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph128.html
