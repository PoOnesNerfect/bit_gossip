use bevy::prelude::*;
use rand::rngs::OsRng;
use rand::Rng;

use super::maze::{MyGraph, GRID_SIZE};
use super::movement::{node_to_pos, CurrentNode, MovementBundle, TargetNode};
use super::player::Player;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_enemies, follow_player, move_to_next_tile));
    }
}

#[derive(Component)]
pub struct Enemy;

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

fn spawn_enemies(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    graph: Query<&MyGraph>,
    player: Query<&Player>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player = player.single().0;
        let g = graph.get_single();

        for _ in 0..50 {
            let node = loop {
                let node = OsRng.gen_range(0..GRID_SIZE);
                if node != player {
                    break node;
                }
            };

            let target = if let Ok(g) = g.as_ref() {
                g.0.neighbor_to(node, player).unwrap_or(node)
            } else {
                node
            };

            commands.spawn((
                Enemy,
                Transform::from_translation(node_to_pos(node).extend(3.)),
                MovementBundle::new(node)
                    .with_speed(30.)
                    .with_target(target),
            ));
        }
    }
}
