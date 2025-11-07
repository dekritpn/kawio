use crate::game::{Game, Player};
use rusqlite::{Connection, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct PlayerStats {
    pub name: String,
    pub elo: f64,
    pub wins: i32,
    pub losses: i32,
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS games (
                  id TEXT PRIMARY KEY,
                  black INTEGER NOT NULL,
                  white INTEGER NOT NULL,
                  current_player TEXT NOT NULL,
                  passes INTEGER NOT NULL,
                  player1 TEXT NOT NULL,
                  player2 TEXT NOT NULL
              )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                  name TEXT PRIMARY KEY,
                  elo REAL NOT NULL DEFAULT 1200,
                  wins INTEGER NOT NULL DEFAULT 0,
                  losses INTEGER NOT NULL DEFAULT 0
              )",
            [],
        )?;
        Ok(Storage { conn })
    }

    pub fn save_game(&self, id: &str, game: &Game, player1: &str, player2: &str) -> Result<()> {
        let current_player = match game.current_player {
            Player::Black => "Black",
            Player::White => "White",
        };
        self.conn.execute(
            "INSERT OR REPLACE INTO games (id, black, white, current_player, passes, player1, player2) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [id, &game.black.to_string(), &game.white.to_string(), current_player, &game.passes.to_string(), player1, player2],
        )?;
        Ok(())
    }

    pub fn load_game(&self, id: &str) -> Result<Option<(Game, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT black, white, current_player, passes, player1, player2 FROM games WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let black: i64 = row.get(0)?;
            let white: i64 = row.get(1)?;
            if black < 0 || white < 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            let current_player: String = row.get(2)?;
            let passes: u8 = row.get(3)?;
            let player1: String = row.get(4)?;
            let player2: String = row.get(5)?;
            let player = if current_player == "Black" {
                Player::Black
            } else {
                Player::White
            };
            Ok((
                Game {
                    black: black as u64,
                    white: white as u64,
                    current_player: player,
                    passes,
                },
                player1,
                player2,
            ))
        })?;
        if let Some(row) = rows.next() {
            let (game, p1, p2) = row?;
            Ok(Some((game, p1, p2)))
        } else {
            Ok(None)
        }
    }

    pub fn load_all_games(
        &self,
    ) -> Result<(HashMap<String, Game>, HashMap<String, (String, String)>)> {
        let mut stmt = self.conn.prepare(
            "SELECT id, black, white, current_player, passes, player1, player2 FROM games",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let black: i64 = row.get(1)?;
            let white: i64 = row.get(2)?;
            if black < 0 || white < 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            let current_player: String = row.get(3)?;
            let passes: u8 = row.get(4)?;
            let player1: String = row.get(5)?;
            let player2: String = row.get(6)?;
            let player = if current_player == "Black" {
                Player::Black
            } else {
                Player::White
            };
            Ok((
                id,
                Game {
                    black: black as u64,
                    white: white as u64,
                    current_player: player,
                    passes,
                },
                player1,
                player2,
            ))
        })?;
        let mut games = HashMap::new();
        let mut players = HashMap::new();
        for row in rows {
            let (id, game, p1, p2) = row?;
            games.insert(id.clone(), game);
            players.insert(id, (p1, p2));
        }
        Ok((games, players))
    }

    fn ensure_player(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO players (name, elo, wins, losses) VALUES (?1, 1200, 0, 0)",
            [name],
        )?;
        Ok(())
    }

    fn get_elo(&self, name: &str) -> Result<f64> {
        let mut stmt = self
            .conn
            .prepare("SELECT elo FROM players WHERE name = ?1")?;
        stmt.query_row([name], |row| row.get(0)).or(Ok(1200.0))
    }

    fn update_elo(&self, name: &str, elo: f64) -> Result<()> {
        self.ensure_player(name)?;
        self.conn.execute(
            "UPDATE players SET elo = ?1 WHERE name = ?2",
            [&elo.to_string(), name],
        )?;
        Ok(())
    }

    fn update_wins_losses(&self, name: &str, won: bool) -> Result<()> {
        self.ensure_player(name)?;
        let column = if won { "wins" } else { "losses" };
        self.conn.execute(
            &format!(
                "UPDATE players SET {} = {} + 1 WHERE name = ?1",
                column, column
            ),
            [name],
        )?;
        Ok(())
    }

    fn calculate_elo(rating_a: f64, rating_b: f64, a_won: bool) -> (f64, f64) {
        let k = 32.0;
        let expected_a = 1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) / 400.0));
        let expected_b = 1.0 - expected_a;
        let score_a = if a_won { 1.0 } else { 0.0 };
        let score_b = 1.0 - score_a;
        let new_a = rating_a + k * (score_a - expected_a);
        let new_b = rating_b + k * (score_b - expected_b);
        (new_a, new_b)
    }

    pub fn update_player(&self, player: &str, opponent: &str, player_won: bool) -> Result<()> {
        self.ensure_player(player)?;
        self.ensure_player(opponent)?;
        let player_elo = self.get_elo(player)?;
        let opponent_elo = self.get_elo(opponent)?;
        let (new_player_elo, new_opponent_elo) =
            Self::calculate_elo(player_elo, opponent_elo, player_won);
        self.update_elo(player, new_player_elo)?;
        self.update_elo(opponent, new_opponent_elo)?;
        self.update_wins_losses(player, player_won)?;
        self.update_wins_losses(opponent, !player_won)?;
        Ok(())
    }

    pub fn get_leaderboard(&self) -> Result<Vec<PlayerStats>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, elo, wins, losses FROM players ORDER BY elo DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(PlayerStats {
                name: row.get(0)?,
                elo: row.get(1)?,
                wins: row.get(2)?,
                losses: row.get(3)?,
            })
        })?;
        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }
        Ok(stats)
    }
}
