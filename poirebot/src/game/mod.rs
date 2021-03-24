use crate::pieces::{Piece, Pieces, Position};

/// A chess piece move.
#[derive(Debug, Clone, Copy)]
pub enum Move {
    /// Goes from point A to point B. Might be "taking" a piece.
    Displace(Position, Position),
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
