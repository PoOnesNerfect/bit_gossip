use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bit_gossip::Graph;
use std::sync::Arc;

pub const GRID_WIDTH: u16 = 50;
pub const GRID_HEIGHT: u16 = 50;
pub const GRID_SIZE: u16 = GRID_WIDTH * GRID_HEIGHT;

pub struct MazePlugin;

impl Plugin for MazePlugin {
    fn build(&self, app: &mut App) {
        let maze = bit_gossip::maze::build_maze(GRID_WIDTH, GRID_HEIGHT);
        let mut neighbors = vec![Vec::new(); GRID_SIZE as usize];
        for (a, b) in &maze {
            neighbors[*a as usize].push(*b);
            neighbors[*b as usize].push(*a);
        }

        app.insert_resource(Maze(maze.into()))
            .insert_resource(Neighbors(neighbors))
            .add_systems(Startup, compute_graph)
            .add_systems(Update, poll_compute_graph);
    }
}

// stores the edges of the maze
#[derive(Debug, Resource)]
pub struct Maze(pub Arc<Vec<(u16, u16)>>);

// stores the neighbors of nodes
#[derive(Debug, Resource)]
pub struct Neighbors(pub Vec<Vec<u16>>);

impl Neighbors {
    pub fn is_neighbor(&self, a: u16, b: u16) -> bool {
        self.0[a as usize].contains(&b)
    }
}

// precomputed graph
#[derive(Debug, Component)]
pub struct MyGraph(pub Graph);

#[derive(Component)]
struct ComputeGraph(Task<Graph>);

fn compute_graph(mut commands: Commands, maze: Res<Maze>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let maze = maze.0.clone();

    let task = thread_pool.spawn(async move {
        let mut builder = Graph::builder(GRID_SIZE as usize);

        for (a, b) in maze.as_ref() {
            builder.connect(*a, *b);
        }

        let now = std::time::Instant::now();

        let g = builder.build();

        println!("graph built in {:?}", now.elapsed());

        g
    });

    commands.spawn(ComputeGraph(task));
}

fn poll_compute_graph(
    mut commands: Commands,
    mut compute_graphs: Query<(Entity, &mut ComputeGraph)>,
) {
    for (entity, mut compute_graph) in compute_graphs.iter_mut() {
        if let Some(g) = block_on(future::poll_once(&mut compute_graph.0)) {
            commands.entity(entity).despawn_recursive();
            commands.spawn(MyGraph(g));
        }
    }
}
