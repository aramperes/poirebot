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

#[derive(Debug, Clone, Copy)]
pub struct TurnCounter {
    pub first_move: bool,
    pub our_turn: bool,
}

impl Default for TurnCounter {
    fn default() -> Self {
        TurnCounter {
            first_move: true,
            our_turn: false,
        }
    }
}

impl TurnCounter {
    pub fn next(&mut self) {
        self.first_move = false;
        self.our_turn = !self.our_turn;
    }
}
