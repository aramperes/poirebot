use std::fmt::{Display, Formatter};

use crate::bitboard::BitBoard;
use crate::pieces::{Color, Pieces, Position};

/// A chess piece move (origin and destination).
#[derive(Debug, Clone, Copy)]
pub struct Move(pub Position, pub Position, pub Promotion);

impl From<(&str, &str)> for Move {
    fn from(m: (&str, &str)) -> Self {
        Move(m.0.into(), m.1.into(), Promotion::None)
    }
}

impl From<(Position, Position)> for Move {
    fn from(m: (Position, Position)) -> Self {
        Move(m.0, m.1, Promotion::None)
    }
}

/// A pawn promotion decision. Use `None` when there is no promotion.
#[derive(Debug, Clone, Copy)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

impl Default for Promotion {
    fn default() -> Self {
        Promotion::None
    }
}

impl Display for Promotion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queen => write!(f, "q"),
            Self::Rook => write!(f, "r"),
            Self::Bishop => write!(f, "b"),
            Self::Knight => write!(f, "n"),
            Self::None => Ok(()),
        }
    }
}

impl From<&str> for Promotion {
    fn from(p: &str) -> Self {
        match p {
            "q" => Self::Queen,
            "r" => Self::Rook,
            "b" => Self::Bishop,
            "n" => Self::Knight,
            _ => Self::None,
        }
    }
}

impl From<String> for Promotion {
    fn from(p: String) -> Self {
        Self::from(p.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub white_pieces: BitBoard,
    pub black_pieces: BitBoard,
    pub white_pawns: BitBoard,
    pub black_pawns: BitBoard,
}

impl Board {
    /// Update the board after a player moved.
    pub fn apply_move(&mut self, m: Move, color: Color) {
        let Move(origin, destination, _promotion) = m;

        let origin_bb = BitBoard::from(origin);
        let destination_bb = BitBoard::from(destination);

        match color {
            Color::White => match self.get_piece(origin) {
                Some(Pieces::Pawn(_)) => {
                    self.white_pawns &= !origin_bb;
                    self.white_pawns |= destination_bb;
                    self.white_pieces &= !origin_bb;
                    self.white_pieces |= destination_bb;
                }
                _ => (),
            },
            Color::Black => match self.get_piece(origin) {
                Some(Pieces::Pawn(_)) => {
                    self.black_pawns &= !origin_bb;
                    self.black_pawns |= destination_bb;
                    self.black_pieces &= !origin_bb;
                    self.black_pieces |= destination_bb;
                }
                _ => (),
            },
        }
    }

    /// Get a list of pawns of the given color.
    pub fn get_pawns(&self, color: Color) -> Vec<crate::pieces::pawn::Pawn> {
        let pawns = if color == Color::White {
            self.white_pawns
        } else {
            self.black_pawns
        };
        pawns
            .into_iter()
            .map(move |position| crate::pieces::pawn::Pawn { color, position })
            .collect()
    }

    /// Get the piece at the given position if any.
    pub fn get_piece(&self, position: Position) -> Option<Pieces> {
        let bb = BitBoard::from(position);
        if (bb & self.white_pawns).popcnt() == 1 {
            Some(Pieces::Pawn(crate::pieces::pawn::Pawn {
                position,
                color: Color::White,
            }))
        } else if (bb & self.black_pawns).popcnt() == 1 {
            Some(Pieces::Pawn(crate::pieces::pawn::Pawn {
                position,
                color: Color::Black,
            }))
        } else {
            None
        }
    }
}
impl Default for Board {
    fn default() -> Self {
        let white_pawns: BitBoard = BitBoard::from_position("a2".into())
            | BitBoard::from_position("b2".into())
            | BitBoard::from_position("c2".into())
            | BitBoard::from_position("d2".into())
            | BitBoard::from_position("e2".into())
            | BitBoard::from_position("f2".into())
            | BitBoard::from_position("g2".into())
            | BitBoard::from_position("h2".into());
        let black_pawns = white_pawns.reverse_colors();

        let white_pieces = white_pawns.clone(); // TODO Add other pieces!
        let black_pieces = white_pieces.reverse_colors();

        Self {
            white_pieces,
            black_pieces,
            white_pawns,
            black_pawns,
        }
    }
}

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
