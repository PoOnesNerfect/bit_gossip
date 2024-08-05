use crate::{GridDimensions, Neighbors};

use super::movement::node_to_pos;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, handle_input);
    }
}

#[derive(Component)]
pub struct Player(pub u16);

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid: Res<GridDimensions>,
    neighbors: Res<Neighbors>,
    mut query: Query<&mut Player>,
) {
    for mut player in query.iter_mut() {
        let mut target = None;
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            target = get_neighbor(player.0, KeyCode::ArrowUp, &grid);
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            target = get_neighbor(player.0, KeyCode::ArrowDown, &grid);
        } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            target = get_neighbor(player.0, KeyCode::ArrowLeft, &grid);
        } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            target = get_neighbor(player.0, KeyCode::ArrowRight, &grid);
        }

        if let Some(target) = target {
            if neighbors.0[player.0 as usize].contains(&target) {
                player.0 = target;
            }
        }
    }
}

fn spawn_player(mut commands: Commands, grid: Res<GridDimensions>) {
    commands.spawn((
        Player(0),
        Transform::from_translation(node_to_pos(0, &grid).extend(3.)),
    ));
}

fn get_neighbor(node: u16, dir: KeyCode, grid: &GridDimensions) -> Option<u16> {
    match dir {
        KeyCode::ArrowUp => {
            if node < grid.width {
                None
            } else {
                Some(node - grid.width)
            }
        }
        KeyCode::ArrowDown => {
            if node + grid.width > grid.size() {
                None
            } else {
                Some(node + grid.width)
            }
        }
        KeyCode::ArrowLeft => {
            if node % grid.width == 0 {
                None
            } else {
                Some(node - 1)
            }
        }
        KeyCode::ArrowRight => {
            if node % grid.width == grid.width - 1 {
                None
            } else {
                Some(node + 1)
            }
        }
        _ => None,
    }
}
