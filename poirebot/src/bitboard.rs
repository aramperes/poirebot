use std::fmt;
use std::iter::FromIterator;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, Shr,
};

use crate::game::position::Position;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

/// The Zero board state.
#[allow(dead_code)]
pub const EMPTY: BitBoard = BitBoard(0);

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAnd<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOr<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitXor<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: BitBoard) {
        self.0 &= other.0;
    }
}

impl BitAndAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: &BitBoard) {
        self.0 &= other.0;
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: BitBoard) {
        self.0 |= other.0;
    }
}

impl BitOrAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: &BitBoard) {
        self.0 |= other.0;
    }
}

impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: BitBoard) {
        self.0 ^= other.0;
    }
}

impl BitXorAssign<&BitBoard> for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: &BitBoard) {
        self.0 ^= other.0;
    }
}

impl Mul for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul<&BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: &BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Mul<BitBoard> for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl Not for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl Shl<u8> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: u8) -> BitBoard {
        BitBoard(self.0 << rhs)
    }
}

impl Shl<u8> for &BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: u8) -> BitBoard {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<u8> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: u8) -> BitBoard {
        BitBoard(self.0 >> rhs)
    }
}

impl Shr<u8> for &BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: u8) -> BitBoard {
        BitBoard(self.0 >> rhs)
    }
}

impl fmt::Display for BitBoard {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut v = Vec::new();
        let mut s: String = "".to_owned();
        for x in 0..64 {
            if self.0 & (1u64 << x) == (1u64 << x) {
                s.push_str("X ");
            } else {
                s.push_str(". ");
            }
            if x % 8 == 7 {
                v.push(s.clone());
                s.clear();
            }
        }
        v.reverse();
        write!(f, "{}", v.join("\n"))
    }
}

impl BitBoard {
    /// Construct a new bitboard from a u64
    #[inline]
    pub fn new(b: u64) -> BitBoard {
        BitBoard(b)
    }

    /// Construct a new `BitBoard` with a particular `Position` set
    #[inline]
    pub fn from_position<T: Into<Position>>(position: T) -> BitBoard {
        BitBoard(1u64 << position.into().to_int())
    }

    /// Convert to Position
    #[inline]
    pub fn to_position(&self) -> Position {
        let trailing = self.0.trailing_zeros() as u8;
        let file = trailing & 7;
        let rank = trailing >> 3;
        Position::new(file % 8, rank % 8).unwrap()
    }

    /// Count the number of `Positions` set in this `BitBoard`
    #[inline]
    pub fn popcnt(&self) -> u32 {
        self.0.count_ones()
    }

    /// Reverse this `BitBoard`.  Look at it from the opponents perspective.
    #[inline]
    pub fn reverse_colors(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
    }

    /// Mirror this `Bitboard` horizontally (left becomes right).
    #[inline]
    pub fn mirror_horizontally(&self) -> BitBoard {
        let mut x = self.0;
        let k1: u64 = 0x5555555555555555;
        let k2: u64 = 0x3333333333333333;
        let k4: u64 = 0x0f0f0f0f0f0f0f0f;
        x = ((x >> 1) & k1) | ((x & k1) << 1);
        x = ((x >> 2) & k2) | ((x & k2) << 2);
        x = ((x >> 4) & k4) | ((x & k4) << 4);
        BitBoard(x)
    }

    /// Mirror this `Bitboard` horizontally (left becomes right) and flip the colors.
    #[inline]
    pub fn rotate(&self) -> BitBoard {
        self.reverse_colors().mirror_horizontally()
    }

    /// Flip this `Bitboard` about the diagonal.
    /// Source: https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Diagonal
    #[inline]
    #[rustfmt::skip]
    pub fn flip_diagonally(&self) -> BitBoard {
        let mut x = self.0;
        let mut t;
        let k1: u64 = 0x5500550055005500;
        let k2: u64 = 0x3333000033330000;
        let k4: u64 = 0x0f0f0f0f00000000;
        t  = k4 & (x ^ (x << 28));
        x ^=       t ^ (t >> 28) ;
        t  = k2 & (x ^ (x << 14));
        x ^=       t ^ (t >> 14) ;
        t  = k1 & (x ^ (x <<  7));
        x ^=       t ^ (t >>  7) ;
        BitBoard(x)
    }

    /// Flip this `Bitboard` about the anti-diagonal.
    /// Source: https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Anti-Diagonal
    #[inline]
    #[rustfmt::skip]
    pub fn flip_anti_diagonally(&self) -> BitBoard {
        let mut x = self.0;
        let mut t;
        let k1: u64 = 0xaa00aa00aa00aa00;
        let k2: u64 = 0xcccc0000cccc0000;
        let k4: u64 = 0xf0f0f0f00f0f0f0f;
        t  =       x ^ (x << 36) ;
        x ^= k4 & (t ^ (x >> 36));
        t  = k2 & (x ^ (x << 18));
        x ^=       t ^ (t >> 18) ;
        t  = k1 & (x ^ (x <<  9));
        x ^=       t ^ (t >>  9) ;
        BitBoard(x)
    }

    /// Convert this `BitBoard` to a `usize` (for table lookups)
    #[inline]
    pub fn to_size(&self, rightshift: u8) -> usize {
        (self.0 >> rightshift) as usize
    }

    /// Split the current bitboard to a
    pub fn split(self) -> Vec<BitBoard> {
        let mut vec: Vec<BitBoard> = Vec::with_capacity(self.popcnt() as usize);
        for pos in self {
            vec.push(BitBoard::from(pos));
        }
        vec
    }
}

impl<T: Into<Position>> From<T> for BitBoard {
    fn from(into_pos: T) -> Self {
        Self::from_position(into_pos)
    }
}

impl FromIterator<Position> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Position>>(iter: T) -> Self {
        let mut board = EMPTY;
        for pos in iter {
            board |= BitBoard::from(pos);
        }
        board
    }
}

/// For the `BitBoard`, iterate over every `Position` set.
impl Iterator for BitBoard {
    type Item = Position;

    #[inline]
    fn next(&mut self) -> Option<Position> {
        if self.0 == 0 {
            None
        } else {
            let result = self.to_position();
            *self ^= BitBoard::from(result);
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard() {
        let one_end = BitBoard::from_position("a1");
        let other_end = BitBoard::from_position("h8");
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
