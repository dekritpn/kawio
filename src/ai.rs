use crate::game::{Game, Move};
use crate::mcts::MCTS;

/// Configuration for the MCTS AI.
#[derive(Clone, Debug)]
pub struct AiConfig {
    pub simulations: u32,
    pub exploration_constant: f64,
    pub temperature: f64,
    pub rng_seed: Option<u64>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            simulations: 100,
            exploration_constant: 1.414,
            temperature: 0.0,
            rng_seed: None,
        }
    }
}

/// MCTS-based AI that maintains state for tree reuse.
pub struct MctsAi {
    config: AiConfig,
    mcts: Option<MCTS>,
}

impl MctsAi {
    /// Creates a new MCTS AI with the given configuration.
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            mcts: None,
        }
    }

    /// Notifies the AI that a move was made, allowing tree reuse.
    pub fn make_move(&mut self, mv: Move) {
        if let Some(ref mut mcts) = self.mcts {
            if !mcts.advance_root(mv) {
                // If advance failed, reset tree
                self.mcts = None;
            }
        }
    }

    /// Gets the best move for the current game state.
    /// Reuses the MCTS tree if possible.
    pub fn get_move(&mut self, game: &Game) -> Option<Move> {
        let moves = game.legal_moves();
        if moves.is_empty() {
            Some(Move::Pass)
        } else {
            // Ensure MCTS exists and matches current game
            if self.mcts.is_none() || *self.mcts.as_ref().unwrap().root_game() != *game {
                self.mcts = Some(MCTS::new(game.clone(), self.config.exploration_constant, self.config.rng_seed));
            }
            Some(self.mcts.as_mut().unwrap().search(self.config.simulations, self.config.temperature).best_move)
        }
    }
}

// Legacy static API for backward compatibility
pub struct AI;

impl AI {
    pub fn get_move(game: &Game) -> Option<Move> {
        let mut ai = MctsAi::new(AiConfig::default());
        ai.get_move(game)
    }
}
