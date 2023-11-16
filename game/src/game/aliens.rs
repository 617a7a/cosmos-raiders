use bevy::prelude::*;

use crate::game::ships::Laser;

use super::AtlasIndexable;

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

impl<const P: u32, const I: usize> AtlasIndexable for Alien<P, I> {
    const SPRITE_INDEX: usize = I;
}

impl<const P: u32, const I: usize> Alien<P, I> {
    const VELOCITY: f32 = 100.0; // pixels per second
    const COLLISION_RADIUS: f32 = 16.9705627485; // sqrt(12^2 + 12^2)
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
                }
            }
        }
    }

    pub fn movement_sys(
        time: Res<Time>,
        mut aliens: Query<(&mut Alien<P, I>, &mut Transform)>,
        mut movement: ResMut<AlienMovement>,
    ) {
        let dt = time.delta_seconds();
        let pixels_moved_this_frame = Alien::<P, I>::VELOCITY * dt;

        // aliens start at the left side of the screen, so their first movement
        // is right we keep moving right until any of the alien's
        // abs(x-coord) > 300 then we move down for 12 pixels, then
        // left.
    }
}
