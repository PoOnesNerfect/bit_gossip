# bit_gossip

[<img alt="github" src="https://img.shields.io/badge/github-poonesnerfect/bit_gossip-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/poonesnerfect/bit_gossip)
[<img alt="crates.io" src="https://img.shields.io/crates/v/bit_gossip.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/bit_gossip)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-bit_gossip-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/bit_gossip)

**bit_gossip** is a simple pathfinding library for calculating all node pairs' shortest paths in an unweighted undirected graph.

Once the computation is complete, you can retrieve the shortest path between any two nodes in near constant time; I'm talking _few microseconds_!

This library is ideal for, but not limited to, games where there are numerous entities that need to find the paths to a moving target.

The computation is fast enough to be used not only in static maps but also in dynamically changing maps in games.

- Computing all paths for 100 nodes takes only a few microseconds on a modern CPU.
- Computing all paths for 1000 nodes takes less than 50ms on a modern CPU.
- Computing all paths for 10000 nodes only takes less a few seconds.

See [benchmarks](#benchmarks) for more details.

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
assert_eq!(graph.next_node(0, 9), Some(4));
assert_eq!(graph.next_node(4, 9), Some(8));
assert_eq!(graph.next_node(8, 9), Some(9));

// Both 1 and 4 can reach 11 in the shortest path.
assert_eq!(graph.next_nodes(0, 11).collect::<Vec<_>>(), vec![1, 4]);

// Get the path from 0 to 5
assert_eq!(graph.path_to(0, 5).collect::<Vec<_>>(), vec![4, 5]);
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

let mut count = 0;

while curr != dest {
    let prev = curr;
    curr = graph.next_node(curr, dest).unwrap();
    println!("{prev} -> {curr}");

    count += 1;
    if curr == dest {
        println!("we've reached node '{dest}' in {count} hops!");
        break;
    }
}
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

For varying map sizes over 128 nodes, use `SeqGraph` or `ParaGraph`. In multi-threaded environments, `ParaGraph` is recommended.

The library exposes a generic `Graph` type that automatically selects the parallel or sequential version based on the environment, with manual selection also available.

## Benchmarks

The benchmarks below illustrate computation times for different graph sizes and types. They serve to highlight the performance characteristics of various graph representations rather than providing absolute metrics.

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

## Background

I am an avid day-dreamer. I am constantly thinking about solutions to problems, instead of
googling and finding out that the problem has already been solved 30 years ago.

My journey with _bit_gossip_ began with a quest for optimizing pathfinding in games like _Vampire Survivors_,
where numerous enemies chase a constantly moving player.
Traditional algorithms like A* and Dijkstra are effective for static destinations
but become costly with dynamic targets.

There are optimizations you can try, of course, such as:

1. grouping entities together and computing the shortest path for the group,
2. caching the shortest paths for a certain amount of time,
3. using a hierarchical pathfinding algorithm.

**1** seems hard to implement well,
**2** can be expensive in terms of memory and not worth it if the player is constantly moving around to different places, and
**3** actually sounds like a good idea, but at this point, I was already thinking about a different solution.

So, like any good game developer, instead of actually working on my game,
I started thinking about way of optimizing it.

I started thinking about precomputing paths.
This is an attractive approach since I can compute paths at game loading time, or even compile time,
and, after the game starts, shortest paths to player from any position can be fetched in constant time.

However, some concerns came to mind immediately:

1. It may take a very long time to compute all shortest paths between all pairs of nodes.
2. Storing all shortest paths may take up a lot of memory.

This eventually led me to explore bitwise storage of path information,
where each edge holds **n** bits indicating the presence of shortest paths through it.

**Note**

At time of implementing _bit_gossip_, I was not aware this idea had already been under research for decades.

The idea of asking 'which node should I go to next?' in gamedev is also not original at all.
Even back in 2003, Richard "superpig" Fine published an [article](https://archive.gamedev.net/archive/reference/articles/article1939.html)
that uses matrices to store the shortest paths between all pairs of nodes, although it does not give detail on how to actually compute the matrix.

I also later found out that there is an entire field of mathematics that is dedicated to solving this problem,
called _All-Pairs Shortest Paths (APSP)_ with sub-field for unweighted undirected graphs.

There are established algorithms that solve this problem such as Floyd-Warshall and Johnson's algorithm.

There are also many many academic papers on this topic that claim to have solved this problem in the smallest computational complexity.

I would like to know if my algorithm is already known in some form in the academic world; however, it has been pretty hard for me to find out.

Many of these papers are behind paywalls, and even when they are accessible, I find them very hard to understand, as I am not an academic.
When psuedo codes are less readable than actual code, I just lose all my interest in trying.

So, if you are an academic in this field, and you see that my implementation already exists in some form in some paper,
please let me know. If you could also explain it to me in simple terms, I will be very grateful.

Here are some papers that I found that are related to this topic, that I wish I could read:

- [On the All-Pairs-Shortest-Path Problem in Unweighted Undirected Graphs](https://www.sciencedirect.com/science/article/pii/S0022000085710781?via%3Dihub)
- [Scalable All-pairs Shortest Paths for Huge Graphs on Multi-GPU Clusters](https://dl.acm.org/doi/abs/10.1145/3431379.3460651)
- [All-pairs shortest paths for unweighted undirected graphs in o(mn) time | Proceedings of the seventeenth annual ACM-SIAM symposium on Discrete algorithm](https://dl.acm.org/doi/10.5555/1109557.1109614)

In terms of non-academic projects go, I do think that this project is unique in its approach.
At least I could not find any project that focuses on this specific problem that does not use Floyd-Warshall or Johnson's algorithm.
If you also find a project that is similar to _bit_gossip_, please let me know.

It's hard to find benchmarks for APSP algorithms, let alone any projects that implement them in actual code, not just in pseudo code.
From the very few benchmarks that I was able to find, _bit_gossip_ does seem to outperform them by a long shot.
If you also find benchmarks for other APSP implementations, please do let me know.

[Graph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/enum.Graph.html
[SeqGraph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/sequential/struct.SeqGraph.html
[ParaGraph]: https://docs.rs/bit_gossip/latest/bit_gossip/graph/parallel/struct.ParaGraph.html
[Graph16]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph16.html
[Graph32]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph32.html
[Graph64]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph64.html
[Graph128]: https://docs.rs/bit_gossip/latest/bit_gossip/prim/struct.Graph128.html
