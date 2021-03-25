use std::fmt::{Debug, Display, Formatter};

use crate::pieces::Color;
use anyhow::Context;

const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

/// Position on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The file index, 0 to 7 (maps A to H)
    pub file_x: u8,
    /// The rank index, 0 to 7 (maps 1 to 8)
    pub rank_y: u8,
}

impl Position {
    /// Initialize a position from indexes.
    pub fn new(file_x: u8, rank_y: u8) -> anyhow::Result<Position> {
        if file_x > 7 {
            return Err(anyhow::Error::msg(format!("invalid file: {}", file_x)));
        }
        if rank_y > 7 {
            return Err(anyhow::Error::msg(format!("invalid rank: {}", rank_y)));
        }
        Ok(Position { file_x, rank_y })
    }

    /// Initialize a position from notation (for example: a8).
    pub fn from_notation(notation: &str) -> anyhow::Result<Position> {
        if notation.len() != 2 {
            return Err(anyhow::Error::msg(format!(
                "invalid notation: {}",
                notation
            )));
        }
        let mut chars = notation.chars();
        let file_char = chars.next().unwrap();
        let rank_char = chars.next().unwrap();

        let file_x = FILES
            .iter()
            .position(|c| c == &file_char)
            .with_context(|| "invalid notation file")? as u8;

        let rank_y = RANKS
            .iter()
            .position(|c| c == &rank_char)
            .with_context(|| "invalid notation rank")? as u8;

        Ok(Position { file_x, rank_y })
    }

    /// Rotates the position for the other side.
    pub fn flip(&self) -> Self {
        Position {
            file_x: self.file_x,
            rank_y: 7 - self.rank_y,
        }
    }

    /// Returns a new position, forwards by increment in the direction of the given color.
    pub fn forwards(&self, color: Color, inc: u8) -> Self {
        let rank = match color {
            Color::Black => self.rank_y - inc,
            Color::White => self.rank_y + inc,
        };
        Position {
            file_x: self.file_x,
            rank_y: rank.clamp(0, 7),
        }
    }

    /// Returns a new position, backwards by increment in the direction of the given color.
    pub fn backwards(&self, color: Color, inc: u8) -> Self {
        self.forwards(color.opposite(), inc)
    }

    /// Returns the distance between 2 positions on the Y axis (rank).
    pub fn distance_rank(&self, other: &Position) -> u8 {
        let s_y = self.rank_y as i32;
        let o_y = other.rank_y as i32;
        (s_y - o_y).abs() as u8
    }

    /// Convert to `BitBoard` notation.
    pub fn to_int(&self) -> u8 {
        self.rank_y << 3 ^ self.file_x
    }
}

/// Converts from a notation `String` (e.g. `"a1"`).
/// Note that this function panics if an invalid notation is given. Use `Position::from_notation`
/// for safer conversion.
impl From<String> for Position {
    fn from(p: String) -> Self {
        Position::from_notation(p.as_str()).expect("invalid notation")
    }
}

/// Converts from a notation `&str` (e.g. `"a1"`).
/// Note that this function panics if an invalid notation is given. Use `Position::from_notation`
/// for safer conversion.
impl From<&str> for Position {
    fn from(p: &str) -> Self {
        Position::from_notation(p).expect("invalid notation")
    }
}

/// Converts from `(file, rank)` tuple.
/// Note that this function panics if an invalid values are given. Use `Position::new`
/// for safer conversion.
impl From<(u8, u8)> for Position {
    fn from(p: (u8, u8)) -> Self {
        Position::new(p.0, p.1).expect("invalid notation")
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", FILES[self.file_x as usize], self.rank_y + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn test_position_notation() {
        let position = Position::from_notation("a1").unwrap();
        assert_eq!(format!("{}", position), "a1");
        let position = Position::from_notation("b2").unwrap();
        assert_eq!(format!("{}", position), "b2");
        let position = Position::from_notation("h8").unwrap();
        assert_eq!(format!("{}", position), "h8");
    }

    #[test]
    fn test_position_flip() {
        let position = Position::from_notation("a1").unwrap();
        assert_eq!(format!("{}", position.flip()), "a8");
        let position = Position::from_notation("b2").unwrap();
        assert_eq!(format!("{}", position.flip()), "b7");
        let position = Position::from_notation("h8").unwrap();
        assert_eq!(format!("{}", position.flip()), "h1");
    }
}
