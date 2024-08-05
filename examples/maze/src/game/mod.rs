use bevy::prelude::*;

pub mod enemy;
pub mod movement;
pub mod player;

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            movement::MovementPlugin,
        ));
    }
}
