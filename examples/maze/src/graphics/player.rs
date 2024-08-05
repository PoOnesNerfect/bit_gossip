use crate::{
    game::{movement::node_to_pos, player::Player},
    GridDimensions,
};
use bevy::{color::palettes::tailwind::GREEN_500, prelude::*, sprite::MaterialMesh2dBundle};

use super::CharacterMesh;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_player, follow_player));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh: Res<CharacterMesh>,
    grid: Res<GridDimensions>,
    player: Query<(Entity, &Player), Added<Player>>,
) {
    for (id, Player(node)) in &player {
        commands.entity(id).insert(MaterialMesh2dBundle {
            mesh: mesh.0.clone().into(),
            material: materials.add(Color::from(GREEN_500)),
            transform: Transform::from_translation(node_to_pos(*node, &grid).extend(4.)),
            ..Default::default()
        });
    }
}

fn follow_player(
    grid: Res<GridDimensions>,
    mut query: Query<(&Player, &mut Transform), Changed<Player>>,
) {
    for (Player(node), mut transform) in query.iter_mut() {
        *transform = Transform::from_translation(node_to_pos(*node, &grid).extend(4.));
    }
}
