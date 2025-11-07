use std::collections::HashMap;
use crate::game::{Game, Player};
use crate::storage::Storage;

pub struct Sessions {
    games: HashMap<String, Game>,
    next_id: u64,
    storage: Storage,
}

impl Sessions {
    pub fn new() -> Self {
        let storage = Storage::new("kawio.db").expect("Failed to open database");
        let games = storage.load_all_games().expect("Failed to load games");
        let next_id = games.len() as u64 + 1;
        Sessions {
            games,
            next_id,
            storage,
        }
    }

    pub fn create_game(&mut self) -> String {
        let id = format!("game_{}", self.next_id);
        self.next_id += 1;
        let game = Game::new();
        self.games.insert(id.clone(), game.clone());
        self.storage.save_game(&id, &game).expect("Failed to save game");
        id
    }

    pub fn get_game(&self, id: &str) -> Option<&Game> {
        self.games.get(id)
    }

    pub fn get_game_mut(&mut self, id: &str) -> Option<&mut Game> {
        self.games.get_mut(id)
    }

    pub fn make_move(&mut self, id: &str, pos: u8) -> Result<(), String> {
        if let Some(game) = self.games.get_mut(id) {
            if game.is_valid_move(pos) {
                game.make_move(pos);
                self.storage.save_game(id, game).expect("Failed to save game");
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
            self.storage.save_game(id, game).expect("Failed to save game");
            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn list_games(&self) -> Vec<String> {
        self.games.keys().cloned().collect()
    }
}