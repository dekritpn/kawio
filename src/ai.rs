use crate::game::{Game, Player};

pub struct AI;

impl AI {
    pub fn get_move(game: &Game) -> Option<u8> {
        let moves = game.legal_moves();
        if moves.is_empty() {
            None
        } else {
            Some(Self::minimax(game, 4)) // depth 4
        }
    }

    fn minimax(game: &Game, depth: i32) -> u8 {
        let moves = game.legal_moves();
        let mut best_move = moves[0];
        let mut best_value = i32::MIN;
        for &mv in &moves {
            let mut new_game = game.clone();
            new_game.make_move(mv);
            let value = -Self::alphabeta(&new_game, depth - 1, i32::MIN, i32::MAX);
            if value > best_value {
                best_value = value;
                best_move = mv;
            }
        }
        best_move
    }

    fn alphabeta(game: &Game, depth: i32, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 || game.is_game_over() {
            return Self::evaluate(game);
        }
        let moves = game.legal_moves();
        if moves.is_empty() {
            let mut new_game = game.clone();
            new_game.pass();
            return -Self::alphabeta(&new_game, depth - 1, -beta, -alpha);
        }
        for &mv in &moves {
            let mut new_game = game.clone();
            new_game.make_move(mv);
            let value = -Self::alphabeta(&new_game, depth - 1, -beta, -alpha);
            if value >= beta {
                return beta;
            }
            alpha = alpha.max(value);
        }
        alpha
    }

    fn evaluate(game: &Game) -> i32 {
        if game.is_game_over() {
            let (black, white) = game.disc_count();
            match game.current_player {
                Player::Black => black as i32 - white as i32,
                Player::White => white as i32 - black as i32,
            }
        } else {
            let my_moves = game.legal_moves().len() as i32;
            let mut opp_game = game.clone();
            opp_game.current_player = opp_game.current_player.opponent();
            let opp_moves = opp_game.legal_moves().len() as i32;
            my_moves - opp_moves
        }
    }
}