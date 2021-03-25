use std::fmt::Debug;

use crate::game::position::Position;

/// A chess piece.
#[derive(Debug, Clone, Copy)]
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
}

/// A chess piece set (white or black).
#[derive(Debug, Clone, Copy, PartialEq)]
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
}
