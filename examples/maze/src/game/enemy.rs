use bevy::prelude::*;
use rand::rngs::OsRng;
use rand::Rng;

use crate::GridDimensions;

use super::movement::{node_to_pos, MovementBundle};
use super::player::Player;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_enemies);
    }
}

#[derive(Component)]
pub struct Enemy;

fn spawn_enemies(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid: Res<GridDimensions>,
    player: Query<&Player>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player = player.single().0;
        let grid_size = grid.size();

        for _ in 0..200 {
            let node = loop {
                let node = OsRng.gen_range(0..grid_size);
                if node != player {
                    break node;
                }
            };

            commands.spawn((
                Enemy,
                Transform::from_translation(node_to_pos(node, &grid).extend(3.)),
                MovementBundle::new(node).with_speed(30.),
            ));
        }
    }
}
