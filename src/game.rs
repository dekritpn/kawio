//! Game logic for Othello (Reversi).
//!
//! The board is represented as a 64-bit bitboard, with bit 0 = A8 (top-left), bit 63 = H1 (bottom-right).
//! Coordinates use standard Othello notation: A1 = bottom-left (56), H8 = top-right (7).

use std::fmt;

/// Represents a player in the Othello game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Black,
    White,
}

impl Player {
    /// Returns the opponent of the current player.
    pub fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

/// Represents the state of an Othello game.
#[derive(Clone)]
pub struct Game {
    pub black: u64,  // Bitboard for black discs
    pub white: u64,  // Bitboard for white discs
    pub current_player: Player,
    pub passes: u8,  // Number of consecutive passes
}

const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;

impl Game {
    /// Creates a new Othello game with the standard initial position.
    pub fn new() -> Self {
        // Initial position: Black at E4 and D5, White at D4 and E5
        let black = (1u64 << 28) | (1u64 << 35); // E4=28, D5=35
        let white = (1u64 << 27) | (1u64 << 36); // D4=27, E5=36
        Game {
            black,
            white,
            current_player: Player::Black,
            passes: 0,
        }
    }

    /// Returns a bitboard of all occupied squares.
    /// Returns the bitboard of all occupied squares.
    pub fn occupied(&self) -> u64 {
        self.black | self.white
    }

    /// Returns the bitboard of all empty squares.
    pub fn empty(&self) -> u64 {
        !self.occupied() & ALL
    }

    /// Checks if a move at the given position is valid for the current player.
    pub fn is_valid_move(&self, pos: u8) -> bool {
        if pos >= 64 || (self.occupied() & (1u64 << pos)) != 0 {
            return false;
        }
        self.flips(pos) != 0
    }

    /// Calculates the bitboard of discs that would be flipped by placing a disc at the given position.
    ///
    /// This function checks all eight directions from the position to find opponent discs
    /// that are sandwiched between the new disc and an existing disc of the current player.
    /// Returns a bitboard where each bit represents a disc to be flipped.
    pub fn flips(&self, pos: u8) -> u64 {
        let mut flips = 0u64;
        let player_bb = if self.current_player == Player::Black {
            self.black
        } else {
            self.white
        };
        let opponent_bb = if self.current_player == Player::Black {
            self.white
        } else {
            self.black
        };

        // Directions: (dr, dc) for row and column deltas
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for &(dr, dc) in &directions {
            let mut r = (pos / 8) as i8 + dr;
            let mut c = (pos % 8) as i8 + dc;
            let mut temp_flips = 0u64;

            while r >= 0 && r < 8 && c >= 0 && c < 8 {
                let bit = 1u64 << (r as u64 * 8 + c as u64);
                if (opponent_bb & bit) != 0 {
                    temp_flips |= bit;
                } else if (player_bb & bit) != 0 {
                    flips |= temp_flips;
                    break;
                } else {
                    break;
                }
                r += dr;
                c += dc;
            }
        }
        flips
    }

    /// Places a disc at the given position and flips the appropriate opponent discs.
    /// Returns an error if the move is invalid.
    pub fn make_move(&mut self, pos: u8) -> Result<(), String> {
        if pos >= 64 {
            return Err("Position out of bounds".to_string());
        }
        if (self.occupied() & (1u64 << pos)) != 0 {
            return Err("Square is already occupied".to_string());
        }
        let flips = self.flips(pos);
        if flips == 0 {
            return Err("Move does not flip any discs".to_string());
        }
        let pos_bit = 1u64 << pos;
        if self.current_player == Player::Black {
            self.black |= pos_bit | flips;
            self.white &= !flips;
        } else {
            self.white |= pos_bit | flips;
            self.black &= !flips;
        }
        self.current_player = self.current_player.opponent();
        self.passes = 0;
        Ok(())
    }

    /// Passes the turn to the opponent and increments the pass counter.
    /// Passes the turn to the opponent and increments the pass counter.
    pub fn pass(&mut self) {
        self.current_player = self.current_player.opponent();
        self.passes += 1;
    }

    /// Returns a list of all legal move positions for the current player.
    pub fn legal_moves(&self) -> Vec<u8> {
        let mut moves = Vec::new();
        for pos in 0..64 {
            if self.is_valid_move(pos) {
                moves.push(pos);
            }
        }
        moves
    }

    /// Checks if the game is over (neither player has legal moves).
    pub fn is_game_over(&self) -> bool {
        !self.has_legal_move(Player::Black) && !self.has_legal_move(Player::White)
    }

    /// Returns the winner of the game, or None if it's a tie or not over.
    pub fn winner(&self) -> Option<Player> {
        if !self.is_game_over() {
            return None;
        }
        let black_count = self.black.count_ones();
        let white_count = self.white.count_ones();
        if black_count > white_count {
            Some(Player::Black)
        } else if white_count > black_count {
            Some(Player::White)
        } else {
            None  // Tie
        }
    }

