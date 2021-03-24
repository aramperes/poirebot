use crate::bitboard::BitBoard;
use crate::pieces::{Color, Pieces, Position};
use std::time::Duration;
use tokio::sync::oneshot;

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
    pub fn opponent_move(&mut self, m: String) {
        self.apply_move(m, self.color.opposite());
    }

    pub fn own_move(&mut self, m: String) {
        self.apply_move(m, self.color);
    }

    pub fn choose_move(&mut self, sensor: oneshot::Sender<String>) {
        let color = self.color;
        rayon::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));
            let m = match color {
                Color::White => "a2a4",
                Color::Black => "a7a5",
            };
            sensor
                .send(m.into())
                .expect("Failed to dispatch Brain move");
        })
    }

    fn apply_move(&mut self, m: String, color: Color) {
        // Update the board with whatever the opponent did

        // This only supports simple moves
        // TODO: Taking pieces, castling, etc.
        let origin = m.chars().take(2).collect::<String>();
        let current = m.chars().skip(2).take(2).collect::<String>();

        let origin = Position::from_notation(&origin).unwrap();
        let current = Position::from_notation(&current).unwrap();

        let origin_bb = BitBoard::from_position(origin);
        let current_bb = BitBoard::from_position(current);

        match color {
            Color::White => match self.get_piece(origin) {
                Some(Pieces::Pawn(_)) => {
                    self.white_pawns &= !origin_bb;
                    self.white_pawns |= current_bb;
                    self.white_pieces &= !origin_bb;
                    self.white_pieces |= current_bb;
                }
                _ => (),
            },
            Color::Black => match self.get_piece(origin) {
                Some(Pieces::Pawn(_)) => {
                    self.black_pawns &= !origin_bb;
                    self.black_pawns |= current_bb;
                    self.black_pieces &= !origin_bb;
                    self.black_pieces |= current_bb;
                }
                _ => (),
            },
        }
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
