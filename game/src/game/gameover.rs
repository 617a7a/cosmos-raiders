use crate::game::aliens::ForAnyAlien;
use crate::game::ships::PlayerShip;
use bevy::prelude::*;

use super::scoreboard::Score;

pub fn game_over_sys(
    aliens: Query<&Transform, ForAnyAlien>,
    ships: Query<&Transform, With<PlayerShip>>,
    score: Res<Score>,
) {
    let ship_pos = ships.get_single().unwrap().translation;
    for alien_pos in aliens.iter() {
        if alien_pos.translation.y < ship_pos.y {
            println!("Game Over - score {}", score.0);
            std::process::exit(0);
        }
    }
}
