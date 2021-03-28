use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILE_A, FILE_H, RANK_2, RANK_3, RANK_4};
use crate::game::{Board, Move, Promotion};

/// Returns a collection of moves that are unobstructed pawn double-steps.
pub fn get_pawn_double_steps(board: Board, color: Color) -> Vec<Move> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut all_pieces = board.white.pieces | board.black.pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.mirror_horizontally().reverse_colors();
        all_pieces = all_pieces.mirror_horizontally().reverse_colors();
    }

    // Pawns on rank 2 that are unobstructed to rank 4
    let pawns = own_pawns
        & RANK_2
        & !BitBoard((all_pieces & RANK_3).0 >> 8)
        & !BitBoard((all_pieces & RANK_4).0 >> 16);

    // Convert to `Move`
    pawns
        .split()
        .into_iter()
        .map(|pawn| {
            let destination = BitBoard(pawn.0 << 16); // Add 2 ranks
            if color == Color::Black {
                // De-normalize by reversing colors
                Move(
                    pawn.reverse_colors().mirror_horizontally().to_position(),
                    destination
                        .reverse_colors()
                        .mirror_horizontally()
                        .to_position(),
                    Promotion::None,
                )
            } else {
                Move(
                    pawn.to_position(),
                    destination.to_position(),
                    Promotion::None,
                )
            }
        })
        .collect()
}

/// Returns a collection of moves that are unobstructed pawn double-steps.
pub fn get_pawn_single_steps(board: Board, color: Color) -> Vec<Move> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut all_pieces = board.white.pieces | board.black.pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.reverse_colors().mirror_horizontally();
        all_pieces = all_pieces.reverse_colors().mirror_horizontally();
    }

    // Pawn single-step destinations with no obstruction
    let destinations = BitBoard(own_pawns.0 << 8) & !all_pieces;

    // Convert to `Move`
    destinations
        .split()
        .into_iter()
        .map(|destination| {
            let pawn = BitBoard(destination.0 >> 8); // Remove 1 rank
            if color == Color::Black {
                // De-normalize by reversing colors
                Move(
                    pawn.reverse_colors().mirror_horizontally().to_position(),
                    destination
                        .reverse_colors()
                        .mirror_horizontally()
                        .to_position(),
                    Promotion::None,
                )
            } else {
                Move(
                    pawn.to_position(),
                    destination.to_position(),
                    Promotion::None,
                )
            }
        })
        .collect()
}

/// Returns a collection of moves that are unobstructed pawn attacks on the left side.
/// Each move is paired with the value of the piece taken.
pub fn get_pawn_left_attacks(board: Board, color: Color) -> Vec<(Move, u8)> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut other_pieces = board.get_side(color.opposite()).pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.reverse_colors().mirror_horizontally();
        other_pieces = other_pieces.reverse_colors().mirror_horizontally();
    }

    // Pawn single-step destinations with no obstruction
    let attacked = BitBoard((own_pawns & !FILE_A).0 << 7) & other_pieces;

    // Convert to `Move`
    attacked
        .split()
        .into_iter()
        .map(|destination| {
            let pawn = BitBoard(destination.0 >> 7);
            let m = if color == Color::Black {
                // De-normalize by reversing colors
                Move(
                    pawn.reverse_colors().mirror_horizontally().to_position(),
                    destination
                        .reverse_colors()
                        .mirror_horizontally()
                        .to_position(),
                    Promotion::None,
                )
            } else {
                Move(
                    pawn.to_position(),
                    destination.to_position(),
                    Promotion::None,
                )
            };
            let value = board.get_piece_value(m.1);
            (m, value)
        })
        .collect()
}

/// Returns a collection of moves that are unobstructed pawn attacks on the right side.
/// Each move is paired with the value of the piece taken.
pub fn get_pawn_right_attacks(board: Board, color: Color) -> Vec<(Move, u8)> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut other_pieces = board.get_side(color.opposite()).pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.reverse_colors().mirror_horizontally();
        other_pieces = other_pieces.reverse_colors().mirror_horizontally();
    }

    // Pawn single-step destinations with no obstruction
    let attacked_squares = BitBoard((own_pawns & !FILE_H).0 << 9);
    let attacked = attacked_squares & other_pieces;

    // Convert to `Move`
    attacked
        .split()
        .into_iter()
        .map(|destination| {
            let pawn = BitBoard(destination.0 >> 9);
            let m = if color == Color::Black {
                // De-normalize by reversing colors
                Move(
                    pawn.reverse_colors().mirror_horizontally().to_position(),
                    destination
                        .reverse_colors()
                        .mirror_horizontally()
                        .to_position(),
                    Promotion::None,
                )
            } else {
                Move(
                    pawn.to_position(),
                    destination.to_position(),
                    Promotion::None,
                )
            };
            let value = board.get_piece_value(m.1);
            (m, value)
        })
        .collect()
}
