use crate::game::{Game, Player};

const EXPLORATION_CONSTANT: f64 = 1.414;

struct Node {
    visits: u32,
    wins: u32,
    parent: Option<usize>,
    children: Vec<usize>,
    game: Game,
    move_from_parent: Option<u8>,
}

impl Node {
    fn new(game: Game, parent: Option<usize>, move_from_parent: Option<u8>) -> Self {
        Node {
            visits: 0,
            wins: 0,
            parent,
            children: Vec::new(),
            game,
            move_from_parent,
        }
    }

    fn uct_value(&self, parent_visits: u32) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            (self.wins as f64 / self.visits as f64)
                + EXPLORATION_CONSTANT * (parent_visits as f64).ln() / (self.visits as f64)
        }
    }
}

pub struct MCTS {
    nodes: Vec<Node>,
}

impl MCTS {
    pub fn new(game: Game) -> Self {
        let root_node = Node::new(game, None, None);
        MCTS {
            nodes: vec![root_node],
        }
    }

    pub fn search(&mut self, iterations: u32) -> u8 {
        for _ in 0..iterations {
            let leaf_index = self.select_leaf();
            let expanded_children = self.expand_node(leaf_index);
            for child_index in expanded_children {
                let outcome = self.simulate(child_index);
                self.backpropagate(child_index, outcome);
            }
        }
        self.best_move()
    }

    fn select_leaf(&self) -> usize {
        let mut current_index = 0;
        while !self.nodes[current_index].children.is_empty() {
            let parent_visits = self.nodes[current_index].visits;
            current_index = *self.nodes[current_index]
                .children
                .iter()
                .max_by(|a, b| {
                    self.nodes[**a]
                        .uct_value(parent_visits)
                        .partial_cmp(&self.nodes[**b].uct_value(parent_visits))
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
            let new_node = Node::new(new_game, Some(node_index), Some(mv));
            let new_node_index = self.nodes.len();
            self.nodes.push(new_node);
            self.nodes[node_index].children.push(new_node_index);
            new_children.push(new_node_index);
        }
        new_children
    }

    fn simulate(&self, node_index: usize) -> f64 {
        let mut game = self.nodes[node_index].game.clone();
        while !game.is_game_over() {
            let moves = game.legal_moves();
            if moves.is_empty() {
                game.pass();
            } else {
                let mv = moves[rand::random::<usize>() % moves.len()];
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

    fn best_move(&self) -> u8 {
        let root = &self.nodes[0];
        let best_child = root
            .children
            .iter()
            .max_by_key(|c| self.nodes[**c].visits)
            .unwrap();
        self.nodes[*best_child].move_from_parent.unwrap()
    }
}
