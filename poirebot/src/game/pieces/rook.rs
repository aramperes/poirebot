use crate::game::pieces::sliding::{
    get_sliding_east_moves, get_sliding_vertical_moves, get_sliding_west_moves,
};
use crate::game::pieces::Color;
use crate::game::{Board, Move};

pub fn get_rook_sliding_moves(board: Board, color: Color) -> Vec<(Move, u8)> {
    let own_rooks = board.get_side(color).rooks;

    let mut moves = Vec::new();
    for rook in own_rooks.into_iter() {
        moves.extend(get_sliding_vertical_moves(board, color, rook));
        moves.extend(get_sliding_east_moves(board, color, rook));
        moves.extend(get_sliding_west_moves(board, color, rook));
    }

    moves
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::{Board, Move};

    #[test]
    fn test_get_white_rook_sliding_moves() {
        let board =
            Board::from_fen("2b1kb1r/rpppp1pp/8/Pp5P/2P4q/n1R1p2R/2PPPPP1/1NBQKBNn w k - 0 1")
                .unwrap();
        let moves = super::get_rook_sliding_moves(board, Color::White);
        assert_eq!(
            moves,
            vec![
                (Move::from_pure_notation("c3d3"), 0),
                (Move::from_pure_notation("c3e3"), 1),
                (Move::from_pure_notation("c3b3"), 0),
                (Move::from_pure_notation("c3a3"), 3),
                (Move::from_pure_notation("h3h4"), 8),
                (Move::from_pure_notation("h3h2"), 0),
                (Move::from_pure_notation("h3h1"), 3),
                (Move::from_pure_notation("h3g3"), 0),
                (Move::from_pure_notation("h3f3"), 0),
                (Move::from_pure_notation("h3e3"), 1),
            ]
        );
    }

    #[test]
    fn test_get_black_rook_sliding_moves() {
        let board =
            Board::from_fen("2B1KB1R/RPPPP1PP/8/pP5p/2p4Q/N1r1P2r/2ppppp1/1nbqkbnN b K - 0 1")
                .unwrap();
        let moves = super::get_rook_sliding_moves(board, Color::Black);
        assert_eq!(
            moves,
            vec![
                (Move::from_pure_notation("c3d3"), 0),
                (Move::from_pure_notation("c3e3"), 1),
                (Move::from_pure_notation("c3b3"), 0),
                (Move::from_pure_notation("c3a3"), 3),
                (Move::from_pure_notation("h3h4"), 8),
                (Move::from_pure_notation("h3h2"), 0),
                (Move::from_pure_notation("h3h1"), 3),
                (Move::from_pure_notation("h3g3"), 0),
                (Move::from_pure_notation("h3f3"), 0),
                (Move::from_pure_notation("h3e3"), 1),
            ]
        );
    }
}
