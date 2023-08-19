use std::time::Duration;

use bevy::{prelude::*, window::{PrimaryWindow, WindowMode}, time::common_conditions::on_timer, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_tokio_tasks::TokioTasksPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmos Raiders".to_string(),
                // present_mode: PresentMode::Immediate,
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
          ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (player, bug_movement.run_if(on_timer(Duration::from_secs_f32(0.1))), laser_movement, bug_zapper))
        .add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default(), TokioTasksPlugin::default(), FramepacePlugin))
        .run();
}

#[derive(Component)]
struct Player {
    delta_x: f32,
}

#[derive(Copy, Clone, Component)]
enum BugMovement {
    Left,
    Right,
    Down { n: f32, next_left: bool },
}

#[derive(Component)]
struct Bug {
    movement: BugMovement,
}

#[derive(Component)]
struct Laser;

fn player(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&mut Player, &mut Transform, &Handle<TextureAtlas>)>,
    windows: Query<&Window, With<PrimaryWindow>>
) {
    const ACCELERATION: f32 = 1.0;
    const MAX_VELOCITY: f32 = 16.0;
    
    let Ok(window) = windows.get_single() else {
        return;
    };

    for (mut player, mut trans, atlas_handle) in query.iter_mut() {
        let mut firing = false;

        if keyboard_input.pressed(KeyCode::Left) {
            player.delta_x -= ACCELERATION;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            player.delta_x += ACCELERATION;
        }
        if keyboard_input.just_pressed(KeyCode::Space) {
            firing = true;
        }

        // Apply movement deltas
        player.delta_x = player.delta_x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        trans.translation.x += player.delta_x;
        trans.translation.x = trans.translation.x.clamp(-window.width() / 2.0, window.width() / 2.0);

        // Decelerate
        player.delta_x *= 0.75;

        if firing {
            commands
                .spawn((Laser {}, SpriteSheetBundle {
                    texture_atlas: atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        trans.translation.x,
                        trans.translation.y + 24.0,
                        0.0,
                    )),
                    sprite: TextureAtlasSprite::new(2),
                    ..Default::default()
                }));
        }
    }
}

fn bug_movement(mut query: Query<(&mut Bug, &mut Transform)>) {
    for (mut bug, mut trans) in query.iter_mut() {
        let mut new_movement = bug.movement;
        match bug.movement {
            BugMovement::Left => {
                trans.translation.x -= 2.0;
                if trans.translation.x < -300.0 {
                    new_movement = BugMovement::Down {
                        n: 12.0,
                        next_left: false,
                    };
                }
            }
            BugMovement::Right => {
                trans.translation.x += 2.0;
                if trans.translation.x > 300.0 {
                    new_movement = BugMovement::Down {
                        n: 12.0,
                        next_left: true,
                    };
                }
            }
            BugMovement::Down { n, next_left } => {
                trans.translation.y -= 2.0;
                new_movement = BugMovement::Down {
                    n: n - 1.0,
                    next_left,
                };
                if n < 1.0 {
                    new_movement = if next_left {
                        BugMovement::Left
                    } else {
                        BugMovement::Right
                    };
                }
            }
        }
        bug.movement = new_movement;
    }
}

fn laser_movement(mut query: Query<(Entity, &Laser, &mut Transform)>, mut commands: Commands) {
    for (entity, _, mut trans) in query.iter_mut() {
        trans.translation += Vec3::new(0.0, 4.0, 0.0);

        if trans.translation.y > 240.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn bug_zapper(
    laser_query: Query<(Entity, &Laser, &Transform)>,
    collider_query: Query<(Entity, &Bug, &Transform)>,
    mut commands: Commands,
) {
    for (entity, _, trans) in laser_query.iter() {
        let laser_pos = Vec2::new(trans.translation.x, trans.translation.y);
        for (bug_entity, _, bug_transform) in collider_query.iter() {
            let bug_pos = Vec2::new(bug_transform.translation.x, bug_transform.translation.y);

            if bug_pos.distance(laser_pos) < 24.0 {
                commands.entity(bug_entity).despawn();
                commands.entity(entity).despawn();
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut framepace_settings: ResMut<FramepaceSettings>,
) {
    framepace_settings.limiter = Limiter::from_framerate(240.0);
    // Setup the sprite sheet
    let texture_handle = asset_server.load("spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 3, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn(Camera2dBundle::default());

    // Spawn the player
    commands
        .spawn((Player { delta_x: 0.0 }, SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -220.0, 0.0)),
            sprite: TextureAtlasSprite::new(0),
            ..Default::default()
        }));

    // Spawn rows of enemies
    for bug_row in 0..4 {
        let y = 200.0 - (bug_row as f32 * 30.0);
        for bug_col in 0..20 {
            let x = -300.0 + (bug_col as f32 * 30.0);
            commands
                .spawn((Bug {
                    movement: if bug_row % 2 == 0 {
                        BugMovement::Left
                    } else {
                        BugMovement::Right
                    },
                }, SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                    sprite: TextureAtlasSprite::new(1),
                    ..Default::default()
                }));
        }
    }
}
