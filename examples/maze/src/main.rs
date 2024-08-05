use bevy::prelude::*;

const WINDOW_SCALE: f32 = 3.5;
const WINDOW_SIZE: Vec2 = Vec2::new(480. * WINDOW_SCALE, 270. * WINDOW_SCALE);

mod game;
mod graphics;

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
        .add_plugins(DebugPlugin)
        .add_plugins((game::GamePlugin, graphics::GraphicsPlugin))
        .run();
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        use iyes_perf_ui::prelude::*;

        app.add_systems(Startup, setup)
            // .add_plugins(avian2d::debug_render::PhysicsDebugPlugin::default())
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin);

        fn setup(mut commands: Commands) {
            let mut fps = PerfUiEntryFPS::default();
            fps.label = "fps".to_string();

            let mut fps_worst = PerfUiEntryFPSWorst::default();
            fps_worst.label = "fps (min)".to_string();

            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn((
                PerfUiRoot {
                    display_labels: true,
                    layout_horizontal: false,
                    ..default()
                },
                fps_worst,
                fps,
                PerfUiEntryEntityCount::default(),
            ));
        }
    }
}
