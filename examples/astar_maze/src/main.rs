use bevy::prelude::*;
use maze::MazePlugin;

mod plugin;

const WINDOW_SCALE: f32 = 3.5;
const WINDOW_SIZE: Vec2 = Vec2::new(480. * WINDOW_SCALE, 270. * WINDOW_SCALE);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "maze".to_string(),
                        resolution: WINDOW_SIZE.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((MazePlugin::new(80, 80), plugin::AstarPlugin))
        .run();
}
