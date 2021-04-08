use crate::bitboard::BitBoard;
use crate::game::pieces::sliding::get_sliding_diagonal_moves;
use crate::game::pieces::Color;
use crate::game::Board;

/// Generates a bitboard with the moves that can be performed by the bishops in the given bitboard.
///
/// Note that multiple bishops can be passed in the bitboard; to get the moves for individual bishops,
/// iterate over the bishops and call this function with the singular bitboard for each bishop.
pub fn get_bishop_sliding_moves(board: &Board, color: Color, origin: &BitBoard) -> BitBoard {
    get_sliding_diagonal_moves(&board, color, &origin)
}

#[cfg(test)]
mod tests {
    use crate::bitboard::BitBoard;
    use crate::game::pieces::Color;
    use crate::game::position::Position;
    use crate::game::Board;

    #[test]
    fn get_bishop_sliding_moves() {
        let board = Board::from_fen("7P/6P1/5P2/4P1R1/3P4/2P1b3/1P6/P1r5 b - - 0 1").unwrap();
        let moves = super::get_bishop_sliding_moves(&board, Color::Black, &BitBoard::from("e3"));
        assert_eq!(
            moves.collect::<Vec<Position>>(),
            vec![
                Position::from("g1"),
                Position::from("d2"),
                Position::from("f2"),
                Position::from("d4"),
                Position::from("f4"),
                Position::from("g5"),
            ]
        );
    }

    #[test]
    fn get_bishop_sliding_moves_2() {
        let board = Board::from_fen("rnb1k1nr/7p/7p/8/8/bppp1p2/3B4/1N1QKBNq w kq - 0 1").unwrap();
        let moves = super::get_bishop_sliding_moves(&board, Color::White, &BitBoard::from("f1"));
        assert_eq!(
            moves.collect::<Vec<Position>>(),
            vec![
                Position::from("e2"),
                Position::from("g2"),
                Position::from("d3"),
                Position::from("h3"),
            ]
        );
    }
}
