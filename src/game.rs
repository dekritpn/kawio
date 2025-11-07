use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

#[derive(Clone)]
pub struct Game {
    pub black: u64,  // Bitboard for black discs
    pub white: u64,  // Bitboard for white discs
    pub current_player: Player,
    pub passes: u8,  // Number of consecutive passes
}

impl Game {
    pub fn new() -> Self {
        // Initial position: Black at D4 (3,3) and E5 (4,4), White at E4 (4,3) and D5 (3,4)
        let black = (1u64 << 27) | (1u64 << 36);  // D4=27, E5=36
        let white = (1u64 << 28) | (1u64 << 35);  // E4=28, D5=35
        Game {
            black,
            white,
            current_player: Player::Black,
            passes: 0,
        }
    }

    pub fn occupied(&self) -> u64 {
        self.black | self.white
    }

    pub fn empty(&self) -> u64 {
        !self.occupied()
    }

    pub fn is_valid_move(&self, pos: u8) -> bool {
        if pos >= 64 || (self.occupied() & (1u64 << pos)) != 0 {
            return false;
        }
        self.flips(pos) != 0
    }

    pub fn flips(&self, pos: u8) -> u64 {
        let mut flips = 0u64;
        let player_bb = if self.current_player == Player::Black { self.black } else { self.white };
        let opponent_bb = if self.current_player == Player::Black { self.white } else { self.black };

        // Directions: (dr, dc) for row and column deltas
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
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

    pub fn make_move(&mut self, pos: u8) {
        let pos_bit = 1u64 << pos;
        let flips = self.flips(pos);
        if self.current_player == Player::Black {
            self.black |= pos_bit | flips;
            self.white &= !flips;
        } else {
            self.white |= pos_bit | flips;
            self.black &= !flips;
        }
        self.current_player = self.current_player.opponent();
        self.passes = 0;
    }

    pub fn pass(&mut self) {
        self.current_player = self.current_player.opponent();
        self.passes += 1;
    }

    pub fn legal_moves(&self) -> Vec<u8> {
        let mut moves = Vec::new();
        for pos in 0..64 {
            if self.is_valid_move(pos) {
                moves.push(pos);
            }
        }
        moves
    }

    pub fn is_game_over(&self) -> bool {
        self.passes == 2
    }

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

    pub fn disc_count(&self) -> (u32, u32) {
        (self.black.count_ones(), self.white.count_ones())
    }

    // Convert position to coordinate string, e.g., 0 -> "A1"
    pub fn pos_to_coord(pos: u8) -> String {
        let row = (pos / 8) as u8 + b'1';
        let col = (pos % 8) as u8 + b'A';
        format!("{}{}", col as char, row as char)
    }

    // Convert coordinate string to position, e.g., "A1" -> 0
    pub fn coord_to_pos(coord: &str) -> Option<u8> {
        if coord.len() != 2 {
            return None;
        }
        let bytes = coord.as_bytes();
        let col = bytes[0] as u8;
        let row = bytes[1] as u8;
        if col < b'A' || col > b'H' || row < b'1' || row > b'8' {
            return None;
        }
        let c = col - b'A';
        let r = row - b'1';
        Some(r * 8 + c)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "  A B C D E F G H")?;
        for row in 0..8 {
            write!(f, "{} ", (b'1' + row) as char)?;
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
        assert_eq!(Game::coord_to_pos("A1"), Some(0));
        assert_eq!(Game::coord_to_pos("H8"), Some(63));
        assert_eq!(Game::coord_to_pos("D3"), Some(19)); // D is 3, row 3-1=2, 2*8+3=19
        assert_eq!(Game::coord_to_pos("I1"), None);
    }

    #[test]
    fn test_initial_moves() {
        let game = Game::new();
        let moves = game.legal_moves();
        assert!(moves.contains(&Game::coord_to_pos("E3").unwrap()));
        assert!(moves.contains(&Game::coord_to_pos("F4").unwrap()));
    }

    #[test]
    fn test_make_move() {
        let mut game = Game::new();
        let pos = Game::coord_to_pos("E3").unwrap();
        assert!(game.is_valid_move(pos));
        game.make_move(pos);
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.black.count_ones(), 4); // placed + flipped 1
        assert_eq!(game.white.count_ones(), 1); // flipped 1
    }
}