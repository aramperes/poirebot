use tokio::sync::oneshot;

use crate::game::pieces;
use crate::game::pieces::Color;
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};
use std::cmp::Ordering;
use std::collections::BTreeSet;

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

/// Describes a move that the brain could perform.
#[derive(Debug, Copy, Clone)]
struct BrainMove {
    /// The move being completed.
    m: Move,
    /// A 'risk' heuristic on the move.
    risk: f32,
    /// A 'reward' heuristic on the move. A very high reward is capturing a high-score piece like a queen or rook.
    /// Forks are also very high-reward.
    reward: f32,
    /// Resulting board state.
    result: Board,
}

impl PartialEq for BrainMove {
    fn eq(&self, other: &Self) -> bool {
        self.m == other.m && self.result == other.result
    }
}

impl Eq for BrainMove {}

/// Implements the risk vs. reward scoring.
impl PartialOrd for BrainMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Play defensively for now
        let defensiveness = 0.7;
        let score = -(self.reward - defensiveness * self.risk);
        let other_score = -(other.reward - defensiveness * other.risk);
        score.partial_cmp(&other_score)
    }
}

impl Ord for BrainMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
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

    /// Select a move for the brain.
    pub fn choose_move(&mut self, sensor: oneshot::Sender<Option<Move>>) {
        let board = self.board;
        let color = self.color;

        rayon::spawn(move || {
            let moves = list_potential_moves(board, color);
            let m = moves.into_iter().next().map(|m| m.m);
            sensor.send(m).expect("Failed to dispatch Brain move");
        })
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
}

/// List the potential movements and attacks by all pieces.
///
/// Note: lists the moves on the given board. This can be different than
/// the current board as it allows recursive-ness.
fn list_potential_moves(board: Board, color: Color) -> BTreeSet<BrainMove> {
    [
        "pawn:double_step",
        "pawn:single_step",
        "pawn:attack_left",
        "pawn:attack_right",
    ]
    .iter()
    .map(|task| match *task {
        "pawn:double_step" => pieces::pawn::get_pawn_double_steps(board, color)
            .into_iter()
            .map(|m| {
                let base_reward = if m.0.file_x == 4 { 0.05 } else { 0.01 };
                let base_risk = 0.0;
                (m.with_promotion(promote(m.1)), base_reward, base_risk)
            })
            .collect(),
        "pawn:single_step" => pieces::pawn::get_pawn_single_steps(board, color)
            .into_iter()
            .map(|m| {
                let base_reward = 0.0;
                let base_risk = 0.0;
                (m.with_promotion(promote(m.1)), base_reward, base_risk)
            })
            .collect(),
        "pawn:attack_left" => pieces::pawn::get_pawn_left_attacks(board, color)
            .into_iter()
            .map(|(m, value)| {
                let base_reward = f32::from(value);
                let base_risk = 0.0;
                (m.with_promotion(promote(m.1)), base_reward, base_risk)
            })
            .collect(),
        "pawn:attack_right" => pieces::pawn::get_pawn_right_attacks(board, color)
            .into_iter()
            .map(|(m, value)| {
                let base_reward = f32::from(value);
                let base_risk = 0.0;
                (m.with_promotion(promote(m.1)), base_reward, base_risk)
            })
            .collect(),
        _ => Vec::with_capacity(0),
    })
    .map(|moves| {
        // Score the moves according to board state
        moves.into_iter().map(|(m, reward, risk)| {
            let mut result = board;
            result.apply_move(m);

            let mut reward = reward;
            let risk = risk;

            if m.2 == Promotion::Queen {
                reward += 8.0;
            }

            // TODO: Adjust risk based on board result (self-check, self-fork, etc.)
            // TODO: Adjust reward based on potential future attacks (forks, etc.)
            BrainMove {
                m,
                risk,
                reward,
                result,
            }
        })
    })
    .fold(BTreeSet::new(), |mut sum, val| {
        sum.extend(val);
        sum
    })
}

/// Selects the promotion to get based on destination position.
fn promote(destination: Position) -> Promotion {
    if destination.rank_y == 0 || destination.rank_y == 7 {
        Promotion::Queen
    } else {
        Promotion::None
    }
}
