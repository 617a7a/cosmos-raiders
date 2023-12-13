use lodepng::{decode_memory, encode_file, ColorType, Image, RGBA};

const SPRITE_W: usize = 32;
const SPRITE_H: usize = 32;
const SPRITE_COLS: usize = 8;
const SPRITE_ROWS: usize = 2;
const SPRITES_DATA: &[u8] = include_bytes!("assets/sprites.png");

fn main() {
    // only rerun the build script if the sprite sheet changes
    println!("cargo:rerun-if-changed=assets/sprites.png");
    // the build script generates collision matrices for the sprites before
    // compiling the game
    let png = decode_memory(SPRITES_DATA, ColorType::RGBA, 8).unwrap();
    let bmp = match png {
        Image::RGBA(data) => data,
        _ => panic!("unexpected color type"),
    };

    // check that the sprite sheet is the correct size
    assert_eq!(bmp.height, SPRITE_H * SPRITE_ROWS);
    assert_eq!(bmp.width, SPRITE_W * SPRITE_COLS);

    let mut sprite_pixel_data = [[[false; SPRITE_H]; SPRITE_W]; SPRITE_ROWS * SPRITE_COLS];

    // split the sprite sheet into individual sprites
    // bmp.buffer is a vec of RGBA values, left-to-right, top-to-bottom. we need to
    // split this up into the individual sprite grids

    // then convert the sprite data into a matrix of booleans, where true means the
    // pixel is on and false means it's off (transparent)
    for row in 0..SPRITE_ROWS {
        for col in 0..SPRITE_COLS {
            for y in 0..SPRITE_H {
                for x in 0..SPRITE_W {
                    let bmp_index = (row * SPRITE_H + y) * bmp.width + col * SPRITE_W + x;
                    sprite_pixel_data[row * SPRITE_COLS + col][x][y] = bmp.buffer[bmp_index].a != 0;
                }
            }
        }
    }

    // optimise the matrix by removing internal pixels (pixels that are surrounded
    // on top, bottom, left and right by other pixels)
    // this is effectively edge detection, going from a filled shape to an outline
    // of the shape
    for sprite_matrix in sprite_pixel_data.iter_mut() {
        // initialize a new matrix to store the result
        let mut new_matrix = [[false; SPRITE_H]; SPRITE_W];
        // loop over each pixel in the sprite by its width and height
        for x in 0..SPRITE_W {
            for y in 0..SPRITE_H {
                // check if the current pixel is set to true (pixel is on)
                if sprite_matrix[x][y] {
                    // assume the pixel is surrounded by pixels on all sides
                    let mut surrounded = true;
                    // check the pixel to the left if it's within bounds
                    if x > 0 {
                        surrounded &= sprite_matrix[x - 1][y];
                    }
                    // check the pixel to the right if it's within bounds
                    if x < SPRITE_W - 1 {
                        surrounded &= sprite_matrix[x + 1][y];
                    }
                    // check the pixel above if it's within bounds
                    if y > 0 {
                        surrounded &= sprite_matrix[x][y - 1];
                    }
                    // check the pixel below if it's within bounds
                    if y < SPRITE_H - 1 {
                        surrounded &= sprite_matrix[x][y + 1];
                    }
                    // if the pixel is not surrounded on all sides, mark it as true in the
                    // new_matrix
                    if !surrounded {
                        new_matrix[x][y] = true;
                    }
                }
            }
        }
        // update the original sprite matrix with the new matrix
        *sprite_matrix = new_matrix;
    }

    save_png_for_debugging(&sprite_pixel_data);

    // convert the matrices into a flat vector of booleans
    let sprite_matrices_flat: Vec<bool> = sprite_pixel_data
        .iter()
        .flat_map(|matrix| matrix.iter().flatten())
        .copied()
        .collect();

    // convert the flat vector of booleans into a flat vector of bytes
    let final_bytes: Vec<u8> = sprite_matrices_flat
        // we want to deal with 8 bits at a time, so we need to chunk the vector
        .chunks(8)
        // convert each chunk of 8 booleans into a byte
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                // fold allows us to iterate over the chunk and accumulate a value
                .fold(0, |acc, (i, &on)| acc | ((on as u8) << i))
        })
        .collect();

    // save to assets/sprite_collision_matrices.bin
    std::fs::write("assets/sprite_collision_matrices.bin", final_bytes).unwrap();
}

fn save_png_for_debugging(
    sprite_pixel_data: &[[[bool; SPRITE_H]; SPRITE_W]; SPRITE_COLS * SPRITE_ROWS],
) {
    // for debugging purposes, re-export the sprite matrices as a png using black
    // for on and white for off
    let mut sprite_matrices_png = vec![0; SPRITE_W * SPRITE_H * SPRITE_COLS * SPRITE_ROWS * 4];
    for row in 0..SPRITE_ROWS {
        for col in 0..SPRITE_COLS {
            for y in 0..SPRITE_H {
                for x in 0..SPRITE_W {
                    let png_index = (row * SPRITE_H + y) * SPRITE_W * SPRITE_COLS * 4
                        + col * SPRITE_W * 4
                        + x * 4;
                    let pixel = if sprite_pixel_data[row * SPRITE_COLS + col][x][y] {
                        RGBA {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        }
                    } else {
                        RGBA {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 0,
                        }
                    };
                    sprite_matrices_png[png_index] = pixel.r;
                    sprite_matrices_png[png_index + 1] = pixel.g;
                    sprite_matrices_png[png_index + 2] = pixel.b;
                    sprite_matrices_png[png_index + 3] = pixel.a;
                }
            }
        }
    }

    // save to assets/sprite_collision_matrices.png
    encode_file(
        "assets/matrices.png",
        &sprite_matrices_png,
        256,
        64,
        ColorType::RGBA,
        8,
    )
    .unwrap();
}
