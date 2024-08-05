use crate::{graphics::BOARD_SIZE, GridDimensions};
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, movement);
    }
}

#[derive(Bundle)]
pub struct MovementBundle {
    pub speed: MovementSpeed,
    pub current: CurrentNode,
    pub target: TargetNode,
}

impl MovementBundle {
    pub fn new(current: u16) -> Self {
        Self {
            speed: MovementSpeed(2.),
            current: CurrentNode(current),
            target: TargetNode(current),
        }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = MovementSpeed(speed);
        self
    }

    pub fn with_target(mut self, target: u16) -> Self {
        self.target = TargetNode(target);
        self
    }
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct CurrentNode(pub u16);

#[derive(Component)]
pub struct TargetNode(pub u16);

fn movement(
    time: Res<Time>,
    grid: Res<GridDimensions>,
    mut query: Query<(
        &mut Transform,
        &MovementSpeed,
        &mut CurrentNode,
        &TargetNode,
    )>,
) {
    for (mut tf, speed, mut curr, target) in query.iter_mut() {
        if curr.0 == target.0 {
            continue;
        }

        let target_cell = node_to_cell(target.0, &grid);
        let curr_cell = node_to_cell(curr.0, &grid);

        let curr_pos = node_to_pos(curr.0, &grid);
        let target_pos = node_to_pos(target.0, &grid);

        if target_cell.y == curr_cell.y {
            let tl = &mut tf.translation;

            if tl.y < curr_pos.y {
                tl.y += speed.0 * time.delta_seconds();
                if tl.y > curr_pos.y {
                    tl.y = curr_pos.y;
                }
            } else if tl.y > curr_pos.y {
                tl.y -= speed.0 * time.delta_seconds();
                if tl.y < curr_pos.y {
                    tl.y = curr_pos.y;
                }
            } else if tl.x < target_pos.x {
                tl.x += speed.0 * time.delta_seconds();
                if tl.x > target_pos.x {
                    tl.x = target_pos.x;
                    curr.0 = target.0;
                }
            } else if tl.x > target_pos.x {
                tl.x -= speed.0 * time.delta_seconds();
                if tl.x < target_pos.x {
                    tl.x = target_pos.x;
                    curr.0 = target.0;
                }
            }
        } else if target_cell.x == curr_cell.x {
            let tl = &mut tf.translation;

            if tl.x < curr_pos.x {
                tl.x += speed.0 * time.delta_seconds();
                if tl.x > curr_pos.x {
                    tl.x = curr_pos.x;
                }
            } else if tl.x > curr_pos.x {
                tl.x -= speed.0 * time.delta_seconds();
                if tl.x < curr_pos.x {
                    tl.x = curr_pos.x;
                }
            } else if tl.y < target_pos.y {
                tl.y += speed.0 * time.delta_seconds();
                if tl.y > target_pos.y {
                    tl.y = target_pos.y;
                    curr.0 = target.0;
                }
            } else if tl.y > target_pos.y {
                tl.y -= speed.0 * time.delta_seconds();
                if tl.y < target_pos.y {
                    tl.y = target_pos.y;
                    curr.0 = target.0;
                }
            }
        }
    }
}

pub fn node_to_pos(node: u16, grid: &GridDimensions) -> Vec2 {
    let cell_size = BOARD_SIZE / Vec2::new(grid.width as f32, grid.height as f32);

    let (x, y) = node_to_cell(node, grid).into();

    let pos_x = x as f32 * cell_size.x + cell_size.x / 2. - BOARD_SIZE.x / 2.;
    let pos_y = BOARD_SIZE.y / 2. - y as f32 * cell_size.y - cell_size.y / 2.;

    Vec2::new(pos_x, pos_y)
}

pub fn node_to_cell(node: u16, grid: &GridDimensions) -> UVec2 {
    let x = node % grid.width;
    let y = node / grid.width;

    (x as u32, y as u32).into()
}
