use std::fmt::{Debug, Display, Formatter};

use anyhow::Context;

pub mod pawn;

const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

/// A chess piece.
#[derive(Debug, Clone, Copy)]
pub enum Pieces {
    Pawn(pawn::Pawn),
}

impl Piece for Pieces {
    fn get_color(&self) -> Color {
        match self {
            Pieces::Pawn(p) => p.get_color(),
        }
    }

    fn get_position(&self) -> Position {
        match self {
            Pieces::Pawn(p) => p.get_position(),
        }
    }

    fn set_position(&self, new_position: Position) -> Self {
        match self {
            Pieces::Pawn(p) => p.set_position(new_position).into(),
        }
    }
}

/// A chess piece set (white or black).
#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

/// Position on the board.
#[derive(Debug, Clone, Copy)]
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

    pub fn to_int(&self) -> u8 {
        self.rank_y << 3 ^ self.file_x
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", FILES[self.file_x as usize], self.rank_y + 1)
    }
}

/// Common piece trait.
pub trait Piece {
    /// Get the color of the piece.
    fn get_color(&self) -> Color;
    /// Get the position of the piece.
    fn get_position(&self) -> Position;
    /// Update the position of the current piece.
    fn set_position(&self, new_position: Position) -> Self;
}

#[cfg(test)]
mod tests {
    use crate::pieces::Position;

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
