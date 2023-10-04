use std::time::Duration;

use bevy::{prelude::*, window::{PrimaryWindow, WindowMode, PresentMode}, time::common_conditions::on_timer};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_rapier2d::na::ComplexField;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_tokio_tasks::TokioTasksPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmos Raiders".to_string(),
                ..default()
            }),
          ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, (player, bug_movement, laser_movement, bug_zapper))
        .add_plugin(TokioTasksPlugin::default())
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_plugin(FramepacePlugin)
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
    Down { pixels_left_to_move: f32, should_move_left_after: bool },
}

#[derive(Component)]
struct Bug {
    movement: BugMovement,
}

#[derive(Component)]
struct Laser;

fn player(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Player, &mut Transform, &Handle<TextureAtlas>)>,
    windows: Query<&Window, With<PrimaryWindow>>
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
        trans.translation.x = trans.translation.x.clamp(-window.width() / 2.0, window.width() / 2.0);
        player.delta_x *= 0.8;

        if keyboard_input.just_pressed(KeyCode::Space) {
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

fn bug_movement(time: Res<Time>, mut query: Query<(&mut Bug, &mut Transform)>) {
    let dt = time.delta_seconds();
    const VELOCITY: f32 = 100.0; // pixels per second
    let pixels_moved_this_frame = VELOCITY * dt;

    query.for_each_mut(|(mut bug, mut trans)| {
        match bug.movement {
            BugMovement::Left | BugMovement::Right => {
                let is_moving_left = matches!(bug.movement, BugMovement::Left);
                let move_px = if is_moving_left {-pixels_moved_this_frame} else {pixels_moved_this_frame};
                let new_position = trans.translation.x + move_px;
                if new_position.abs() > 300.0 {
                    bug.movement = BugMovement::Down {
                        pixels_left_to_move: 32.0,
                        should_move_left_after: !is_moving_left,
                    };
                } else {
                    trans.translation.x = new_position;
                }
            },
            BugMovement::Down { pixels_left_to_move: n, should_move_left_after: next_left } => {
                let new_n = f32::max(0.0, n - pixels_moved_this_frame);
                trans.translation.y -= pixels_moved_this_frame;
                bug.movement = if new_n == 0.0 {
                    if next_left {BugMovement::Left} else {BugMovement::Right}
                } else {
                    BugMovement::Down { pixels_left_to_move: new_n, should_move_left_after: next_left }
                };
            }
        }
    });
}

fn laser_movement(time: Res<Time>, mut query: Query<(Entity, &Laser, &mut Transform)>, mut commands: Commands) {
    let dt = time.delta_seconds();
    for (entity, _, mut trans) in query.iter_mut() {
        trans.translation.y += 480.0 * dt;

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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    // mut framepace_settings: ResMut<FramepaceSettings>,
) {
    // framepace_settings.limiter = Limiter::from_framerate(24.0);
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
