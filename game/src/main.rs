use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_tokio_tasks::TokioTasksPlugin;
use ui::menu::*;
mod game;
mod ui;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
}

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
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(OnExit(GameState::Menu), remove_menu)
        .add_systems(OnEnter(GameState::InGame), game::setup)
        .add_systems(
            Update,
            handle_menu_interactions.run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            Update,
            (
                game::ships::player_movement,
                game::ships::laser_movement,
                game::aliens::alien_movement,
                game::aliens::ship_laser_collision_detection,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_plugins((
            TokioTasksPlugin::default(),
            ScreenDiagnosticsPlugin {
                timestep: 0.1,
                font: Some("fonts/space_invaders.ttf"),
                ..default()
            },
            ScreenFrameDiagnosticsPlugin,
            FramepacePlugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
