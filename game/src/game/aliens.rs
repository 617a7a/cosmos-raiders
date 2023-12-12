use bevy::prelude::*;

use crate::game::{ships::Laser, Score};

use super::{AssetHandles, AtlasIndexable, Spawnable};

#[derive(Component, Default)]
pub struct Alien<const POINT_VALUE: u32, const SPRITE_INDEX: usize>;

#[derive(Copy, Clone, Resource, PartialEq, Default)]
pub enum AlienMovement {
    Left,
    #[default]
    Right,
    Down {
        pixels_left_to_move: f32,
        should_move_left_after: bool,
    },
}

/// A low-level alien is an alien with 10 points and sprite index 1.
pub type LowLevelAlien = Alien<10, 1>;
/// A mid-level alien is an alien with 20 points and sprite index 2.
pub type MidLevelAlien = Alien<20, 2>;
/// A high-level alien is an alien with 30 points and sprite index 3.
pub type HighLevelAlien = Alien<30, 3>;

impl<const P: u32, const I: usize> AtlasIndexable for Alien<P, I> {
    const SPRITE_INDEX: usize = I;
}

impl<const P: u32, const I: usize> Alien<P, I> {
    const VELOCITY: f32 = 100.0; // pixels per second
    const COLLISION_RADIUS: f32 = 2.0 * 12.0;
    const DOWN_STEP_Y: f32 = 24.0;
    const POINT_VALUE: u32 = P;

    fn next_pos(&self, movement: AlienMovement, mut current: Vec3, ds: f32) -> Vec3 {
        match movement {
            AlienMovement::Left | AlienMovement::Right => {
                if movement == AlienMovement::Left {
                    current.x -= ds;
                } else {
                    current.x += ds;
                }
                current
            }
            AlienMovement::Down {
                pixels_left_to_move,
                ..
            } => {
                current.y -= f32::max(0.0, pixels_left_to_move - ds);
                current
            }
        }
    }

    pub fn laser_collision_sys(
        lasers: Query<(Entity, &Laser, &Transform)>,
        aliens: Query<(Entity, &Alien<P, I>, &Transform)>,
        mut commands: Commands,
        mut score: ResMut<Score>,
        asset_handles: Res<AssetHandles>,
    ) {
        for (laser_entity, _, laser_transform) in lasers.iter() {
            for (alien_entity, _, alien_transform) in aliens.iter() {
                if alien_transform
                    .translation
                    .distance(laser_transform.translation)
                    < Alien::<P, I>::COLLISION_RADIUS
                {
                    commands.entity(alien_entity).despawn();
                    commands.entity(laser_entity).despawn();
                    score.0 += Alien::<P, I>::POINT_VALUE;
                    commands.spawn(AudioBundle {
                        source: asset_handles.explosion_sound.clone(),
                        ..Default::default()
                    });
                }
            }
        }
    }

    pub fn movement_sys(
        time: Res<Time>,
        mut query: Query<(&mut Alien<P, I>, &mut Transform)>,
        mut movement: ResMut<AlienMovement>,
    ) {
        let dt = time.delta_seconds();
        let pixels_moved_this_frame = Self::VELOCITY * dt;

        match *movement {
            AlienMovement::Left => {
                for (_, mut transform) in query.iter_mut() {
                    transform.translation.x -= pixels_moved_this_frame;
                }
                let left_most_position = query
                    .iter()
                    .map(|(_, t)| t.translation.x)
                    .fold(f32::INFINITY, f32::min);
                if left_most_position <= -SCREEN_BOUNDARY_X {
                    *movement = AlienMovement::Down {
                        pixels_left_to_move: Self::DOWN_STEP_Y,
                        should_move_left_after: false,
                    };
                }
            }
            AlienMovement::Right => {
                for (_, mut transform) in query.iter_mut() {
                    transform.translation.x += pixels_moved_this_frame;
                }
                let right_most_position = query
                    .iter()
                    .map(|(_, t)| t.translation.x)
                    .fold(f32::NEG_INFINITY, f32::max);
                if right_most_position >= SCREEN_BOUNDARY_X {
                    *movement = AlienMovement::Down {
                        pixels_left_to_move: Self::DOWN_STEP_Y,
                        should_move_left_after: true,
                    };
                }
            }
            AlienMovement::Down {
                ref mut pixels_left_to_move,
                should_move_left_after,
            } => {
                for (_, mut transform) in query.iter_mut() {
                    // Move the alien down
                    let move_down = f32::min(*pixels_left_to_move, pixels_moved_this_frame);
                    transform.translation.y -= move_down;
                }
                *pixels_left_to_move -= pixels_moved_this_frame;

                // If the aliens have finished moving down, change horizontal direction
                if *pixels_left_to_move <= 0.0 {
                    *movement = if should_move_left_after {
                        AlienMovement::Left
                    } else {
                        AlienMovement::Right
                    };
                }
            }
        }
    }

    pub fn respawn_sys(
        aliens: Query<(Entity, &Alien<P, I>, &Transform)>,
        mut commands: Commands,
        asset_handles: Res<AssetHandles>,
    ) {
        if aliens.iter().len() == 0 {
            spawn_aliens(&mut commands, &asset_handles.texture_atlas)
        }
    }
}
const SCREEN_BOUNDARY_X: f32 = 300.0;

pub fn spawn_aliens(commands: &mut Commands, texture_atlas_handle: &Handle<TextureAtlas>) {
    for alien_row in 0..2 {
        let y = 200.0 - (alien_row as f32 * 30.0);
        for alien_col in 0..11 {
            let x = -300.0 + (alien_col as f32 * 30.0);
            LowLevelAlien::spawn(Vec3::new(x, y, 0.0), texture_atlas_handle.clone(), commands);
        }
    }
}
