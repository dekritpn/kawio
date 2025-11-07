use crate::game::{Game, Player};
use crate::storage::Storage;
use std::collections::HashMap;
use std::env;

pub struct Sessions {
    games: HashMap<String, Game>,
    players: HashMap<String, (String, String)>,
    next_id: u64,
    pub storage: Storage,
    queue: Vec<String>,
}

impl Sessions {
    pub fn new() -> Self {
        let db_path = env::var("DB_PATH").unwrap_or_else(|_| "kawio.db".to_string());
        let storage = Storage::new(&db_path).expect("Failed to open database");
        let (games, players) = storage.load_all_games().expect("Failed to load games");
        let next_id = games.len() as u64 + 1;
        Sessions {
            games,
            players,
            next_id,
            storage,
            queue: Vec::new(),
        }
    }

    pub fn join_matchmaking(&mut self, player: String) -> Option<String> {
        if self.queue.is_empty() {
            self.queue.push(player);
            None
        } else {
            let opponent = self.queue.remove(0);
            Some(self.create_game(player, opponent))
        }
    }

    pub fn create_game(&mut self, player1: String, player2: String) -> String {
        let id = format!("game_{}", self.next_id);
        self.next_id += 1;
        let game = Game::new();
        self.games.insert(id.clone(), game.clone());
        self.players
            .insert(id.clone(), (player1.clone(), player2.clone()));
        self.storage
            .save_game(&id, &game, &player1, &player2)
            .expect("Failed to save game");
        id
    }

    pub fn get_game(&self, id: &str) -> Option<&Game> {
        self.games.get(id)
    }

    pub fn get_game_mut(&mut self, id: &str) -> Option<&mut Game> {
        self.games.get_mut(id)
    }

    pub fn get_players(&self, id: &str) -> Option<&(String, String)> {
        self.players.get(id)
    }

    pub fn make_move(&mut self, id: &str, pos: u8, player: &str) -> Result<(), String> {
        let (p1, p2) = self.players.get(id).ok_or("Game not found".to_string())?;
        if let Some(game) = self.games.get_mut(id) {
            let current_player_name = match game.current_player {
                Player::Black => p1,
                Player::White => p2,
            };
            if player != current_player_name {
                return Err("Not your turn".to_string());
            }
            if game.is_valid_move(pos) {
                game.make_move(pos)?;
                if game.is_game_over() {
                    if let Some(winner) = game.winner() {
                        let player_won = winner == Player::Black;
                        self.storage
                            .update_player(p1, p2, player_won)
                            .expect("Failed to update player");
                    }
                }
                self.storage
                    .save_game(id, game, p1, p2)
                    .expect("Failed to save game");
                Ok(())
            } else {
                Err("Invalid move".to_string())
            }
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn pass(&mut self, id: &str) -> Result<(), String> {
        if let Some(game) = self.games.get_mut(id) {
            game.pass();
            if game.is_game_over() {
                let (p1, p2) = self.players.get(id).unwrap();
                if let Some(winner) = game.winner() {
                    let player_won = winner == Player::Black;
                    self.storage
                        .update_player(p1, p2, player_won)
                        .expect("Failed to update player");
                }
                self.storage
                    .save_game(id, game, p1, p2)
                    .expect("Failed to save game");
            }
            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn list_games(&self) -> Vec<String> {
        self.games.keys().cloned().collect()
    }

    // Test helpers
    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    pub fn has_game(&self, id: &str) -> bool {
        self.games.contains_key(id)
    }

    pub fn has_player(&self, id: &str) -> bool {
        self.players.contains_key(id)
    }
}
