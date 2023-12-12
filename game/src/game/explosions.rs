use bevy::prelude::*;

use super::AtlasIndexable;

impl AtlasIndexable for Explosion {
    const SPRITE_INDEX: usize = 12;
}

#[derive(Component)]
pub struct Explosion {
    msecs_till_drop: f32,
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            msecs_till_drop: 50.,
        }
    }
}

pub fn explosion_removal_sys(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Explosion)>,
) {
    for (entity, mut explosion) in query.iter_mut() {
        explosion.msecs_till_drop -= time.delta().as_millis() as f32;
        if explosion.msecs_till_drop <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
