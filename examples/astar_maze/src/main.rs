use bit_gossip::Graph;
use pathfinding::prelude::astar;

type NodeId = u16;

const GRID_WIDTH: NodeId = 25;
const GRID_HEIGHT: NodeId = 25;
const GRID_SIZE: NodeId = GRID_WIDTH * GRID_HEIGHT;

fn main() {
    // Initialize a builder with 10000 nodes
    let mut builder = Graph::builder(GRID_SIZE as usize);

    for (a, b) in maze::build_maze(GRID_WIDTH as u16, GRID_HEIGHT as u16) {
        builder.connect(a as NodeId, b as NodeId);
    }

    let now = std::time::Instant::now();

    let graph = builder.build();

    println!("graph built in {:?}", now.elapsed());

    // Check the shortest path from 0 to 9900
    // This is fast
    let mut curr = 0;
    let dest = 98;

    // println!("path exists: {}", graph.path_exists(curr, dest));

    let node_to_pos = |node: NodeId| (node % 100, node / 100);

    let dest_pos = node_to_pos(dest);

    let now = std::time::Instant::now();

    let astar_path = astar(
        &curr,
        |node| graph.neighbors(*node).iter().map(|n| (n.clone(), 1)),
        |node| {
            let node_pos = node_to_pos(*node);
            (dest_pos.0 as i32 - node_pos.0 as i32).pow(2)
                + (dest_pos.1 as i32 - node_pos.1 as i32).pow(2)
        },
        |node| *node == dest,
    )
    .unwrap()
    .0;

    println!("astar path found in {:?}", now.elapsed());
    // println!("{:?}", astar_path);

    let now = std::time::Instant::now();

    // let path = graph.path_to(curr, dest).collect::<Vec<_>>();
    // println!("{:?}", path);

    let next = graph.next_node(curr, dest).unwrap();

    println!("bit gossip path found in {:?}", now.elapsed());

    assert_eq!(astar_path[1], next as NodeId);
}
