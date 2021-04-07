use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{Color, FILE_A, FILE_H, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7};
use crate::game::position::Position;
use crate::game::Board;

/// Generates a bitboard with the moves that can be performed by the pawn at the given origin.
pub fn get_pawn_moves_and_attacks(board: Board, color: Color, origin: Position) -> BitBoard {
    let origin_bb = BitBoard::from(origin);
    let mut result = EMPTY;

    let all_pieces = &board.white.pieces | &board.black.pieces;
    let other_side = board.get_side(color.opposite());
    let other_pieces = &other_side.pieces;
    let other_en_passant_target = &other_side.en_passant_target;

    if color.is_white() {
        // Single steps (lshift by 8 bits for: rank +1)
        result |= (origin_bb << 8) & !all_pieces;

        // Right-side attacks (lshift by 9 bits for: rank +1, file +1)
        // 'H' file is excluded to prevent overflow
        // Also checks possibility of en-passant
        result |= (origin_bb & !FILE_H) << 9 & (other_pieces | other_en_passant_target);

        // Left-side attacks (lshift by 7 bits for: rank +1, file -1).
        // 'A' file is excluded to prevent underflow
        // Also checks possibility of en-passant
        result |= ((origin_bb & !FILE_A) << 7) & (other_pieces | other_en_passant_target);

        // Double steps
        // 1. Only include pawns in rank 2
        // 2. Ensure there are no pieces in rank 3
        // 3. Ensure there are no pieces in rank 4
        // 4. Add 2 ranks to the original position
        result |=
            (origin_bb & RANK_2 & !(all_pieces & RANK_3) >> 8 & !(all_pieces & RANK_4) >> 16) << 16;
    } else {
        // Single steps (rshift by 8 bits for: rank -1)
        result |= (origin_bb >> 8) & !all_pieces;

        // Right-side attacks (rshift by 7 bits for: rank -1, file +1)
        // 'H' file is excluded to prevent overflow
        // Also checks possibility of en-passant
        result |= (origin_bb & !FILE_H) >> 7 & (other_pieces | other_en_passant_target);

        // Left-side attacks (rshift by 9 bits for: rank -1, file -1).
        // 'A' file is excluded to prevent underflow
        // Also checks possibility of en-passant
        result |= ((origin_bb & !FILE_A) >> 9) & (other_pieces | other_en_passant_target);

        // Double steps
        // 1. Only include pawns in rank 7
        // 2. Ensure there are no pieces in rank 6
        // 3. Ensure there are no pieces in rank 5
        // 4. Remove 2 ranks to the original position
        result |=
            (origin_bb & RANK_7 & !(all_pieces & RANK_6) << 8 & !(all_pieces & RANK_5) << 16) >> 16;
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::position::Position;
    use crate::game::Board;

    #[test]
    fn test_get_pawn_moves_and_attacks_white() {
        let board =
            Board::from_fen("rn2k2r/pp2pp1p/8/2pPb1p1/1b3P1n/3qP1Pp/PP1P3P/RNBQKBNR w KQkq c6 0 1")
                .unwrap();

        // a2 can do single-step or double-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "a2".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("a3"), Position::from("a4")]
        );

        // b2 can do single-step only
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "b2".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("b3")]
        );

        // d2 cannot move
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "d2".into())
                .collect::<Vec<Position>>(),
            vec![]
        );

        // d5 can do en-passant or single-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "d5".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("c6"), Position::from("d6")]
        );

        // e3 can do single-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "e3".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("e4")]
        );

        // f4 can capture left, right, or single-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "f4".into())
                .collect::<Vec<Position>>(),
            vec![
                Position::from("e5"),
                Position::from("f5"),
                Position::from("g5")
            ]
        );

        // g3 can capture right, or single-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::White, "g3".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("g4"), Position::from("h4")]
        );
    }

    #[test]
    fn test_get_pawn_moves_and_attacks_black() {
        let board =
            Board::from_fen("rn2k2r/p4p1p/3P1b2/b7/5Ppn/pppqP1Pp/PP1P3P/RNBQKBNR b KQkq f3 0 1")
                .unwrap();

        // h8 can do single-step or double-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "h7".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("h5"), Position::from("h6")]
        );

        // h3 cannot move
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "h3".into())
                .collect::<Vec<Position>>(),
            vec![]
        );

        // g4 can only do en-passant
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "g4".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("f3")]
        );

        // f7 cannot move
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "f7".into())
                .collect::<Vec<Position>>(),
            vec![]
        );

        // c3 can capture left, right, or single-step
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "c3".into())
                .collect::<Vec<Position>>(),
            vec![
                Position::from("b2"),
                Position::from("c2"),
                Position::from("d2")
            ]
        );

        // b3 can capture left only
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "b3".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("a2")]
        );

        // a3 can capture right only
        assert_eq!(
            super::get_pawn_moves_and_attacks(board, Color::Black, "a3".into())
                .collect::<Vec<Position>>(),
            vec![Position::from("b2")]
        );
    }
}
