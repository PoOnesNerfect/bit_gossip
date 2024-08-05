use crate::game::maze::{Maze, GRID_HEIGHT, GRID_WIDTH};
use bevy::{color::palettes::tailwind::GRAY_200, prelude::*, sprite::MaterialMesh2dBundle};

mod enemy;
mod player;

pub const BOARD_SIZE: Vec2 = Vec2::new(270. * 3., 270. * 3.);

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player::PlayerPlugin, enemy::EnemyPlugin))
            .add_systems(Startup, (camera_setup, insert_character_mesh, draw_maze));
    }
}

#[derive(Resource)]
pub struct CharacterMesh(pub Handle<Mesh>);

fn insert_character_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let circle_mesh = meshes.add(Circle::new(5.));
    commands.insert_resource(CharacterMesh(circle_mesh));
}

// spawn one big board
// with black lines (GRID_WIDTH - 1) vertically and (GRID_HEIGHT - 1) horizontally.
// For all the edges in the maze, spawn a white to overwrite the black line for the edge.
fn draw_maze(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    maze: Res<Maze>,
) {
    let bg = materials.add(Color::from(GRAY_200));

    let board = MaterialMesh2dBundle {
        mesh: meshes
            .add(Rectangle::new(BOARD_SIZE.x, BOARD_SIZE.y))
            .into(),
        material: bg.clone(),
        ..Default::default()
    };

    commands.spawn(board);

    let black = materials.add(Color::BLACK);

    let x_wall_mesh = meshes.add(Rectangle::new(2., BOARD_SIZE.y));
    for i in 0..GRID_WIDTH {
        let x = BOARD_SIZE.x / GRID_WIDTH as f32 * i as f32;
        let wall = MaterialMesh2dBundle {
            mesh: x_wall_mesh.clone().into(),
            material: black.clone(),
            transform: Transform::from_translation(Vec3::new(x - BOARD_SIZE.x / 2., 0., 1.)),
            ..Default::default()
        };
        commands.spawn(wall);
    }

    let y_wall_mesh = meshes.add(Rectangle::new(BOARD_SIZE.x, 2.));

    for i in 0..GRID_HEIGHT {
        let y = BOARD_SIZE.y / GRID_HEIGHT as f32 * i as f32;
        let wall = MaterialMesh2dBundle {
            mesh: y_wall_mesh.clone().into(),
            material: black.clone(),
            transform: Transform::from_translation(Vec3::new(0., y - BOARD_SIZE.y / 2., 1.)),
            ..Default::default()
        };
        commands.spawn(wall);
    }

    let x_edge_mesh = meshes.add(Rectangle::new(2., BOARD_SIZE.y / GRID_HEIGHT as f32 - 1.));
    let y_edge_mesh = meshes.add(Rectangle::new(BOARD_SIZE.x / GRID_WIDTH as f32 - 1., 2.));

    let cell_size = BOARD_SIZE / Vec2::new(GRID_WIDTH as f32, GRID_HEIGHT as f32);

    for (a, b) in maze.0.iter() {
        let (a_x, a_y) = (a % GRID_WIDTH, a / GRID_WIDTH);
        let (b_x, b_y) = (b % GRID_WIDTH, b / GRID_WIDTH);

        if a_y == b_y {
            // Horizontal edge
            let x = (a_x + b_x) as f32 / 2. * cell_size.x;
            let x = x - (BOARD_SIZE.x / 2.) + (cell_size.x / 2.);

            let y = a_y as f32 * cell_size.y;
            let y = (BOARD_SIZE.y / 2.) - y - (cell_size.y / 2.);

            let wall = MaterialMesh2dBundle {
                mesh: x_edge_mesh.clone().into(),
                material: bg.clone(),
                transform: Transform::from_translation(Vec3::new(x, y, 2.)),
                ..Default::default()
            };
            commands.spawn(wall);
        } else {
            // Vertical edge
            let x = a_x as f32 * cell_size.x;
            let x = x + (cell_size.x / 2.) - (BOARD_SIZE.x / 2.);

            let y = (a_y + b_y) as f32 / 2. * cell_size.y;
            let y = (BOARD_SIZE.y / 2.) - y - (cell_size.y / 2.);

            let wall = MaterialMesh2dBundle {
                mesh: y_edge_mesh.clone().into(),
                material: bg.clone(),
                transform: Transform::from_translation(Vec3::new(x, y, 2.)),
                ..Default::default()
            };
            commands.spawn(wall);
        }
    }
}

fn camera_setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.;
    commands.spawn(camera);
}
