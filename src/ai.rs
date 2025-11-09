use crate::game::Game;
use crate::mcts::MCTS;

pub struct AI;

impl AI {
    pub fn get_move(game: &Game) -> Option<u8> {
        let moves = game.legal_moves();
        if moves.is_empty() {
            None
        } else {
            let mut mcts = MCTS::new(game.clone());
            Some(mcts.search(1000))
        }
    }
}
