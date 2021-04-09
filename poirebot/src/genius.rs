use std::cmp::{max, min, Ordering};
use std::collections::BTreeSet;
use std::ops::Neg;

use itertools::Itertools;
use rand::{thread_rng, Rng};
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
    /// An estimate of how good the move may be.
    estimate: f32,
}

impl PartialEq for BrainMove {
    fn eq(&self, other: &Self) -> bool {
        self.m == other.m && self.estimate == other.estimate
    }
}

impl Eq for BrainMove {}

/// Implements the risk vs. reward scoring.
impl PartialOrd for BrainMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.estimate.partial_cmp(&other.estimate)
    }
}

impl Ord for BrainMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

type MoveCollection = Vec<BrainMove>;

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
    pub fn choose_move(&self, sensor: oneshot::Sender<Option<Move>>) {
        let board = self.board;
        let brain_color = self.color;

        rayon::spawn(move || {
            let mut rng = thread_rng();
            let best = negamax(
                brain_color,
                board,
                4,
                Evaluation::Worst,
                Evaluation::Best,
                brain_color,
                vec![],
            );

            info!("Best eval: {:?}", best);
            sensor.send(Some(best.m)).expect("Failed to dispatch Brain move");
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

/// List the potential movements and attacks by all pieces by the given color in the given board.
fn list_potential_moves(board: Board, color: Color) -> MoveCollection {
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
        let piece_type = board.get_piece(m.0).unwrap();
        let mut estimate = 0.0;
        if piece_type.is_pawn() {
            estimate += 0.5;
        }
        BrainMove { estimate, m }
    })
    .sorted()
    .rev()
    .collect::<MoveCollection>()
}

/// The recursive MiniMax function, with alpha-beta pruning.
fn negamax(
    genius_color: Color,
    board: Board,
    depth: usize,
    mut alpha: Evaluation,
    mut beta: Evaluation,
    color: Color,
    previous_moves: Vec<Move>,
) -> Node {
    let moves = list_potential_moves(board, color);
    if depth == 0 || moves.is_empty() {
        let eval = if moves.is_empty() && color == genius_color {
            Evaluation::Worst
        } else if moves.is_empty() && color != genius_color {
            Evaluation::Best
        } else {
            evaluate(genius_color, &board)
        };
        info!("Moves: {:?} = {:?}", previous_moves, eval);
        Node {
            eval,
            m: previous_moves[0],
        }
    } else {
        let mut value = Node::default();
        for m in moves {
            let mut outcome = board;
            outcome.apply_move(m.m);

            // Debugging
            let mut previous_moves = previous_moves.clone();
            previous_moves.push(m.m);

            value = max(
                value,
                -negamax(
                    genius_color,
                    outcome,
                    depth - 1,
                    -beta,
                    -alpha,
                    color.opposite(),
                    previous_moves,
                ),
            );

            alpha = max(alpha, value.eval);
            if alpha >= beta {
                break;
            }
        }
        value
    }
}

fn evaluate(color: Color, board: &Board) -> Evaluation {
    let score = board.piecewise_score(color);
    Evaluation::Score(score as i32)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Node {
    eval: Evaluation,
    m: Move,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            eval: Evaluation::Worst,
            m: Move(Position::default(), Position::default(), Promotion::None),
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.eval.cmp(&other.eval))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.eval.cmp(&other.eval)
    }
}

impl Neg for Node {
    type Output = Node;

    fn neg(self) -> Self::Output {
        Self {
            eval: -self.eval,
            m: self.m,
        }
    }
}

/// An assessment of a game state from a particular player's perspective.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Evaluation {
    /// An absolutely disastrous outcome, e.g. a loss.
    Worst,
    /// An outcome with some score. Higher values mean a more favorable state.
    Score(i32),
    /// An absolutely wonderful outcome, e.g. a win.
    Best,
}

/// Negating an evaluation results in the corresponding one from the other
/// player's perspective.
impl Neg for Evaluation {
    type Output = Evaluation;
    #[inline]
    fn neg(self) -> Evaluation {
        match self {
            Evaluation::Worst => Evaluation::Best,
            Evaluation::Score(s) => Evaluation::Score(-s),
            Evaluation::Best => Evaluation::Worst,
        }
    }
}

/// Selects the promotion to get based on destination position.
fn promote(destination: Position) -> Promotion {
    if destination.rank_y == 0 || destination.rank_y == 7 {
        Promotion::Queen
    } else {
        Promotion::None
    }
}
