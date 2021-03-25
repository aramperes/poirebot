use std::time::Duration;

use rand::seq::SliceRandom;
use tokio::sync::oneshot;

use crate::game::{Board, Move, Promotion};
use crate::pieces::Color;

#[derive(Debug, Clone, Copy)]
pub struct Brain {
    /// This brain's color.
    pub color: Color,
    /// The current board state.
    board: Board,
}

impl Default for Brain {
    /// An empty brain that doesn't have anything to do right now.
    fn default() -> Self {
        Brain {
            color: Color::Black, // Note: this should be changed before the moves are given
            board: Board::default(),
        }
    }
}

impl Brain {
    /// Apply a move from the opponent.
    pub fn opponent_move(&mut self, m: Move) {
        self.board.apply_move(m);
    }

    /// Apply a move by the brain.
    pub fn own_move(&mut self, m: Move) {
        self.board.apply_move(m);
    }

    /// Select a move for the brain.
    pub fn choose_move(&mut self, sensor: oneshot::Sender<Option<Move>>) {
        let color = self.color;
        let board = self.board;

        rayon::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));

            // Choose a random pawn and move forwards by one
            let pawns = board.get_pawns(color);

            if let Some(pawn) = pawns.choose(&mut rand::thread_rng()) {
                let origin = pawn.get_position();
                let destination = origin.forwards(color, 1);
                let promotion = Promotion::None;
                sensor
                    .send(Some(Move(origin, destination, promotion)))
                    .expect("Failed to dispatch Brain move");
            } else {
                sensor.send(None).expect("Failed to dispatch Brain move");
            }
        })
    }
}
