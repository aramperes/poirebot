use crate::pieces::{Color, Position, Piece};

/// A pawn.
#[derive(Debug, Clone)]
pub struct Pawn {
    /// The position of the piece.
    pub position: Position,
    /// The pawn's color.
    pub color: Color,
}

impl Piece for Pawn {
    fn get_color(&self) -> Color {
        self.color
    }

    fn get_position(&self) -> Position {
        self.position
    }
}