    /// Returns the count of black and white discs as (black, white).
    pub fn disc_count(&self) -> (u32, u32) {
        (self.black.count_ones(), self.white.count_ones())
    }

    /// Checks if the given player has any legal moves.
    pub fn has_legal_move(&self, player: Player) -> bool {
        let temp_game = Game {
            black: self.black,
            white: self.white,
            current_player: player,
            passes: self.passes,
        };
        !temp_game.legal_moves().is_empty()
    }

    /// Converts a position (0-63) to a coordinate string, e.g., 56 -> "A1" (bottom-left).
    /// Uses standard Othello notation where A1 is bottom-left, H8 is top-right.
    pub fn pos_to_coord(pos: u8) -> String {
        let row_index = pos / 8;
        let col_index = pos % 8;
        let row = 8 - row_index;
        let col = (col_index + b'A') as char;
        format!("{}{}", col, row)
    }

    /// Converts a coordinate string to a position (0-63), e.g., "A1" -> 56.
    /// Uses standard Othello notation where A1 is bottom-left, H8 is top-right.
    /// Accepts lowercase and validates input.
    pub fn coord_to_pos(coord: &str) -> Result<u8, String> {
        if coord.len() != 2 {
            return Err("Coordinate must be exactly 2 characters".to_string());
        }
        let upper = coord.to_uppercase();
        let bytes = upper.as_bytes();
        let col = bytes[0];
        let row = bytes[1];
        if col < b'A' || col > b'H' {
            return Err("Column must be A-H".to_string());
        }
        if row < b'1' || row > b'8' {
            return Err("Row must be 1-8".to_string());
        }
        let col_index = col - b'A';
        let row_num = (row - b'0') as u8; // '1' -> 1
        let row_index = 8 - row_num;
        Ok(row_index * 8 + col_index)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "  A B C D E F G H")?;
        for row in 0..8 {
            let row_num = 8 - row;
            write!(f, "{} ", row_num)?;
            for col in 0..8 {
                let pos = row * 8 + col;
                let bit = 1u64 << pos;
                if (self.black & bit) != 0 {
                    write!(f, "B ")?;
                } else if (self.white & bit) != 0 {
                    write!(f, "W ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = Game::new();
        assert_eq!(game.black.count_ones(), 2);
        assert_eq!(game.white.count_ones(), 2);
        assert_eq!(game.current_player, Player::Black);
        assert_eq!(game.passes, 0);
    }

    #[test]
    fn test_coord_conversion() {
        assert_eq!(Game::coord_to_pos("A1"), Ok(56)); // bottom-left
        assert_eq!(Game::coord_to_pos("H8"), Ok(7));  // top-right
        assert_eq!(Game::coord_to_pos("A8"), Ok(0));  // top-left
        assert_eq!(Game::coord_to_pos("H1"), Ok(63)); // bottom-right
        assert_eq!(Game::coord_to_pos("D3"), Ok(43)); // D col 3, row 3 -> row_index 8-3=5, 5*8+3=43
        assert_eq!(Game::coord_to_pos("a1"), Ok(56)); // lowercase
        assert!(Game::coord_to_pos("I1").is_err());
        assert!(Game::coord_to_pos("A9").is_err());
        assert!(Game::coord_to_pos("A").is_err());
    }

    #[test]
    fn test_initial_moves() {
        let game = Game::new();
        let moves = game.legal_moves();
        assert_eq!(moves.len(), 4); // Standard Othello opening has 4 legal moves
        assert!(!moves.is_empty());
    }

    #[test]
    fn test_make_move() {
        let mut game = Game::new();
        let moves = game.legal_moves();
        assert!(!moves.is_empty());
        let pos = moves[0];
        game.make_move(pos).unwrap();
        assert_eq!(game.current_player, Player::White);
        assert!(game.black.count_ones() > 2); // more black discs
        assert!(game.white.count_ones() < 2); // fewer white discs
    }

    #[test]
    fn test_invalid_move() {
        let mut game = Game::new();
        // Try to place on occupied square
        let occupied_pos = 27; // D4, occupied by white
        assert!(game.make_move(occupied_pos).is_err());
        // Try to place where no flips
        let no_flip_pos = 0; // A8, empty but no flips
        assert!(game.make_move(no_flip_pos).is_err());
        // Out of bounds
        assert!(game.make_move(64).is_err());
    }

    #[test]
    fn test_game_over() {
        let game = Game::new();
        // Game is not over initially
        assert!(!game.is_game_over());
        // Game over when neither player has legal moves
        // For a new game, both have moves
        assert!(game.has_legal_move(Player::Black));
        assert!(game.has_legal_move(Player::White));
    }
}
