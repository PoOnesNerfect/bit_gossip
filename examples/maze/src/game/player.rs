use super::{
    maze::{Neighbors, GRID_SIZE, GRID_WIDTH},
    movement::node_to_pos,
};
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
    neighbors: Res<Neighbors>,
    mut query: Query<&mut Player>,
) {
    for mut player in query.iter_mut() {
        let mut target = None;
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            target = get_neighbor(player.0, KeyCode::ArrowUp);
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            target = get_neighbor(player.0, KeyCode::ArrowDown);
        } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            target = get_neighbor(player.0, KeyCode::ArrowLeft);
        } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            target = get_neighbor(player.0, KeyCode::ArrowRight);
        }

        if let Some(target) = target {
            if neighbors.0[player.0 as usize].contains(&target) {
                player.0 = target;
            }
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player(0),
        Transform::from_translation(node_to_pos(0).extend(3.)),
    ));
}

fn get_neighbor(node: u16, dir: KeyCode) -> Option<u16> {
    match dir {
        KeyCode::ArrowUp => {
            if node < GRID_WIDTH {
                None
            } else {
                Some(node - GRID_WIDTH)
            }
        }
        KeyCode::ArrowDown => {
            if node + GRID_WIDTH > GRID_SIZE {
                None
            } else {
                Some(node + GRID_WIDTH)
            }
        }
        KeyCode::ArrowLeft => {
            if node % GRID_WIDTH == 0 {
                None
            } else {
                Some(node - 1)
            }
        }
        KeyCode::ArrowRight => {
            if node % GRID_WIDTH == GRID_WIDTH - 1 {
                None
            } else {
                Some(node + 1)
            }
        }
        _ => None,
    }
}
