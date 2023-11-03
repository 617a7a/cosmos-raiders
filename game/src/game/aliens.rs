use bevy::prelude::*;

use super::Laser;

#[derive(Component)]
pub struct Alien {
    movement: AlienMovement,
}

impl Default for Alien {
    fn default() -> Self {
        Self {
            movement: AlienMovement::Right,
        }
    }
}

#[derive(Copy, Clone, Component, PartialEq)]
pub enum AlienMovement {
    Left,
    Right,
    Down {
        pixels_left_to_move: f32,
        should_move_left_after: bool,
    },
}

pub fn alien_movement(time: Res<Time>, mut query: Query<(&mut Alien, &mut Transform)>) {
    let dt = time.delta_seconds();
    const VELOCITY: f32 = 100.0; // pixels per second
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

pub fn alien_collision_detection(
    laser_query: Query<(Entity, &Laser, &Transform)>,
    collider_query: Query<(Entity, &Alien, &Transform)>,
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
