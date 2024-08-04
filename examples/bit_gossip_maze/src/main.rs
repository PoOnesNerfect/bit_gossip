use bit_gossip::Graph;
use maze::build_maze;

const GRID_WIDTH: u16 = 50;
const GRID_HEIGHT: u16 = 50;
const GRID_SIZE: u16 = GRID_WIDTH * GRID_HEIGHT;

fn main() {
    let maze = build_maze(GRID_WIDTH, GRID_HEIGHT);

    let mut builder = Graph::builder(GRID_SIZE as usize);

    for (a, b) in maze {
        builder.connect(a, b);
    }

    let now = std::time::Instant::now();

    let _graph = builder.build();

    println!("graph built in {:?}", now.elapsed());
}
