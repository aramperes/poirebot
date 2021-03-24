use crate::bitboard::BitBoard;
use crate::pieces::Color;

#[derive(Debug, Clone)]
pub struct Brain {
    pub color: Color,
    pub black_pieces: BitBoard,
    pub white_pieces: BitBoard,
    pub black_pawns: BitBoard,
    pub white_pawns: BitBoard,
}

impl Default for Brain {
    /// An empty brain that doesn't have anything to do right now.
    fn default() -> Self {
        Brain {
            color: Color::Black, // Note: this should be changed before the moves are given.,
            ..Default::default()
        }
    }
}

impl Brain {
    pub fn opponent_move(&mut self) {}
}

#[cfg(test)]
mod tests {
    use crate::pieces::Position;

    use super::*;

    #[test]
    fn test_bitboard() {
        let one_end = BitBoard::from_position(Position::new(0, 0).unwrap());
        let other_end = BitBoard::from_position(Position::new(7, 7).unwrap());
        let combined = one_end | other_end;

        assert_eq!(
            vec!["a1", "h8"],
            combined
                .into_iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<String>>()
        );
    }
}
