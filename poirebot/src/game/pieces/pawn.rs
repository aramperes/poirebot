use crate::bitboard::BitBoard;
use crate::game::pieces::{FILE_A, FILE_H};

/// A bitboard with the potential diagonal attacks for pawns (does not include en passant).
///
/// Note: Assumes board was normalized to white's perspective.
pub fn get_pawn_diagonal_attack_squares(pawns: BitBoard) -> BitBoard {
    let left_side_attacks = BitBoard((pawns & !FILE_A).0 << 7);
    let right_side_attacks = BitBoard((pawns & !FILE_H).0 << 9);
    left_side_attacks | right_side_attacks
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use crate::game::position::Position;

    use super::*;

    #[test]
    fn test_get_pawn_diagonal_attack_squares() {
        let pawns: Vec<Position> = vec![
            "b1".into(),
            "b3".into(),
            "b5".into(),
            "a7".into(),
            "h7".into(),
            "h8".into(),
        ];
        let pawns = BitBoard::from_iter(pawns);

        eprintln!("\n{}", pawns);

        let attacks = get_pawn_diagonal_attack_squares(BitBoard::from_iter(pawns));
        assert_eq!(
            attacks
                .into_iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<String>>(),
            vec!["a2", "c2", "a4", "c4", "a6", "c6", "b8", "g8"]
        );
    }
}
