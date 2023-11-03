use bevy::{prelude::*, window::PrimaryWindow};

use super::Laser;

#[derive(Component, Default)]
pub struct Player {
    delta_x: f32,
}

impl Player {
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
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Player, &mut Transform, &Handle<TextureAtlas>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    const ACCELERATION: f32 = 70.0; // pixels per second per second
    const MAX_VELOCITY: f32 = 1500.0; // pixels per second
    let dt = time.delta_seconds();

    let Ok(window) = windows.get_single() else {
        return;
    };

    for (mut player, mut trans, atlas_handle) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            player.delta_x -= ACCELERATION * dt;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            player.delta_x += ACCELERATION * dt;
        }

        player.delta_x = player.delta_x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        trans.translation.x += player.delta_x; // change position
        trans.translation.x = trans
            .translation
            .x
            .clamp(-window.width() / 2.0, window.width() / 2.0);
        player.delta_x *= 0.8;

        if keyboard_input.just_pressed(KeyCode::Space) {
            Player::fire_laser(&mut commands, trans.translation, atlas_handle)
        }
    }
}

pub fn laser_movement(
    time: Res<Time>,
    mut query: Query<(Entity, &Laser, &mut Transform)>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    for (entity, _, mut trans) in query.iter_mut() {
        trans.translation.y += 480.0 * dt;

        if trans.translation.y > 240.0 {
            commands.entity(entity).despawn();
        }
    }
}
