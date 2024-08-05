use bevy::prelude::*;
use std::sync::Arc;

pub const WINDOW_SCALE: f32 = 3.5;
pub const WINDOW_SIZE: Vec2 = Vec2::new(480. * WINDOW_SCALE, 270. * WINDOW_SCALE);

pub mod bit_gossip;
pub mod game;
pub mod graphics;

#[derive(Debug)]
pub struct MazePlugin {
    grid: GridDimensions,
}

impl MazePlugin {
    pub fn new(width: u16, height: u16) -> Self {
        MazePlugin {
            grid: GridDimensions { width, height },
        }
    }
}

impl Plugin for MazePlugin {
    fn build(&self, app: &mut App) {
        let grid = &self.grid;

        let maze = ::bit_gossip::maze::build_maze(grid.width, grid.height);
        let mut neighbors = vec![Vec::new(); grid.size() as usize];
        for (a, b) in &maze {
            neighbors[*a as usize].push(*b);
            neighbors[*b as usize].push(*a);
        }

        app.insert_resource(Maze(maze.into()))
            .insert_resource(Neighbors(neighbors))
            .insert_resource(grid.clone())
            .add_plugins(DebugPlugin)
            .add_plugins((game::GamePlugin, graphics::GraphicsPlugin));
    }
}

#[derive(Debug, Clone, Resource)]
pub struct GridDimensions {
    pub width: u16,
    pub height: u16,
}

impl Default for GridDimensions {
    fn default() -> Self {
        GridDimensions {
            width: 50,
            height: 50,
        }
    }
}

impl GridDimensions {
    pub fn size(&self) -> u16 {
        self.width * self.height
    }
}

// stores the edges of the maze
#[derive(Debug, Resource)]
pub struct Maze(pub Arc<Vec<(u16, u16)>>);

// stores the neighbors of nodes
#[derive(Debug, Resource)]
pub struct Neighbors(pub Vec<Vec<u16>>);

impl Neighbors {
    pub fn is_neighbor(&self, a: u16, b: u16) -> bool {
        self.0[a as usize].contains(&b)
    }
}

struct DebugPlugin;

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
