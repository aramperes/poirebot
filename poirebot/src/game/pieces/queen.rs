use crate::bitboard::BitBoard;
use crate::game::pieces::sliding::{get_sliding_diagonal_moves, get_sliding_straight_moves};
use crate::game::pieces::Color;
use crate::game::Board;

/// Generates a bitboard with the moves that can be performed by the queens in the given bitboard.
///
/// Note that multiple queens can be passed in the bitboard; to get the moves for individual queens,
/// iterate over the queens and call this function with the singular bitboard for each queen.
pub fn get_queen_sliding_moves(board: &Board, color: Color, origin: &BitBoard) -> BitBoard {
    get_sliding_diagonal_moves(&board, color, &origin)
        | get_sliding_straight_moves(&board, color, &origin)
}
