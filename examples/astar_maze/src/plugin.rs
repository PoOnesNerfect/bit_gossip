use bevy::prelude::*;
use maze::{
    game::{
        enemy::Enemy,
        movement::{CurrentNode, TargetNode},
        player::Player,
    },
    GridDimensions, Neighbors,
};
use pathfinding::directed::astar;
use std::vec;

pub struct AstarPlugin;

impl Plugin for AstarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (follow_player, move_to_next_tile, insert_path));
    }
}

#[derive(Debug, Component)]
pub struct AstarPath(pub vec::IntoIter<u16>);

fn follow_player(
    mut commands: Commands,
    grid: Res<GridDimensions>,
    neighbors: Res<Neighbors>,
    player: Query<&Player, Changed<Player>>,
    mut query: Query<(Entity, &CurrentNode, &mut TargetNode, &mut AstarPath), With<Enemy>>,
) {
    let Ok(Player(player)) = player.get_single() else {
        return;
    };

    for (enemy, curr, mut target, mut path) in query.iter_mut() {
        if curr.0 == *player {
            commands.entity(enemy).despawn();
            continue;
        }

        path.0 = astar_path(curr.0, *player, &neighbors.0, &grid);

        if let Some(new_target) = path.0.next() {
            if target.0 != new_target {
                target.0 = new_target;
            }
        }
    }
}

fn move_to_next_tile(
    mut commands: Commands,
    player: Query<&Player>,
    mut query: Query<
        (Entity, &CurrentNode, &mut TargetNode, &mut AstarPath),
        (With<Enemy>, Changed<CurrentNode>),
    >,
) {
    let Ok(Player(player)) = player.get_single() else {
        return;
    };

    for (id, CurrentNode(curr), mut target, mut path) in query.iter_mut() {
        if *curr == *player {
            commands.entity(id).despawn();
            continue;
        }

        if let Some(new_target) = path.0.next() {
            if target.0 != new_target {
                target.0 = new_target;
            }
        }
    }
}

fn insert_path(
    mut commands: Commands,
    grid: Res<GridDimensions>,
    neighbors: Res<Neighbors>,
    mut query: Query<(Entity, &CurrentNode, &mut TargetNode), Added<Enemy>>,
    player: Query<&Player>,
) {
    let player = player.single();

    for (id, curr, mut target) in query.iter_mut() {
        let mut path = astar_path(curr.0, player.0, &neighbors.0, &grid);
        if let Some(next) = path.next() {
            target.0 = next;
        }

        commands.entity(id).insert(AstarPath(path));
    }
}

fn astar_path(
    curr: u16,
    dest: u16,
    neighbors: &Vec<Vec<u16>>,
    grid: &GridDimensions,
) -> vec::IntoIter<u16> {
    let node_to_pos = |node: u16| (node % grid.width, node / grid.width);

    let dest_pos = node_to_pos(dest);

    let mut path = astar::astar(
        &curr,
        |node| neighbors[*node as usize].iter().map(|n| (n.clone(), 1)),
        |node| {
            let node_pos = node_to_pos(*node);
            (dest_pos.0 as i32 - node_pos.0 as i32).pow(2)
                + (dest_pos.1 as i32 - node_pos.1 as i32).pow(2)
        },
        |node| *node == dest,
    )
    .unwrap()
    .0
    .into_iter();

    // curr
    path.next();

    path
}
