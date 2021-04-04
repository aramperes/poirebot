use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILE_A, FILE_H, RANK_2, RANK_3, RANK_4};
use crate::game::{Board, Move, Promotion};

/// Returns a collection of moves that are unobstructed pawn double-steps.
pub fn get_pawn_double_steps(board: Board, color: Color) -> Vec<Move> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut all_pieces = board.white.pieces | board.black.pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.rotate();
        all_pieces = all_pieces.rotate();
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
            let mut pawn = pawn;
            let mut destination = BitBoard(pawn.0 << 16); // Add 2 ranks
            if color == Color::Black {
                // De-normalize
                pawn = pawn.rotate();
                destination = destination.rotate();
            }
            Move(
                pawn.to_position(),
                destination.to_position(),
                Promotion::None,
            )
        })
        .collect()
}

/// Returns a collection of moves that are unobstructed pawn double-steps.
pub fn get_pawn_single_steps(board: Board, color: Color) -> Vec<Move> {
    let mut own_pawns = board.get_side(color).pawns;
    let mut all_pieces = board.white.pieces | board.black.pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_pawns = own_pawns.rotate();
        all_pieces = all_pieces.rotate();
    }

    // Pawn single-step destinations with no obstruction
    let destinations = BitBoard(own_pawns.0 << 8) & !all_pieces;

    // Convert to `Move`
    destinations
        .split()
        .into_iter()
        .map(|destination| {
            let mut pawn = BitBoard(destination.0 >> 8); // Remove 1 rank
            let mut destination = destination;
            if color == Color::Black {
                // De-normalize
                pawn = pawn.rotate();
                destination = destination.rotate();
            }
            Move(
                pawn.to_position(),
                destination.to_position(),
                Promotion::None,
            )
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
        own_pawns = own_pawns.rotate();
        other_pieces = other_pieces.rotate();
    }

    // Pawn single-step destinations with no obstruction
    let attacked = BitBoard((own_pawns & !FILE_A).0 << 7) & other_pieces;

    // Convert to `Move`
    attacked
        .split()
        .into_iter()
        .map(|destination| {
            let mut destination = destination;
            let mut pawn = BitBoard(destination.0 >> 7);
            if color == Color::Black {
                // De-normalize
                pawn = pawn.rotate();
                destination = destination.rotate();
            }
            let m = Move(
                pawn.to_position(),
                destination.to_position(),
                Promotion::None,
            );
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
        own_pawns = own_pawns.rotate();
        other_pieces = other_pieces.rotate();
    }

    // Pawn single-step destinations with no obstruction
    let attacked_squares = BitBoard((own_pawns & !FILE_H).0 << 9);
    let attacked = attacked_squares & other_pieces;

    // Convert to `Move`
    attacked
        .split()
        .into_iter()
        .map(|destination| {
            let mut destination = destination;
            let mut pawn = BitBoard(destination.0 >> 9);
            if color == Color::Black {
                // De-normalize
                pawn = pawn.rotate();
                destination = destination.rotate();
            }
            let m = Move(
                pawn.to_position(),
                destination.to_position(),
                Promotion::None,
            );
            let value = board.get_piece_value(m.1);
            (m, value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::{Board, Move};

    #[test]
    fn test_pawn_double_step_white() {
        let board = Board::default();
        let double_steps = super::get_pawn_double_steps(board, Color::White);
        assert_eq!(
            double_steps,
            vec![
                ("a2", "a4").into(),
                ("b2", "b4").into(),
                ("c2", "c4").into(),
                ("d2", "d4").into(),
                ("e2", "e4").into(),
                ("f2", "f4").into(),
                ("g2", "g4").into(),
                ("h2", "h4").into(),
            ]
        );
    }

    #[test]
    fn test_pawn_double_step_black() {
        let board = Board::default();
        let double_steps = super::get_pawn_double_steps(board, Color::Black);
        assert_eq!(
            double_steps,
            vec![
                ("a7", "a5").into(),
                ("b7", "b5").into(),
                ("c7", "c5").into(),
                ("d7", "d5").into(),
                ("e7", "e5").into(),
                ("f7", "f5").into(),
                ("g7", "g5").into(),
                ("h7", "h5").into(),
            ]
            .into_iter()
            .rev()
            .collect::<Vec<Move>>()
        );
    }

    #[test]
    fn test_pawn_double_step_obstructed() {
        let board =
            Board::from_fen("rn2kbnr/pp2pppp/2p5/8/1b3P2/3qP1Pp/PPPP3P/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let double_steps = super::get_pawn_double_steps(board, Color::White);
        assert_eq!(
            double_steps,
            vec![("a2", "a4").into(), ("c2", "c4").into(),]
        );
    }
}
