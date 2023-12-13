use bevy::prelude::{Query, Transform, With};
use crate::game::aliens::ForAnyAlien;
use crate::game::ships::PlayerShip;

pub fn game_over_sys(
    aliens: Query<&Transform, ForAnyAlien>,
    ships: Query<&Transform, With<PlayerShip>>
) {
    let ship_pos = ships.get_single().unwrap().translation;
    for alien_pos in aliens.iter() {
        if alien_pos.translation.y < ship_pos.y {
            println!("Game Over!");
            std::process::exit(0);
        }
    }
}