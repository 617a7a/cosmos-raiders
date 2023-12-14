use std::time::Duration;

use crate::game::scoreboard::Score;
use bevy::{
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
};
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;

use crate::game::ships::Laser;

use super::{
    collisions::{collide, CollisionMatrices},
    explosions::Explosion,
    AssetHandles, AtlasIndexable, Spawnable,
};

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

#[derive(Copy, Clone, Resource, PartialEq, Deref, DerefMut)]
pub struct AlienVelocity(pub f32);

impl Default for AlienVelocity {
    fn default() -> Self {
        Self(100.)
    }
}

pub type LowLevelAlien = Alien<10, 4>;
pub type MidLevelAlien = Alien<20, 2>;
pub type HighLevelAlien = Alien<30, 0>;

pub type ForAnyAlien = Or<(
    With<LowLevelAlien>,
    With<MidLevelAlien>,
    With<HighLevelAlien>,
)>;

#[derive(Component)]
pub struct CurrentSpriteIndex {
    original: usize,
    current: usize,
}

impl<const P: u32, const I: usize> Spawnable for Alien<P, I> {
    fn spawn(pos: Vec3, texture_atlas: Handle<TextureAtlas>, commands: &mut Commands) {
        commands.spawn((
            Alien::<P, I>::default(),
            CurrentSpriteIndex {
                original: I,
                current: I,
            },
            SpriteSheetBundle {
                texture_atlas,
                transform: Transform::from_translation(pos),
                sprite: TextureAtlasSprite::new(I),
                ..Default::default()
            },
        ));
    }
}

impl<const P: u32, const I: usize> Alien<P, I> {
    const POINT_VALUE: u32 = P;

    pub fn laser_collision_sys(
        lasers: Query<(Entity, &Transform), With<Laser>>,
        mut commands: Commands,
        mut score: ResMut<Score>,
        asset_handles: Res<AssetHandles>,
        alien_spatial_tree: Res<KDTree2<Self>>,
        alien_sprite_indices: Query<&CurrentSpriteIndex, With<Self>>,
        matrices: Res<CollisionMatrices>,
        mut rumble_requests: EventWriter<GamepadRumbleRequest>,
        gamepads: Res<Gamepads>,
    ) {
        for (laser_entity, laser_transform) in lasers.iter() {
            let laser_pos = laser_transform.translation.truncate();

            if let Some((alien_pos, entity)) = alien_spatial_tree.nearest_neighbour(laser_pos) {
                let alien_entity = match entity {
                    Some(entity) => entity,
                    None => return,
                };

                let alien_sprite_index = match alien_sprite_indices.get(alien_entity) {
                    Ok(sprite_index) => sprite_index,
                    Err(_) => return,
                };

                if collide(
                    &matrices,
                    Laser::SPRITE_INDEX,
                    alien_sprite_index.current,
                    laser_pos,
                    alien_pos,
                ) {
                    commands.entity(alien_entity).despawn();
                    commands.entity(laser_entity).despawn();
                    score.0 += Self::POINT_VALUE;
                    commands.spawn(AudioBundle {
                        source: asset_handles.explosion_sound.clone(),
                        ..Default::default()
                    });
                    Explosion::spawn(
                        alien_pos.extend(0.),
                        asset_handles.texture_atlas.clone(),
                        &mut commands,
                    );
                    for gamepad in gamepads.iter() {
                        rumble_requests.send(GamepadRumbleRequest::Add {
                            gamepad,
                            intensity: GamepadRumbleIntensity::STRONG_MAX,
                            duration: Duration::from_millis(1000),
                        });
                    }
                }
            }
        }
    }
}
const SCREEN_BOUNDARY_X: f32 = 300.0;

pub fn spawn_aliens(commands: &mut Commands, texture_atlas_handle: &Handle<TextureAtlas>) {
    for alien_row in 0..5 {
        let y = 200.0 - (alien_row as f32 * 32.0);
        for alien_col in 0..11 {
            let x = -300.0 + (alien_col as f32 * 32.0);
            match alien_row {
                0 => HighLevelAlien::spawn(
                    Vec3::new(x, y, 0.0),
                    texture_atlas_handle.clone(),
                    commands,
                ),
                1 | 2 => MidLevelAlien::spawn(
                    Vec3::new(x, y, 0.0),
                    texture_atlas_handle.clone(),
                    commands,
                ),
                3 | 4 => LowLevelAlien::spawn(
                    Vec3::new(x, y, 0.0),
                    texture_atlas_handle.clone(),
                    commands,
                ),
                _ => unreachable!(),
            }
        }
    }
}

const DOWN_STEP_Y: f32 = 24.0;

pub fn movement_sys(
    time: Res<Time>,
    mut query: Query<
        &mut Transform,
        Or<(
            With<LowLevelAlien>,
            With<MidLevelAlien>,
            With<HighLevelAlien>,
        )>,
    >,
    mut movement: ResMut<AlienMovement>,
    velocity: Res<AlienVelocity>,
) {
    let dt = time.delta_seconds();
    let pixels_moved_this_frame = **velocity * dt;

    match *movement {
        AlienMovement::Left => {
            for mut transform in query.iter_mut() {
                transform.translation.x -= pixels_moved_this_frame;
            }
            let left_most_position = query
                .iter()
                .map(|t| t.translation.x)
                .fold(f32::INFINITY, f32::min);
            if left_most_position <= -SCREEN_BOUNDARY_X {
                *movement = AlienMovement::Down {
                    pixels_left_to_move: DOWN_STEP_Y,
                    should_move_left_after: false,
                };
            }
        }
        AlienMovement::Right => {
            for mut transform in query.iter_mut() {
                transform.translation.x += pixels_moved_this_frame;
            }
            let right_most_position = query
                .iter()
                .map(|t| t.translation.x)
                .fold(f32::NEG_INFINITY, f32::max);
            if right_most_position >= SCREEN_BOUNDARY_X {
                *movement = AlienMovement::Down {
                    pixels_left_to_move: DOWN_STEP_Y,
                    should_move_left_after: true,
                };
            }
        }
        AlienMovement::Down {
            ref mut pixels_left_to_move,
            should_move_left_after,
        } => {
            for mut transform in query.iter_mut() {
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
    aliens: Query<(), ForAnyAlien>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
) {
    if aliens.iter().len() == 0 {
        spawn_aliens(&mut commands, &asset_handles.texture_atlas)
    }
}

pub fn sprite_alternator_sys(
    mut aliens: Query<(&mut CurrentSpriteIndex, &mut TextureAtlasSprite), ForAnyAlien>,
) {
    for (mut current_sprite_index, mut sprite) in aliens.iter_mut() {
        if current_sprite_index.current == current_sprite_index.original {
            current_sprite_index.current = current_sprite_index.original + 1;
        } else {
            current_sprite_index.current = current_sprite_index.original;
        }
        sprite.index = current_sprite_index.current;
    }
}
