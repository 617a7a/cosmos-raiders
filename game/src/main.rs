use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_tokio_tasks::TokioTasksPlugin;
use game::{aliens::AlienMovement, Score};
mod game;
mod ui;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    MainMenu,
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
        // background color
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_state::<GameState>()
        .insert_resource(AlienMovement::default())
        .insert_resource(Score::default())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2dBundle::default());
        })
        // main menu systems
        .add_systems(OnEnter(GameState::MainMenu), ui::menu::setup_sys)
        .add_systems(OnExit(GameState::MainMenu), ui::menu::remove_menu_sys)
        .add_systems(
            Update,
            ui::menu::handle_menu_interactions_sys.run_if(in_state(GameState::MainMenu)),
        )
        // game systems
        .add_systems(OnEnter(GameState::InGame), game::setup_sys)
        .add_systems(
            Update,
            (
                game::ships::PlayerShip::movement_sys,
                game::ships::Laser::movement_sys,
                game::aliens::LowLevelAlien::movement_sys,
                game::aliens::LowLevelAlien::laser_collision_sys,
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
