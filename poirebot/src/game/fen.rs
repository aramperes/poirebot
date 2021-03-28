use anyhow::Context;

use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, Pieces};
use crate::game::position::Position;
use crate::game::{Board, BoardSide};

/// The default starting position in FEN.
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Board {
    /// Convert a Forsyth–Edwards Notation (FEN) string to `Board`
    pub fn from_fen(fen: &str) -> anyhow::Result<Self> {
        if fen == "startpos" {
            return Self::from_fen(DEFAULT_FEN);
        }

        let mut fen_split = fen.split_whitespace();

        let piece_placement = fen_split.next().with_context(|| "no piece placement")?;
        let ranks: Vec<&str> = piece_placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(anyhow::Error::msg("piece placement does not have 8 ranks"));
        }

        let mut pieces = Vec::with_capacity(64);

        for (rank, rank_pieces) in ranks.into_iter().enumerate() {
            let rank = (7 - rank) as u8; // FEN starts with rank 8 (top to bottom from White's perspective)
            let mut file: u8 = 0;

            for piece in rank_pieces.chars() {
                if let Some(skip) = piece.to_digit(10) {
                    file += skip as u8;
                } else {
                    let color = if piece.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let piece = match piece.to_lowercase().next() {
                        Some('p') => Pieces::Pawn(color, (file, rank).into()),
                        Some('r') => Pieces::Rook(color, (file, rank).into()),
                        Some('n') => Pieces::Knight(color, (file, rank).into()),
                        Some('b') => Pieces::Bishop(color, (file, rank).into()),
                        Some('q') => Pieces::Queen(color, (file, rank).into()),
                        Some('k') => Pieces::King(color, (file, rank).into()),
                        _ => return Err(anyhow::Error::msg("invalid piece descriptor")),
                    };
                    pieces.push(piece);
                    file += 1;
                }
            }
            if file != 8 {
                return Err(anyhow::Error::msg("incomplete rank pieces"));
            }
        }

        // TODO
        let _active_color = if fen_split.next().with_context(|| "no active color")? == "w" {
            Color::White
        } else {
            Color::Black
        };

        let castling_availability = fen_split
            .next()
            .with_context(|| "no castling availability")?;
        let white_queenside_castle = castling_availability.contains('Q');
        let white_kingside_castle = castling_availability.contains('K');
        let black_queenside_castle = castling_availability.contains('q');
        let black_kingside_castle = castling_availability.contains('k');

        let en_passant_target = fen_split
            .next()
            .with_context(|| "no en passant target square")?;
        let en_passant_target: Option<(Position, Color)> = match en_passant_target {
            "-" => None,
            p => {
                let pos =
                    Position::from_notation(p).with_context(|| "invalid en passant target")?;
                let color = if pos.rank_y == 2 {
                    // 3rd rank
                    Color::White
                } else if pos.rank_y == 5 {
                    // 6th rank
                    Color::Black
                } else {
                    return Err(anyhow::Error::msg("impossible en passant target"));
                };
                Some((pos, color))
            }
        };

        // TODO
        let _half_move_clock = fen_split.next().with_context(|| "no half-move clock")?;

        // TODO
        let _full_move_clock = fen_split.next().with_context(|| "no full-move clock")?;

        // Construct board
        let white = BoardSide::new(Color::White, |side| {
            for piece in pieces.iter().filter(|p| p.is_white()) {
                match piece {
                    Pieces::Pawn(_, position) => side.pawns |= BitBoard::from(*position),
                    Pieces::Rook(_, position) => side.rooks |= BitBoard::from(*position),
                    Pieces::Knight(_, position) => side.knights |= BitBoard::from(*position),
                    Pieces::Bishop(_, position) => side.bishops |= BitBoard::from(*position),
                    Pieces::Queen(_, position) => side.queens |= BitBoard::from(*position),
                    Pieces::King(_, position) => side.king |= BitBoard::from(*position),
                }
            }
            if white_queenside_castle {
                side.unmoved_rooks |= BitBoard::from_position("a1".into());
            }
            if white_kingside_castle {
                side.unmoved_rooks |= BitBoard::from_position("h1".into());
            }
            if let Some((pos, color)) = en_passant_target {
                if color == Color::White {
                    side.en_passant_target |= BitBoard::from(pos.forwards(color, 1));
                }
            }
        });
        let black = BoardSide::new(Color::Black, |side| {
            for piece in pieces.iter().filter(|p| p.is_black()) {
                match piece {
                    Pieces::Pawn(_, position) => side.pawns |= BitBoard::from(*position),
                    Pieces::Rook(_, position) => side.rooks |= BitBoard::from(*position),
                    Pieces::Knight(_, position) => side.knights |= BitBoard::from(*position),
                    Pieces::Bishop(_, position) => side.bishops |= BitBoard::from(*position),
                    Pieces::Queen(_, position) => side.queens |= BitBoard::from(*position),
                    Pieces::King(_, position) => side.king |= BitBoard::from(*position),
                }
            }
            if black_queenside_castle {
                side.unmoved_rooks |= BitBoard::from_position("a8".into());
            }
            if black_kingside_castle {
                side.unmoved_rooks |= BitBoard::from_position("h8".into());
            }
            if let Some((pos, color)) = en_passant_target {
                if color == Color::Black {
                    side.en_passant_target |= BitBoard::from(pos.forwards(color, 1));
                }
            }
        });

        Ok(Self { white, black })
    }

    /// Convert a `Board` to Forsyth–Edwards Notation (FEN) string
    pub fn to_fen(&self) -> String {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fen() {
        let parsed = Board::from_fen(DEFAULT_FEN).expect("failed to parse FEN");
        assert_eq!(parsed, Board::default());
    }
}
