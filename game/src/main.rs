use bevy::{prelude::*, window::WindowResolution};
use bevy_framepace::FramepacePlugin;
#[cfg(feature = "fps_counter")]
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use bevy_tokio_tasks::TokioTasksPlugin;
use game::{
    aliens::{AlienMovement, AlienVelocity, HighLevelAlien, LowLevelAlien, MidLevelAlien},
    collisions::load_collision_matrices,
    scoreboard::Score,
};

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
                resolution: WindowResolution::new(700.0, 700.0),
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((
            AutomaticUpdate::<LowLevelAlien>::new()
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(SpatialStructure::KDTree2),
            AutomaticUpdate::<MidLevelAlien>::new()
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(SpatialStructure::KDTree2),
            AutomaticUpdate::<HighLevelAlien>::new()
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(SpatialStructure::KDTree2),
        ))
        // background color
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_state::<GameState>()
        .insert_resource(AlienMovement::default())
        .insert_resource(Score(0))
        .insert_resource(load_collision_matrices())
        .insert_resource(AlienVelocity::default())
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
                game::aliens::movement_sys,
                game::aliens::LowLevelAlien::laser_collision_sys,
                game::aliens::LowLevelAlien::respawn_sys,
                game::aliens::MidLevelAlien::laser_collision_sys,
                game::aliens::MidLevelAlien::respawn_sys,
                game::aliens::HighLevelAlien::laser_collision_sys,
                game::aliens::HighLevelAlien::respawn_sys,
                game::scoreboard::update_sys,
                game::explosions::explosion_removal_sys,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_plugins((
            TokioTasksPlugin::default(),
            #[cfg(feature = "fps_counter")]
            ScreenDiagnosticsPlugin {
                timestep: 0.1,
                font: Some("fonts/space_invaders.ttf"),
                ..default()
            },
            #[cfg(feature = "fps_counter")]
            ScreenFrameDiagnosticsPlugin,
            FramepacePlugin,
        ))
        .run();
}
