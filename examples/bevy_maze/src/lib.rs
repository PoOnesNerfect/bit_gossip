use bevy_app::Plugin;
use bit_gossip::maze::build_maze;

pub struct MazePlugin;

impl Plugin for MazePlugin {
    fn build(&self, app: &mut bevy_app::App) {}
}
