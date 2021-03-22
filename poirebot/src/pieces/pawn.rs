use crate::pieces::{Color, Piece, Pieces, Position};

/// A pawn.
#[derive(Debug, Clone, Copy)]
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

    fn set_position(&self, new_position: Position) -> Self {
        let mut new_pawn = self.clone();
        new_pawn.position = new_position;
        new_pawn
    }
}

impl From<Pawn> for Pieces {
    fn from(pawn: Pawn) -> Self {
        Pieces::Pawn(pawn)
    }
}
