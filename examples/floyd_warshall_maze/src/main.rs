use bit_gossip::maze::build_maze;
use petgraph::{algo::floyd_warshall, graph::UnGraph};

const GRID_WIDTH: u32 = 50;
const GRID_HEIGHT: u32 = 50;
const GRID_SIZE: u32 = GRID_WIDTH * GRID_HEIGHT;

fn main() {
    // Initialize a builder with 10000 nodes
    // let mut builder = Graph::builder(GRID_SIZE as usize);

    let maze = build_maze(GRID_WIDTH, GRID_HEIGHT);

    let mut g = UnGraph::<u32, ()>::from_edges(&maze);

    for i in 0..GRID_SIZE {
        g.add_node(i);
    }

    // Connect the nodes
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let node = y * GRID_WIDTH + x;

            if x < GRID_WIDTH - 1 {
                // builder.connect(node, node + 1);
                g.add_edge(node.into(), (node + 1).into(), ());
            }
            if y < GRID_HEIGHT - 1 {
                // builder.connect(node, node + GRID_WIDTH);
                g.add_edge(node.into(), (node + GRID_WIDTH).into(), ());
            }
        }
    }

    let now = std::time::Instant::now();

    let _res = floyd_warshall(&g, |_| 1).unwrap();

    println!("graph built in {:?}", now.elapsed());
}
