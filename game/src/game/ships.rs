use bevy::{prelude::*, window::PrimaryWindow};

use super::AtlasIndexable;

const SHIP_ACCELERATION: f32 = 70.0; // pixels per second per second
const SHIP_MAX_VELOCITY: f32 = 1500.0; // pixels per second

#[derive(Component, Default)]
pub struct PlayerShip {
    delta_x: f32,
}

impl AtlasIndexable for PlayerShip {
    const SPRITE_INDEX: usize = 0;
}

impl PlayerShip {
    fn fire_laser(commands: &mut Commands, player_pos: Vec3, atlas_handle: &Handle<TextureAtlas>) {
        commands.spawn((
            Laser {},
            SpriteSheetBundle {
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    player_pos.x,
                    player_pos.y + 24.0,
                    0.0,
                )),
                sprite: TextureAtlasSprite::new(2),
                ..Default::default()
            },
        ));
    }

    fn accelerate_left(&mut self, dt: f32) {
        self.delta_x -= SHIP_ACCELERATION * dt;
    }

    fn accelerate_right(&mut self, dt: f32) {
        self.delta_x += SHIP_ACCELERATION * dt;
    }

    fn apply_delta_x(&mut self, pos: &mut Vec3, window_width: f32) {
        self.delta_x = self.delta_x.clamp(-SHIP_MAX_VELOCITY, SHIP_MAX_VELOCITY);
        pos.x += self.delta_x; // change position
        pos.x = pos.x.clamp(-window_width / 2.0, window_width / 2.0);
        self.delta_x *= 0.8;
    }

    pub fn movement_sys(
        keyboard_input: Res<Input<KeyCode>>,
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(&mut PlayerShip, &mut Transform, &Handle<TextureAtlas>)>,
        windows: Query<&Window, With<PrimaryWindow>>,
    ) {
        let dt = time.delta_seconds();

        let Ok(window) = windows.get_single() else {
            return;
        };

        for (mut player, mut trans, atlas_handle) in query.iter_mut() {
            if keyboard_input.pressed(KeyCode::Left) {
                player.accelerate_left(dt)
            }

            if keyboard_input.pressed(KeyCode::Right) {
                player.accelerate_right(dt)
            }

            player.apply_delta_x(&mut trans.translation, window.width());

            if keyboard_input.just_pressed(KeyCode::Space) {
                PlayerShip::fire_laser(&mut commands, trans.translation, atlas_handle)
            }
        }
    }
}

const LASER_VELOCITY: f32 = 480.0; // pixels per second

#[derive(Component)]
pub struct Laser;

impl Laser {
    /// Update the laser's position according to the frame delta time
    fn update_position(&mut self, dt: f32, pos: &mut Vec3) {
        pos.y += LASER_VELOCITY * dt;
    }

    /// Despawn the laser if it goes off the top of the screen
    fn despawn_if_needed(&self, pos: &mut Vec3, commands: &mut Commands, entity: Entity) {
        if pos.y > 240.0 {
            commands.entity(entity).despawn();
        }
    }

    pub fn movement_sys(
        time: Res<Time>,
        mut query: Query<(Entity, &mut Laser, &mut Transform)>,
        mut commands: Commands,
    ) {
        let dt = time.delta_seconds();
        for (entity, mut laser, mut trans) in query.iter_mut() {
            let pos = &mut trans.translation;
            laser.update_position(dt, pos);
            laser.despawn_if_needed(pos, &mut commands, entity)
        }
    }
}
