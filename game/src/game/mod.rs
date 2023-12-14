pub mod aliens;
pub mod collisions;
pub mod explosions;
pub mod gameover;
pub mod scoreboard;
pub mod shields;
pub mod ships;

use bevy::prelude::*;

use self::{aliens::spawn_aliens, scoreboard::spawn_scoreboard, ships::PlayerShip};

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

#[derive(Resource)]
pub struct AssetHandles {
    pub texture_atlas: Handle<TextureAtlas>,
    pub font: Handle<Font>,
    pub shoot_sound: Handle<AudioSource>,
    pub explosion_sound: Handle<AudioSource>,
}

pub fn setup_sys(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 8, 2, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let font = asset_server.load("fonts/space_invaders.ttf");

    commands.insert_resource(AssetHandles {
        texture_atlas: texture_atlas_handle.clone(),
        font: font.clone(),
        shoot_sound: asset_server.load("sfx/shoot.ogg"),
        explosion_sound: asset_server.load("sfx/explosion.ogg"),
    });

    PlayerShip::spawn(
        Vec3::new(0.0, -130.0, 0.0),
        texture_atlas_handle.clone(),
        &mut commands,
    );

    spawn_aliens(&mut commands, &texture_atlas_handle);
    spawn_scoreboard(&mut commands, font);
}
