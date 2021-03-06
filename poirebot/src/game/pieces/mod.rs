use std::fmt::Debug;

use crate::bitboard::BitBoard;
use crate::game::position::Position;
use crate::game::Move;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;
pub mod sliding;

/// The A file.
#[allow(dead_code)]
pub const FILE_A: BitBoard =
    BitBoard(0b0000000100000001000000010000000100000001000000010000000100000001);

/// The B file
#[allow(dead_code)]
pub const FILE_B: BitBoard = BitBoard(FILE_A.0 << 1);

/// The C file
#[allow(dead_code)]
pub const FILE_C: BitBoard = BitBoard(FILE_A.0 << 2);

/// The D file
#[allow(dead_code)]
pub const FILE_D: BitBoard = BitBoard(FILE_A.0 << 3);

/// The E file
#[allow(dead_code)]
pub const FILE_E: BitBoard = BitBoard(FILE_A.0 << 4);

/// The F file
#[allow(dead_code)]
pub const FILE_F: BitBoard = BitBoard(FILE_A.0 << 5);

/// The G file
#[allow(dead_code)]
pub const FILE_G: BitBoard = BitBoard(FILE_A.0 << 6);

/// The H file
#[allow(dead_code)]
pub const FILE_H: BitBoard = BitBoard(FILE_A.0 << 7);

