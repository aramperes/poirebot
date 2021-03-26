use std::time::Duration;

use rand::seq::SliceRandom;
use tokio::sync::oneshot;

use crate::game::pieces::Color;
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

#[derive(Debug, Clone, Copy)]
pub struct Brain {
    /// This brain's color.
    pub color: Color,
    /// The current board state.
    board: Board,
    /// The last move by the brain.
    pub last_move: Option<Move>,
    /// The last move by the opponent.
    pub opponent_last_move: Option<Move>,
}

impl Brain {
    /// Create a new brain with the given board and color.
    pub fn new(board: Board, color: Color) -> Self {
        Self {
            board,
            color,
            last_move: None,
            opponent_last_move: None,
        }
    }

    /// Apply a move from the opponent.
    pub fn opponent_move(&mut self, m: Move) {
        self.board.apply_move(m);
        self.opponent_last_move = Some(m);
    }

    /// Apply a move by the brain.
    pub fn own_move(&mut self, m: Move) {
        self.board.apply_move(m);
        self.last_move = Some(m);
    }

    /// Select a move for the brain.
    pub fn choose_move(&mut self, sensor: oneshot::Sender<Option<Move>>) {
        let color = self.color;
        let board = self.board;

        rayon::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));
            let mut m = None;

            // Check if we can attack anything with pawns
            if let Some(destination) = board
                .get_squares_attacked_by_pawns(color)
                .into_iter()
                .next()
            {
                // HACK: to find which pawn to use to attack, we re-use the same function
                // but we make all enemy pieces pawns and see what THEY can attack
                let mut board_swap = board.clone();
                board_swap.mutate(|board| {
                    let side = board.get_side_mut(color.opposite());
                    side.mutate(|side| {
                        (*side).pawns |= side.rooks;
                        (*side).pawns |= side.knights;
                        (*side).pawns |= side.bishops;
                        (*side).pawns |= side.queens;
                    });
                });
                let origin = board_swap
                    .get_squares_attacked_by_pawns(color.opposite())
                    .next()
                    .expect("inconsistency while trying to get attacking pawn");

                m = Some(Move(origin, destination, choose_promotion(destination)))
            } else {
                // Choose a random pawn and move forwards by one
                let pawns = board.get_pawns(color);
                if let Some(pawn) = pawns.choose(&mut rand::thread_rng()) {
                    let origin = pawn.get_position();
                    let destination = origin.forwards(color, 1);
                    m = Some(Move(origin, destination, choose_promotion(destination)));
                }
            }
            sensor.send(m).expect("Failed to dispatch Brain move");
        })
    }
}

/// Selects the promotion to get based on destination position.
fn choose_promotion(destination: Position) -> Promotion {
    if destination.rank_y == 0 || destination.rank_y == 7 {
        Promotion::Queen
    } else {
        Promotion::None
    }
}
