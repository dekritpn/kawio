use crate::game::{Game, Player};
use rand::Rng;

pub struct AI;

impl AI {
    pub fn get_move(game: &Game) -> Option<u8> {
        let moves = game.legal_moves();
        if moves.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..moves.len());
            Some(moves[index])
        }
    }
}