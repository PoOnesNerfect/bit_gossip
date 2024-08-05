use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bit_gossip::Graph;

use crate::{
    game::{
        enemy::Enemy,
        movement::{CurrentNode, TargetNode},
        player::Player,
    },
    GridDimensions, Maze,
};

pub struct BitGossipPlugin;

impl Plugin for BitGossipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, compute_graph)
            .add_systems(Update, poll_compute_graph)
            .add_systems(Update, (follow_player, move_to_next_tile));
    }
}

// precomputed graph
#[derive(Debug, Component)]
pub struct MyGraph(pub Graph);

#[derive(Component)]
struct ComputeGraph(Task<Graph>);

fn follow_player(
    mut commands: Commands,
    graph: Query<&MyGraph>,
    player: Query<&Player, Changed<Player>>,
    mut query: Query<(Entity, &CurrentNode, &mut TargetNode), With<Enemy>>,
) {
    let Ok(Player(player)) = player.get_single() else {
        return;
    };
    let Ok(g) = graph.get_single() else {
        return;
    };

    for (enemy, curr, mut target) in query.iter_mut() {
        if curr.0 == *player {
            commands.entity(enemy).despawn();
            continue;
        }

        if let Some(new_target) = g.0.neighbor_to(curr.0, *player) {
            if target.0 != new_target {
                target.0 = new_target;
            }
        }
    }
}

fn move_to_next_tile(
    mut commands: Commands,
    graph: Query<&MyGraph>,
    player: Query<&Player>,
    mut query: Query<(Entity, &CurrentNode, &mut TargetNode), (With<Enemy>, Changed<CurrentNode>)>,
) {
    let Ok(Player(player)) = player.get_single() else {
        return;
    };
    let Ok(g) = graph.get_single() else {
        return;
    };

    for (id, CurrentNode(curr), mut target) in query.iter_mut() {
        if *curr == *player {
            commands.entity(id).despawn();
            continue;
        }

        if let Some(new_target) = g.0.neighbor_to(*curr, *player) {
            if target.0 != new_target {
                target.0 = new_target;
            }
        }
    }
}

fn compute_graph(mut commands: Commands, maze: Res<Maze>, grid: Res<GridDimensions>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let maze = maze.0.clone();

    let grid_size = grid.size();

    let task = thread_pool.spawn(async move {
        let mut builder = Graph::builder(grid_size as usize);

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
