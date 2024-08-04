use bevy_app::Plugin;
use bit_gossip::graph::U16orU32;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

pub struct MazePlugin;

impl Plugin for MazePlugin {
    fn build(&self, app: &mut bevy_app::App) {}
}

/// Builds a maze of the given width and height.
/// Returns a list of pairs of cells that are connected.
pub fn build_maze<N: U16orU32>(w: N, h: N) -> Vec<(N, N)> {
    let mut maze = Vec::with_capacity(w.as_usize() * h.as_usize());

    // create maze with following algorithm, in a loop, not recursive:
    // 1. choose a cell to begin
    // 2. from your current cell, choose a random neighbor that you haven’t visited yet
    // 3. move to the chosen neighbor, knocking down the wall between it
    // 4. if there are no unvisited neighbors, backtrack to the previous cell you were in and repeat
    // 5. otherwise, repeat from your new cell

    let mut seed = [0u8; 32];
    seed[..4].copy_from_slice(&[1; 4]);
    let mut rng = SmallRng::from_seed(seed);
    // let mut rng = SmallRng::from_entropy();
    let mut visited = vec![false; w.as_usize() * h.as_usize()];
    let mut stack = vec![];

    let mut curr = 0usize;
    visited[curr] = true;

    loop {
        let mut neighbors = vec![];

        if curr % w.as_usize() < w.as_usize() - 1 && !visited[curr + 1] {
            neighbors.push(curr + 1);
        }
        if curr / w.as_usize() < h.as_usize() - 1 && !visited[curr + w.as_usize()] {
            neighbors.push(curr + w.as_usize());
        }
        if curr % w.as_usize() > 0 && !visited[curr - 1] {
            neighbors.push(curr - 1);
        }
        if curr / w.as_usize() > 0 && !visited[curr - w.as_usize()] {
            neighbors.push(curr - w.as_usize());
        }

        if !neighbors.is_empty() {
            let next = *neighbors.choose(&mut rng).unwrap();
            stack.push(curr);
            maze.push((N::from_usize(curr), N::from_usize(next)));
            curr = next;
            visited[curr as usize] = true;
        } else if let Some(prev) = stack.pop() {
            curr = prev;
        } else {
            break;
        }
    }

    maze
}
