use crate::pieces::{Piece, Pieces, Position};

/// A chess piece move.
#[derive(Debug, Clone, Copy)]
pub struct Move {
    /// The piece making the move.
    pub piece: Pieces,
    /// The original position of the move.
    pub source: Position,
    /// The final position of the move.
    pub target: Position,
}

impl Move {
    pub fn apply_to(&self, piece: &Pieces) -> Pieces {
        piece.set_position(self.target).into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {}
