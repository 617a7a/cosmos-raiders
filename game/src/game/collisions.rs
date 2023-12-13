use bevy::prelude::*;

const COLLISION_MATRICES: &[u8; 2048] =
    include_bytes!("../../assets/sprite_collision_matrices.bin");
const SPRITE_W: usize = 32;
const SPRITE_H: usize = 32;
const SPRITE_COLS: usize = 8;
const SPRITE_ROWS: usize = 2;
const SPRITE_N: usize = SPRITE_COLS * SPRITE_ROWS;
const BITS_PER_MATRIX: usize = SPRITE_W * SPRITE_H;
const SPRITE_HALF_W: f32 = SPRITE_W as f32 / 2.0;
const SPRITE_HALF_H: f32 = SPRITE_H as f32 / 2.0;

// A 2D array representing the collision matrix of a sprite,
// where the first dimension is the y-coordinate (rows)
// and the second dimension is the x-coordinate (columns).
pub type CollisionMatrix = [[bool; SPRITE_W]; SPRITE_H];

#[derive(Default, Deref, Resource)]
pub struct CollisionMatrices([CollisionMatrix; SPRITE_N]);

pub fn load_collision_matrices() -> CollisionMatrices {
    let mut collision_matrices = [CollisionMatrix::default(); SPRITE_N];

    for (matrix_index, matrix) in collision_matrices.iter_mut().enumerate() {
        for bit_index in 0..BITS_PER_MATRIX {
            // calculate which byte this bit is in and whether it's set
            let byte_index = (matrix_index * BITS_PER_MATRIX + bit_index) / 8;
            let bit_position = bit_index % 8;
            let byte = COLLISION_MATRICES[byte_index];
            let bit_set = (byte & (1 << bit_position)) != 0;

            // Calculating the row (y) and column (x) for this bit
            let row = bit_index / SPRITE_W;
            let column = bit_index % SPRITE_W;
            matrix[row][column] = bit_set;
        }
    }

    CollisionMatrices(collision_matrices)
}

/// Performs a collision check between two sprites - a and b. Returns true if
/// they collide. Uses pre-calculated collision matrices to perform the check.
pub fn collide(matrices: &CollisionMatrices, a: usize, b: usize, a_pos: Vec2, b_pos: Vec2) -> bool {
    let a_matrix = &matrices.0[a];
    let b_matrix = &matrices.0[b];

    // Calculate the bounding box for both sprites based on the center position.
    let a_min = Vec2::new(a_pos.x - SPRITE_HALF_W, a_pos.y - SPRITE_HALF_H);
    let a_max = Vec2::new(a_pos.x + SPRITE_HALF_W, a_pos.y + SPRITE_HALF_H);
    let b_min = Vec2::new(b_pos.x - SPRITE_HALF_W, b_pos.y - SPRITE_HALF_H);
    let b_max = Vec2::new(b_pos.x + SPRITE_HALF_W, b_pos.y + SPRITE_HALF_H);

    // Check if the bounding boxes overlap; if not, there's no collision.
    if a_max.x <= b_min.x || a_min.x >= b_max.x || a_max.y <= b_min.y || a_min.y >= b_max.y {
        return false;
    }

    // Calculate the overlapping rectangle (intersecting area).
    let overlap_x_start = a_min.x.max(b_min.x).ceil() as isize;
    let overlap_y_start = a_min.y.max(b_min.y).ceil() as isize;
    let overlap_x_end = a_max.x.min(b_max.x).floor() as isize;
    let overlap_y_end = a_max.y.min(b_max.y).floor() as isize;

    // Calculate the local sprite matrix coordinates for the top-left corner of the
    // overlapping area.
    let a_local_min_x = (overlap_x_start - (a_pos.x - SPRITE_HALF_W) as isize).max(0) as usize;
    let a_local_min_y = (overlap_y_start - (a_pos.y - SPRITE_HALF_H) as isize).max(0) as usize;
    let b_local_min_x = (overlap_x_start - (b_pos.x - SPRITE_HALF_W) as isize).max(0) as usize;
    let b_local_min_y = (overlap_y_start - (b_pos.y - SPRITE_HALF_H) as isize).max(0) as usize;

    // Determine the width and height of the overlapping area.
    let overlap_width = (overlap_x_end - overlap_x_start).max(0) as usize;
    let overlap_height = (overlap_y_end - overlap_y_start).max(0) as usize;

    // Iterate over the width and height of the overlapping area
    // by traversing the local coordinate space of each sprite's collision matrix.
    for y in 0..overlap_height {
        let a_matrix_y = a_local_min_y + y;
        for x in 0..overlap_width {
            let a_matrix_x = a_local_min_x + x;
            let b_matrix_x = b_local_min_x + x;
            let b_matrix_y = b_local_min_y + y;

            // Perform collision check and return true if a collision is detected.
            if a_matrix[a_matrix_y][a_matrix_x] && b_matrix[b_matrix_y][b_matrix_x] {
                return true;
            }
        }
    }

    // No collision found if the loop completes without returning true.
    false
}
