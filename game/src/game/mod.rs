pub mod aliens;
pub mod shields;
pub mod ships;

use bevy::prelude::*;

use self::{aliens::Alien, ships::Player};

#[derive(Component)]
pub struct Laser;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    // mut framepace_settings: ResMut<FramepaceSettings>,
) {
    let texture_handle = asset_server.load("spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 3, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Spawn the player
    commands.spawn((
        Player::default(),
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -220.0, 0.0)),
            sprite: TextureAtlasSprite::new(0),
            ..Default::default()
        },
    ));

    // Spawn rows of enemies
    for bug_row in 0..2 {
        let y = 200.0 - (bug_row as f32 * 30.0);
        for bug_col in 0..11 {
            let x = -300.0 + (bug_col as f32 * 30.0);
            commands.spawn((
                Alien::default(),
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                    sprite: TextureAtlasSprite::new(1),
                    ..Default::default()
                },
            ));
        }
    }
}
