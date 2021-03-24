use std::sync::Arc;
use std::time::Duration;

use rand::seq::SliceRandom;
use tokio::sync::oneshot;

use crate::bitboard::BitBoard;
use crate::game::Move;
use crate::pieces::{Color, Piece, Pieces, Position};

#[derive(Debug, Clone)]
pub struct Brain {
    pub color: Color,
    pub white_pieces: BitBoard,
    pub black_pieces: BitBoard,
    pub white_pawns: BitBoard,
    pub black_pawns: BitBoard,
}

impl Default for Brain {
    /// An empty brain that doesn't have anything to do right now.
    fn default() -> Self {
        let white_pawns = BitBoard::from_position(Position::from_notation("a2").unwrap())
            | BitBoard::from_position(Position::from_notation("b2").unwrap())
            | BitBoard::from_position(Position::from_notation("c2").unwrap())
            | BitBoard::from_position(Position::from_notation("d2").unwrap())
            | BitBoard::from_position(Position::from_notation("e2").unwrap())
            | BitBoard::from_position(Position::from_notation("f2").unwrap())
            | BitBoard::from_position(Position::from_notation("g2").unwrap())
            | BitBoard::from_position(Position::from_notation("h2").unwrap());
        let black_pawns = white_pawns.reverse_colors();

        let white_pieces = white_pawns.clone(); // TODO Add other pieces!
        let black_pieces = white_pieces.reverse_colors();

        Brain {
            color: Color::Black, // Note: this should be changed before the moves are given
            white_pieces,
            black_pieces,
            white_pawns,
            black_pawns,
        }
    }
}

impl Brain {
    pub fn opponent_move(&mut self, m: Move) {
        self.apply_move(m, self.color.opposite());
    }

    pub fn own_move(&mut self, m: Move) {
        self.apply_move(m, self.color);
    }

    pub fn choose_move(&mut self, sensor: oneshot::Sender<Move>) {
        let color = self.color;
        let brain_ref = self.clone();

        rayon::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));

            // Choose a random pawn and move forwards by one
            let pawns = brain_ref.get_pawns(color);
            let pawn = pawns
                .choose(&mut rand::thread_rng())
                .expect("no pawn to move");

            let origin = pawn.get_position();
            let destination = pawn.get_position().forwards(color, 1);

            sensor
                .send(Move::Displace(origin, destination))
                .expect("Failed to dispatch Brain move");
        })
    }

    fn apply_move(&mut self, m: Move, color: Color) {
        // Update the board with whatever the opponent did

        match m {
            Move::Displace(origin, destination) => {
                let origin_bb = BitBoard::from_position(origin);
                let destination_bb = BitBoard::from_position(destination);

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
        }
    }

    fn get_pawns(&self, color: Color) -> Vec<crate::pieces::pawn::Pawn> {
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

    fn get_piece(&self, position: Position) -> Option<Pieces> {
        let bb = BitBoard::from_position(position);
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
