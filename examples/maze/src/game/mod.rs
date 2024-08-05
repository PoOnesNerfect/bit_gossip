use bevy::prelude::*;

pub mod enemy;
pub mod maze;
pub mod movement;
pub mod player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            maze::MazePlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            movement::MovementPlugin,
        ));
    }
}
