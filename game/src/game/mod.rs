pub mod aliens;
pub mod shields;
pub mod ships;

use bevy::prelude::*;

use self::{aliens::LowLevelAlien, ships::PlayerShip};

#[derive(Resource, Default)]
pub struct Score(pub u32);

pub trait Spawnable: Component {
    fn spawn(pos: Vec3, texture_atlas: Handle<TextureAtlas>, commands: &mut Commands);
}

pub trait AtlasIndexable: Component {
    /// The index of the sprite in the texture atlas.
    const SPRITE_INDEX: usize;
}

impl<T: AtlasIndexable + Default> Spawnable for T {
    fn spawn(pos: Vec3, texture_atlas: Handle<TextureAtlas>, commands: &mut Commands) {
        commands.spawn((
            T::default(),
            SpriteSheetBundle {
                texture_atlas,
                transform: Transform::from_translation(pos),
                sprite: TextureAtlasSprite::new(Self::SPRITE_INDEX),
                ..Default::default()
            },
        ));
    }
}

pub fn setup_sys(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 3, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    PlayerShip::spawn(
        Vec3::new(0.0, -220.0, 0.0),
        texture_atlas_handle.clone(),
        &mut commands,
    );

    for alien_row in 0..2 {
        let y = 200.0 - (alien_row as f32 * 30.0);
        for alien_col in 0..11 {
            let x = -300.0 + (alien_col as f32 * 30.0);
            LowLevelAlien::spawn(
                Vec3::new(x, y, 0.0),
                texture_atlas_handle.clone(),
                &mut commands,
            );
        }
    }
}
