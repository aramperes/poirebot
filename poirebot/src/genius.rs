use std::cmp::Ordering;
use std::collections::BTreeSet;

use tokio::sync::oneshot;

use crate::bitboard::BitBoard;
use crate::game::pieces;
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

/// Describes a move that the brain could perform.
#[derive(Debug, Copy, Clone)]
struct BrainMove {
    /// The move being completed.
    m: Move,
    /// MiniMax 'min' heuristic.
    min: f32,
    /// MiniMax 'max' heuristic.
    max: f32,
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
        let score = -(self.max - defensiveness * self.min);
        let other_score = -(other.max - defensiveness * other.min);
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
    debug!(
        "Board before Genius move: \n{}",
        board.draw_ascii(Color::White)
    );
    let side = board.get_side(color);
    [
        "pawn",
        "rook:sliding",
        "knight:step",
        "bishop:sliding",
        "queen:sliding",
        "king:step",
    ]
    .iter()
    .flat_map(|task| match *task {
        "pawn" => side
            .pawns
            .map(|pawn| {
                (
                    pawn,
                    pieces::pawn::get_pawn_moves_and_attacks(&board, color, &BitBoard::from(pawn)),
                    true,
                )
            })
            .collect(),
        "rook:sliding" => side
            .rooks
            .map(|rook| {
                (
                    rook,
                    pieces::rook::get_rook_sliding_moves(&board, color, &BitBoard::from(rook)),
                    false,
                )
            })
            .collect(),
        "knight:step" => side
            .knights
            .map(|knight| {
                (
                    knight,
                    pieces::knight::get_knight_moves(&board, color, knight),
                    false,
                )
            })
            .collect(),
        "bishop:sliding" => side
            .bishops
            .map(|bishop| {
                (
                    bishop,
                    pieces::bishop::get_bishop_sliding_moves(
                        &board,
                        color,
                        &BitBoard::from(bishop),
                    ),
                    false,
                )
            })
            .collect(),
        "queen:sliding" => side
            .queens
            .map(|queen| {
                (
                    queen,
                    pieces::queen::get_queen_sliding_moves(&board, color, &BitBoard::from(queen)),
                    false,
                )
            })
            .collect(),
        "king:step" => side
            .king
            .map(|king| {
                (
                    king,
                    pieces::king::get_king_steps(&board, color, king),
                    false,
                )
            })
            .collect(),
        _ => Vec::with_capacity(0),
    })
    .flat_map(|(origin, moves, can_promote)| {
        moves.map(move |destination| {
            if can_promote {
                Move(origin, destination, promote(destination))
            } else {
                Move::from((origin, destination))
            }
        })
    })
    .map(|m| {
        let mut result = board;
        result.apply_move(m);
        (m, result)
    })
    .filter(|(_, result)| !result.is_in_check(color))
    .map(|(m, result)| {
        // TODO: Evaluate minimax
        BrainMove {
            result,
            min: 0.0,
            max: 1.0,
            m,
        }
    })
    .collect()
}

/// Selects the promotion to get based on destination position.
fn promote(destination: Position) -> Promotion {
    if destination.rank_y == 0 || destination.rank_y == 7 {
        Promotion::Queen
    } else {
        Promotion::None
    }
}