/// All the files.
#[allow(dead_code)]
pub const FILES: &[BitBoard; 8] = &[
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

/// The 1st rank
#[allow(dead_code)]
pub const RANK_1: BitBoard = BitBoard(0b11111111);

/// The 2nd rank
#[allow(dead_code)]
pub const RANK_2: BitBoard = BitBoard(RANK_1.0 << 8);

/// The 3rd rank
#[allow(dead_code)]
pub const RANK_3: BitBoard = BitBoard(RANK_2.0 << 8);

/// The 4th rank
#[allow(dead_code)]
pub const RANK_4: BitBoard = BitBoard(RANK_3.0 << 8);

/// The 4th rank
#[allow(dead_code)]
pub const RANK_5: BitBoard = BitBoard(RANK_4.0 << 8);

/// The 4th rank
#[allow(dead_code)]
pub const RANK_6: BitBoard = BitBoard(RANK_5.0 << 8);

/// The 4th rank
#[allow(dead_code)]
pub const RANK_7: BitBoard = BitBoard(RANK_6.0 << 8);

/// The 4th rank
#[allow(dead_code)]
pub const RANK_8: BitBoard = BitBoard(RANK_7.0 << 8);

/// All the ranks.
#[allow(dead_code)]
pub const RANKS: &[BitBoard; 8] = &[
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

/// The main diagonal, from a1 to h8
#[allow(dead_code)]
pub const MAIN_DIAGONAL: BitBoard = BitBoard(9241421688590303745);

/// The 15 diagonals parallel to the main diagonal. Starts from the bottom to the top,
/// with the main diagonal in the middle:
///   - index 0: h1
///   - index 1: g1 -> h2
///     ...
///   - index 7: a1 to h8 (main diagonal)
///   - index 8: a2 to g8
///     ...
///   - index 14: a8
/// A trick to get the diagonal matching a tile is to use the index `7 + rank - file` (0-indexed)
/// Or just use the provided `get_main_diagonal`.
#[allow(dead_code)]
pub const DIAGONALS: &[BitBoard; 15] = &[
    BitBoard(MAIN_DIAGONAL.0 >> (7 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> (6 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> (5 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> (4 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> (3 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> (2 * 8)),
    BitBoard(MAIN_DIAGONAL.0 >> 8),
    MAIN_DIAGONAL,
    BitBoard(MAIN_DIAGONAL.0 << 8),
    BitBoard(MAIN_DIAGONAL.0 << (2 * 8)),
    BitBoard(MAIN_DIAGONAL.0 << (3 * 8)),
    BitBoard(MAIN_DIAGONAL.0 << (4 * 8)),
    BitBoard(MAIN_DIAGONAL.0 << (5 * 8)),
    BitBoard(MAIN_DIAGONAL.0 << (6 * 8)),
    BitBoard(MAIN_DIAGONAL.0 << (7 * 8)),
];

/// The anti-diagonal, from a8 to h1
#[allow(dead_code)]
pub const ANTI_DIAGONAL: BitBoard = BitBoard(72624976668147840);

/// The 15 diagonals parallel to the anti diagonal. Starts from the bottom to the top,
/// with the main diagonal in the middle:
///   - index 0: a1
///   - index 1: a2 -> b1
///     ...
///   - index 7: a8 to h1 (anti diagonal)
///   - index 8: a2 to g8
///     ...
///   - index 14: h8
/// A trick to get the diagonal matching a tile is to use the index `rank + file` (0-indexed)
/// Or just use the provided `get_anti_diagonal`.
#[allow(dead_code)]
pub const ANTI_DIAGONALS: &[BitBoard; 15] = &[
    BitBoard(ANTI_DIAGONAL.0 >> (7 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> (6 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> (5 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> (4 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> (3 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> (2 * 8)),
    BitBoard(ANTI_DIAGONAL.0 >> 8),
    ANTI_DIAGONAL,
    BitBoard(ANTI_DIAGONAL.0 << 8),
    BitBoard(ANTI_DIAGONAL.0 << (2 * 8)),
    BitBoard(ANTI_DIAGONAL.0 << (3 * 8)),
    BitBoard(ANTI_DIAGONAL.0 << (4 * 8)),
    BitBoard(ANTI_DIAGONAL.0 << (5 * 8)),
    BitBoard(ANTI_DIAGONAL.0 << (6 * 8)),
    BitBoard(ANTI_DIAGONAL.0 << (7 * 8)),
];

/// Get the main-diagonal containing the given tile (file and rank, 0-indexed).
pub const fn get_main_diagonal(pos: &Position) -> &'static BitBoard {
    &DIAGONALS[(7 + pos.rank_y - pos.file_x) as usize]
}

/// Get the antidiagonal containing the given tile (file and rank, 0-indexed).
pub const fn get_anti_diagonal(pos: &Position) -> &'static BitBoard {
    &ANTI_DIAGONALS[(pos.file_x + pos.rank_y) as usize]
}

/// A chess piece.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pieces {
    Pawn(Color, Position),
    Rook(Color, Position),
    Knight(Color, Position),
    Bishop(Color, Position),
    Queen(Color, Position),
    King(Color, Position),
}

impl Pieces {
    /// Get the color of the piece.
    pub fn get_color(&self) -> Color {
        match self {
            Pieces::Pawn(color, _) => *color,
            Pieces::Rook(color, _) => *color,
            Pieces::Knight(color, _) => *color,
            Pieces::Bishop(color, _) => *color,
            Pieces::Queen(color, _) => *color,
            Pieces::King(color, _) => *color,
        }
    }

    /// Get the position of the piece.
    pub fn get_position(&self) -> Position {
        match self {
            Pieces::Pawn(_, position) => *position,
            Pieces::Rook(_, position) => *position,
            Pieces::Knight(_, position) => *position,
            Pieces::Bishop(_, position) => *position,
            Pieces::Queen(_, position) => *position,
            Pieces::King(_, position) => *position,
        }
    }

    /// Whether the piece is a `Pawn`.
    pub fn is_pawn(&self) -> bool {
        matches!(self, Pieces::Pawn(_, _))
    }

    /// Whether the piece is a `Rook`.
    pub fn is_rook(&self) -> bool {
        matches!(self, Pieces::Rook(_, _))
    }

    /// Whether the piece is a `Knight`.
    pub fn is_knight(&self) -> bool {
        matches!(self, Pieces::Knight(_, _))
    }

    /// Whether the piece is a `Bishop`.
    pub fn is_bishop(&self) -> bool {
        matches!(self, Pieces::Bishop(_, _))
    }

    /// Whether the piece is a `Queen`.
    pub fn is_queen(&self) -> bool {
        matches!(self, Pieces::Queen(_, _))
    }

    /// Whether the piece is a `King`.
    pub fn is_king(&self) -> bool {
        matches!(self, Pieces::King(_, _))
    }

    /// Whether the piece is black.
    pub fn is_black(&self) -> bool {
        matches!(self.get_color(), Color::Black)
    }

    /// Whether the piece is white.
    pub fn is_white(&self) -> bool {
        matches!(self.get_color(), Color::White)
    }

    /// Converts the piece type & color to a single letter. Uppercase is for white characters.
    pub fn to_letter_notation(&self) -> char {
        let c = match self {
            Pieces::Pawn(_, _) => 'p',
            Pieces::Rook(_, _) => 'r',
            Pieces::Knight(_, _) => 'n',
            Pieces::Bishop(_, _) => 'b',
            Pieces::Queen(_, _) => 'q',
            Pieces::King(_, _) => 'k',
        };
        if self.get_color() == Color::Black {
            c
        } else {
            c.to_ascii_uppercase()
        }
    }

    /// Returns the piece type & color in unicode.
    pub fn to_unicode_symbol(&self) -> char {
        match self {
            Pieces::Pawn(Color::White, _) => '???',
            Pieces::Pawn(Color::Black, _) => '???',
            Pieces::Rook(Color::White, _) => '???',
            Pieces::Rook(Color::Black, _) => '???',
            Pieces::Knight(Color::White, _) => '???',
            Pieces::Knight(Color::Black, _) => '???',
            Pieces::Bishop(Color::White, _) => '???',
            Pieces::Bishop(Color::Black, _) => '???',
            Pieces::Queen(Color::White, _) => '???',
            Pieces::Queen(Color::Black, _) => '???',
            Pieces::King(Color::White, _) => '???',
            Pieces::King(Color::Black, _) => '???',
        }
    }
}

/// When the king moves, find whether it was a castling action.
/// If it is a castling move, return the corresponding `Move` for the rook.
pub fn get_castling_rook_move(king_move: &Move) -> Option<Move> {
    if *king_move == ("e1", "c1").into() {
        // White Queenside (O-O-O)
        Some(("a1", "d1").into())
    } else if *king_move == ("e1", "g1").into() {
        // White Kingside (O-O)
        Some(("h1", "f1").into())
    } else if *king_move == ("e8", "c8").into() {
        // Black Queenside (O-O-O)
        Some(("a8", "d8").into())
    } else if *king_move == ("e8", "g8").into() {
        // Black Kingside (O-O)
        Some(("h8", "f8").into())
    } else {
        None
    }
}

/// Whether the move was a pawn "two-step" (when they move by 2 from their original position).
/// This function assumes the piece is already determined to be a pawn.
pub fn is_pawn_two_step(pawn_move: &Move) -> bool {
    let Move(origin, destination, _) = pawn_move;
    origin.distance_rank(destination) == 2 && (origin.rank_y == 1 || origin.rank_y == 6)
}

/// A chess piece set (white or black).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub fn is_black(&self) -> bool {
        *self == Color::Black
    }

    pub fn is_white(&self) -> bool {
        *self == Color::White
    }
}
