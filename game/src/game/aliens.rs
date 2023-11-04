use bevy::prelude::*;

use crate::game::ships::Laser;

use super::AtlasIndexable;

const VELOCITY: f32 = 100.0; // pixels per second

#[derive(Component, Default)]
pub struct Alien<const POINT_VALUE: u32, const SPRITE_INDEX: usize> {
    movement: AlienMovement,
}

#[derive(Copy, Clone, Component, PartialEq, Default)]
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

impl<const P: u32, const I: usize> AtlasIndexable for Alien<P, I> {
    const SPRITE_INDEX: usize = I;
}

impl<const P: u32, const I: usize> Alien<P, I> {
    fn point_value() -> u32 {
        P
    }

    fn next_pos(&self, mut current: Vec3, ds: f32) -> Vec3 {
        match self.movement {
            AlienMovement::Left | AlienMovement::Right => {
                let is_moving_left = self.movement == AlienMovement::Left;
                if is_moving_left {
                    current.x -= ds;
                } else {
                    current.x += ds;
                }
                current
            }
            AlienMovement::Down {
                mut pixels_left_to_move,
                should_move_left_after,
            } => {
                pixels_left_to_move = f32::max(0.0, pixels_left_to_move - ds);
                current.y -= ds;
                self.movement = if pixels_left_to_move == 0.0 {
                    if should_move_left_after {
                        AlienMovement::Left
                    } else {
                        AlienMovement::Right
                    }
                } else {
                    AlienMovement::Down {
                        pixels_left_to_move,
                        should_move_left_after,
                    }
                };
                current
            }
        }
    }

    pub fn laser_collision_sys(
        laser_query: Query<(Entity, &Laser, &Transform)>,
        collider_query: Query<(Entity, &LowLevelAlien, &Transform)>,
        mut commands: Commands,
    ) {
        for (laser_entity, _, laser_transform) in laser_query.iter() {
            let laser_pos = Vec2::new(laser_transform.translation.x, laser_transform.translation.y);
            for (bug_entity, _, bug_transform) in collider_query.iter() {
                let bug_pos = Vec2::new(bug_transform.translation.x, bug_transform.translation.y);

                if bug_pos.distance(laser_pos) < 24.0 {
                    commands.entity(bug_entity).despawn();
                    commands.entity(laser_entity).despawn();
                }
            }
        }
    }

    pub fn movement_sys(time: Res<Time>, mut query: Query<(&mut LowLevelAlien, &mut Transform)>) {
        let dt = time.delta_seconds();
        let pixels_moved_this_frame = VELOCITY * dt;

        let mut aliens = query.iter_mut().collect::<Vec<_>>();
        let mut positions: Vec<(usize, Vec3)> = Vec::with_capacity(aliens.len());
        for (i, (bug, trans)) in aliens.iter_mut().enumerate() {
            if let AlienMovement::Down {
                mut pixels_left_to_move,
                should_move_left_after,
            } = bug.movement
            {
                pixels_left_to_move = f32::max(0.0, pixels_left_to_move - pixels_moved_this_frame);
                trans.translation.y -= pixels_moved_this_frame;
                bug.movement = if pixels_left_to_move == 0.0 {
                    if should_move_left_after {
                        AlienMovement::Left
                    } else {
                        AlienMovement::Right
                    }
                } else {
                    AlienMovement::Down {
                        pixels_left_to_move,
                        should_move_left_after,
                    }
                };
            }

            if let AlienMovement::Left | AlienMovement::Right = bug.movement {
                let is_moving_left = bug.movement == AlienMovement::Left;
                let mut move_px = pixels_moved_this_frame;
                if is_moving_left {
                    move_px *= -1.0;
                }
                let mut new_pos = trans.translation;
                new_pos.x = trans.translation.x + move_px;

                positions.push((i, new_pos));
            }
        }

        let any_out_of_bounds = positions.iter().any(|(_, pos)| pos.x.abs() > 300.0);
    }
}
