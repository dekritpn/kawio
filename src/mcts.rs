use crate::game::{Game, Player, Move};
use rand::prelude::*;

/// Telemetry data from MCTS search.
#[derive(Debug, Clone)]
pub struct Telemetry {
    pub total_simulations: u32,
    pub average_depth: f64,
    pub chosen_q_value: f64,
    pub visit_distribution: Vec<u32>,
}

/// Result of MCTS search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub telemetry: Telemetry,
}



struct Node {
    visits: u32,
    wins: u32,
    parent: Option<usize>,
    children: Vec<usize>,
    game: Game,
    move_from_parent: Option<Move>,
}

impl Node {
    fn new(game: Game, parent: Option<usize>, move_from_parent: Option<Move>) -> Self {
        Node {
            visits: 0,
            wins: 0,
            parent,
            children: Vec::new(),
            game,
            move_from_parent,
        }
    }

    fn uct_value(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            (self.wins as f64 / self.visits as f64)
                + exploration_constant * (parent_visits as f64).ln() / (self.visits as f64)
        }
    }
}

pub struct MCTS {
    nodes: Vec<Node>,
    exploration_constant: f64,
    root_index: usize,
    rng: StdRng,
}

impl MCTS {
    pub fn new(game: Game, exploration_constant: f64, seed: Option<u64>) -> Self {
        let rng = if let Some(s) = seed {
            StdRng::seed_from_u64(s)
        } else {
            StdRng::from_entropy()
        };
        let root_node = Node::new(game, None, None);
        MCTS {
            nodes: vec![root_node],
            exploration_constant,
            root_index: 0,
            rng,
        }
    }

    pub fn search(&mut self, iterations: u32, temperature: f64) -> SearchResult {
        for _ in 0..iterations {
            let leaf_index = self.select_leaf();
            let expanded_children = self.expand_node(leaf_index);
            for child_index in expanded_children {
                let outcome = self.simulate(child_index);
                self.backpropagate(child_index, outcome);
            }
        }
        let best_move = self.best_move(temperature);
        let telemetry = self.compute_telemetry();
        SearchResult { best_move, telemetry }
    }

    fn select_leaf(&self) -> usize {
        let mut current_index = self.root_index;
        while !self.nodes[current_index].children.is_empty() {
            let parent_visits = self.nodes[current_index].visits;
            current_index = *self.nodes[current_index]
                .children
                .iter()
                .max_by(|a, b| {
                    self.nodes[**a]
                        .uct_value(parent_visits, self.exploration_constant)
                        .partial_cmp(&self.nodes[**b].uct_value(parent_visits, self.exploration_constant))
                        .unwrap()
                })
                .unwrap();
        }
        current_index
    }

    fn expand_node(&mut self, node_index: usize) -> Vec<usize> {
        if self.nodes[node_index].game.is_game_over() {
            return Vec::new();
        }

        let game_clone = self.nodes[node_index].game.clone();
        let moves = game_clone.legal_moves();
        let mut new_children = Vec::new();
        for &mv in &moves {
            let mut new_game = game_clone.clone();
            let _ = new_game.make_move(mv);
            let new_node = Node::new(new_game, Some(node_index), Some(Move::Place(mv)));
            let new_node_index = self.nodes.len();
            self.nodes.push(new_node);
            self.nodes[node_index].children.push(new_node_index);
            new_children.push(new_node_index);
        }
        new_children
    }

    fn simulate(&mut self, node_index: usize) -> f64 {
        let mut game = self.nodes[node_index].game.clone();
        while !game.is_game_over() {
            let moves = game.legal_moves();
            if moves.is_empty() {
                game.pass();
            } else {
                let mv = moves[self.rng.gen_range(0..moves.len())];
                let _ = game.make_move(mv);
            }
        }
        let (black, white) = game.disc_count();
        let current_player = self.nodes[node_index].game.current_player;
        match current_player {
            Player::Black => {
                if black > white {
                    1.0
                } else if white > black {
                    0.0
                } else {
                    0.5
                }
            }
            Player::White => {
                if white > black {
                    1.0
                } else if black > white {
                    0.0
                } else {
                    0.5
                }
            }
        }
    }

    fn backpropagate(&mut self, node_index: usize, outcome: f64) {
        let mut current_index = Some(node_index);
        while let Some(index) = current_index {
            self.nodes[index].visits += 1;
            self.nodes[index].wins += outcome as u32;
            current_index = self.nodes[index].parent;
        }
    }

    fn best_move(&mut self, temperature: f64) -> Move {
        let root = &self.nodes[self.root_index];
        if temperature == 0.0 || root.children.is_empty() {
            let best_child = root
                .children
                .iter()
                .max_by_key(|c| self.nodes[**c].visits)
                .unwrap();
            self.nodes[*best_child].move_from_parent.unwrap()
        } else {
            // Sample proportionally to visits^(1/temperature)
            let weights: Vec<f64> = root.children.iter().map(|&c| (self.nodes[c].visits as f64).powf(1.0 / temperature)).collect();
            let total_weight: f64 = weights.iter().sum();
            let mut rand_val = self.rng.gen::<f64>() * total_weight;
            for (i, &weight) in weights.iter().enumerate() {
                rand_val -= weight;
                if rand_val <= 0.0 {
                    let child_index = root.children[i];
                    return self.nodes[child_index].move_from_parent.unwrap();
                }
            }
            // Fallback
            let best_child = root.children.iter().max_by_key(|c| self.nodes[**c].visits).unwrap();
            self.nodes[*best_child].move_from_parent.unwrap()
        }
    }

    /// Advances the root to the child corresponding to the given move.
    /// Returns true if successful, false if no such child exists.
    pub fn advance_root(&mut self, mv: Move) -> bool {
        let root = &self.nodes[self.root_index];
        for &child_index in &root.children {
            if self.nodes[child_index].move_from_parent == Some(mv) {
                self.root_index = child_index;
                return true;
            }
        }
        false
    }

    /// Returns a reference to the root game state.
    pub fn root_game(&self) -> &Game {
        &self.nodes[self.root_index].game
    }

    fn compute_telemetry(&self) -> Telemetry {
        let root = &self.nodes[self.root_index];
        let total_simulations = root.visits;
        let _total_depth = 0u32;
        let mut visit_distribution = Vec::new();
        for &child in &root.children {
            let child_node = &self.nodes[child];
            visit_distribution.push(child_node.visits);
            // Approximate depth as visits or something, but for simplicity, use 0
        }
        let average_depth = 0.0; // TODO: implement proper depth calculation
        let chosen_q_value = if root.children.is_empty() {
            0.0
        } else {
            let best_child = root.children.iter().max_by_key(|c| self.nodes[**c].visits).unwrap();
            self.nodes[*best_child].wins as f64 / self.nodes[*best_child].visits as f64
        };
        Telemetry {
            total_simulations,
            average_depth,
            chosen_q_value,
            visit_distribution,
        }
    }
}
