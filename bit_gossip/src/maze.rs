//! contains functions to build a maze.
//!
//! This modules is not related to the main functionality of the library.
//! It is used to demonstrate the library's capabilities in the examples.
//!
//! You're still free to use these functions in your own projects.

use crate::graph::U16orU32;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, RngCore, SeedableRng};

/// Builds a maze of the given width and height.
///
/// Returns a list of pairs of cells that are connected.
pub fn build_maze<N: U16orU32>(w: N, h: N) -> Vec<(N, N)> {
    build_maze_with_rng(w, h, &mut StdRng::from_entropy())
}

/// Given width and height, build a maze with the provided seed.
///
/// Returns a list of pairs of cells that are connected.
///
/// Uses [StdRng] with the provided seed.
pub fn build_maze_from_seed<N: U16orU32>(w: N, h: N, seed: [u8; 32]) -> Vec<(N, N)> {
    build_maze_with_rng(w, h, &mut StdRng::from_seed(seed))
}

/// Given width and height, build a maze with the provided Rng.
///
/// Returns a list of pairs of cells that are connected.
pub fn build_maze_with_rng<N: U16orU32, R: RngCore>(w: N, h: N, rng: &mut R) -> Vec<(N, N)> {
    let w_usize = w.as_usize();
    let h_usize = h.as_usize();

    let mut maze = Vec::with_capacity(w_usize * h_usize);

    // create maze with following algorithm, in a loop, not recursive:
    // 1. choose a cell to begin
    // 2. from your current cell, choose a random neighbor that you haven’t visited yet
    // 3. move to the chosen neighbor, knocking down the wall between it
    // 4. if there are no unvisited neighbors, backtrack to the previous cell you were in and repeat
    // 5. otherwise, repeat from your new cell

    // let mut rng = SmallRng::from_entropy();
    let mut visited = vec![false; w_usize * h_usize];
    let mut stack = vec![];

    let mut curr = rng.gen_range(0..(w_usize * h_usize));
    visited[curr] = true;

    let mut depth = 0;
    let max_depth = (w_usize + h_usize) / 2;

    loop {
        let mut neighbors = vec![];

        if curr % w_usize < w_usize - 1 && !visited[curr + 1] {
            neighbors.push(curr + 1);
        }
        if curr / w_usize < h_usize - 1 && !visited[curr + w_usize] {
            neighbors.push(curr + w_usize);
        }
        if curr % w_usize > 0 && !visited[curr - 1] {
            neighbors.push(curr - 1);
        }
        if curr / w_usize > 0 && !visited[curr - w_usize] {
            neighbors.push(curr - w_usize);
        }

        if !neighbors.is_empty() && depth < max_depth {
            let next = *neighbors.choose(rng).unwrap();
            stack.push(curr);
            maze.push((N::from_usize(curr), N::from_usize(next)));
            curr = next;
            visited[curr as usize] = true;
            depth += 1;
        } else if let Some(prev) = stack.pop() {
            curr = prev;
            depth = 0;
        } else {
            break;
        }
    }

    maze
}
