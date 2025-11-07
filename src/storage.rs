use rusqlite::{Connection, Result};
use std::collections::HashMap;
use crate::game::{Game, Player};

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
            let player = if current_player == "Black" { Player::Black } else { Player::White };
            Ok((Game { black: black as u64, white: white as u64, current_player: player, passes }, player1, player2))
        })?;
        if let Some(row) = rows.next() {
            let (game, p1, p2) = row?;
            Ok(Some((game, p1, p2)))
        } else {
            Ok(None)
        }
    }

    pub fn load_all_games(&self) -> Result<(HashMap<String, Game>, HashMap<String, (String, String)>)> {
        let mut stmt = self.conn.prepare("SELECT id, black, white, current_player, passes, player1, player2 FROM games")?;
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
            let player = if current_player == "Black" { Player::Black } else { Player::White };
            Ok((id, Game { black: black as u64, white: white as u64, current_player: player, passes }, player1, player2))
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
}