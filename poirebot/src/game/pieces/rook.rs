use crate::bitboard::BitBoard;
use crate::game::pieces::sliding::get_sliding_straight_moves;
use crate::game::pieces::Color;
use crate::game::Board;

/// Generates a bitboard with the moves that can be performed by the rooks in the given bitboard.
///
/// Note that multiple rooks can be passed in the bitboard; to get the moves for individual rooks,
/// iterate over the rooks and call this function with the singular bitboard for each rook.
pub fn get_rook_sliding_moves(board: &Board, color: Color, origin: &BitBoard) -> BitBoard {
    get_sliding_straight_moves(&board, color, &origin)
}

#[cfg(test)]
mod tests {
    use crate::bitboard::BitBoard;
    use crate::game::pieces::Color;
    use crate::game::position::Position;
    use crate::game::Board;

    #[test]
    fn test_get_white_rook_sliding_moves() {
        let board = Board::from_fen("7P/6P1/5P2/4P1R1/3P4/2P5/1P2b1r1/P7 w - - 0 1").unwrap();
        let moves = super::get_rook_sliding_moves(&board, Color::White, &BitBoard::from("g5"));
        assert_eq!(
            moves.collect::<Vec<Position>>(),
            vec![
                Position::from("g2"),
                Position::from("g3"),
                Position::from("g4"),
                Position::from("f5"),
                Position::from("h5"),
                Position::from("g6"),
            ]
        );
    }
}
