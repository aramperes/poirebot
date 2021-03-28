use std::fmt::{Debug, Display, Formatter};

use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{get_castling_rook_move, is_pawn_two_step, Color, Pieces};
use crate::game::position::Position;

pub mod fen;
pub mod pieces;
pub mod position;

/// A chess piece move (origin and destination).
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Move(pub Position, pub Position, pub Promotion);

impl From<(&str, &str)> for Move {
    fn from(m: (&str, &str)) -> Self {
        Move(m.0.into(), m.1.into(), Promotion::None)
    }
}

impl From<(&str, &str, Promotion)> for Move {
    fn from(m: (&str, &str, Promotion)) -> Self {
        Move(m.0.into(), m.1.into(), m.2)
    }
}

impl From<(Position, Position)> for Move {
    fn from(m: (Position, Position)) -> Self {
        Move(m.0, m.1, Promotion::None)
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_pure_notation())
    }
}

impl Move {
    /// Convert a `Move` to Algebraic Pure Coordinate Notation.
    ///
    /// Ref: https://www.chessprogramming.org/Algebraic_Chess_Notation#Pure_coordinate_notation
    ///
    /// For example: `Move(a1, a2, Queen)` becomes `"a1a2q"`.
    pub fn to_pure_notation(&self) -> String {
        let Move(origin, destination, promotion) = self;
        format!("{}{}{}", origin, destination, promotion)
    }

    /// Convert a `Move` from Algebraic Pure Coordinate Notation.
    ///
    /// Ref: https://www.chessprogramming.org/Algebraic_Chess_Notation#Pure_coordinate_notation
    ///
    /// For example: `"a1a2q"` becomes `Move(a1, a2, Queen)`.
    pub fn from_pure_notation(notation: &str) -> Self {
        let origin = notation.chars().take(2).collect::<String>();
        let destination = notation.chars().skip(2).take(2).collect::<String>();
        let promotion = notation.chars().skip(3).take(1).collect::<String>();

        Move(origin.into(), destination.into(), promotion.into())
    }

    /// Returns a new `Move` with the given `Promotion`.
    pub fn with_promotion(&self, promotion: Promotion) -> Self {
        Move(self.0, self.1, promotion)
    }
}

/// A pawn promotion decision. Use `None` when there is no promotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

impl Default for Promotion {
    fn default() -> Self {
        Promotion::None
    }
}

impl Display for Promotion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queen => write!(f, "q"),
            Self::Rook => write!(f, "r"),
            Self::Bishop => write!(f, "b"),
            Self::Knight => write!(f, "n"),
            Self::None => Ok(()),
        }
    }
}

impl From<&str> for Promotion {
    fn from(p: &str) -> Self {
        match p {
            "q" => Self::Queen,
            "r" => Self::Rook,
            "b" => Self::Bishop,
            "n" => Self::Knight,
            _ => Self::None,
        }
    }
}

impl From<String> for Promotion {
    fn from(p: String) -> Self {
        Self::from(p.as_str())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Board {
    /// The `Color::White` board side.
    pub white: BoardSide,
    /// The `Color::Black` board side.
    pub black: BoardSide,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BoardSide {
    pub color: Color,
    /// Where all this side's pawns are.
    pub pawns: BitBoard,
    /// Where all this side's rooks are.
    pub rooks: BitBoard,
    /// Where all this side's knights are.
    pub knights: BitBoard,
    /// Where all this side's bishops are.
    pub bishops: BitBoard,
    /// Where all this side's queens are.
    pub queens: BitBoard,
    /// Where this side's king is.
    pub king: BitBoard,
    /// Rooks that haven't moved.
    pub unmoved_rooks: BitBoard,
    /// Pawn that could get en-passant'd in the next turn.
    pub en_passant_target: BitBoard,

    /// Inherited; Where all this side's pieces are.
    pub pieces: BitBoard,
    /// Inherited; squares attacked by this side's pieces.
    pub attacks: BitBoard,

    /// Whether the king has already moved.
    pub king_has_moved: bool,
}

impl BoardSide {
    /// Get the piece at the given position if any.
    pub fn get_piece(&self, position: Position) -> Option<Pieces> {
        let bb = BitBoard::from(position);

        if (bb & self.pawns).popcnt() == 1 {
            Some(Pieces::Pawn(self.color, position))
        } else if (bb & self.rooks).popcnt() == 1 {
            Some(Pieces::Rook(self.color, position))
        } else if (bb & self.knights).popcnt() == 1 {
            Some(Pieces::Knight(self.color, position))
        } else if (bb & self.bishops).popcnt() == 1 {
            Some(Pieces::Bishop(self.color, position))
        } else if (bb & self.queens).popcnt() == 1 {
            Some(Pieces::Queen(self.color, position))
        } else if (bb & self.king).popcnt() == 1 {
            Some(Pieces::King(self.color, position))
        } else {
            None
        }
    }

    /// Return a new instance for the other side.
    pub fn flip(&self) -> Self {
        Self {
            color: self.color.opposite(),
            pawns: self.pawns.reverse_colors(),
            rooks: self.rooks.reverse_colors(),
            knights: self.knights.reverse_colors(),
            bishops: self.bishops.reverse_colors(),
            queens: self.queens.reverse_colors(),
            king: self.king.reverse_colors(),
            unmoved_rooks: self.unmoved_rooks.reverse_colors(),
            pieces: self.pieces.reverse_colors(),
            attacks: self.attacks.reverse_colors(),
            en_passant_target: self.en_passant_target.reverse_colors(),
            king_has_moved: self.king_has_moved,
        }
    }

    /// Initialize a new side (empty at first) and then refresh inherited properties.
    pub fn new<F: FnOnce(&mut Self)>(color: Color, f: F) -> Self {
        let mut side = Self {
            color,
            pawns: EMPTY,
            rooks: EMPTY,
            knights: EMPTY,
            bishops: EMPTY,
            queens: EMPTY,
            king: EMPTY,
            unmoved_rooks: EMPTY,
            pieces: EMPTY,
            attacks: EMPTY,
            en_passant_target: EMPTY,
            king_has_moved: false,
        };
        side.mutate(f);
        side
    }

    /// Mutates and then refresh inherited properties.
    pub fn mutate<F: FnOnce(&mut Self)>(&mut self, f: F) {
        f(self);
        self.refresh();
    }

    /// Re-calculates inherited properties (attacks, pieces, etc.)
    fn refresh(&mut self) -> &mut Self {
        self.pieces =
            self.pawns | self.rooks | self.knights | self.bishops | self.queens | self.king;
        self.attacks = BitBoard::default(); // TODO: Calculate attacks
        self
    }
}

impl Board {
    /// Update the board after a player moved.
    pub fn apply_move(&mut self, m: Move) {
        let Move(origin, destination, promotion) = m;
        let origin_bb = BitBoard::from(origin);
        let destination_bb = BitBoard::from(destination);

        let piece_moved = self
            .get_piece(origin)
            .expect("moved something that doesn't exist");
        let color = piece_moved.get_color();

        let (mut side, mut opponent) = match color {
            Color::White => (self.white, self.black),
            Color::Black => (self.black, self.white),
        };
        let mut piece_taken = self.get_piece(destination);

        // Move the piece in the side
        match piece_moved {
            Pieces::Pawn(_, _) => {
                side.mutate(|side| {
                    side.pawns &= !origin_bb;
                    match promotion {
                        Promotion::None => side.pawns |= destination_bb,
                        Promotion::Queen => side.queens |= destination_bb,
                        Promotion::Rook => side.rooks |= destination_bb,
                        Promotion::Bishop => side.bishops |= destination_bb,
                        Promotion::Knight => side.knights |= destination_bb,
                    };

                    // Detect capture via en-passant
                    if opponent.en_passant_target.popcnt() == 1 {
                        let en_passant_target = opponent.en_passant_target.to_position();
                        if en_passant_target == destination.backwards(color, 1) {
                            // holy hell
                            piece_taken = self.get_piece(en_passant_target);
                        }
                    }

                    opponent.en_passant_target = EMPTY;

                    // When a pawn does a two-step, it becomes the en passant target
                    if is_pawn_two_step(&m) {
                        side.en_passant_target |= destination_bb;
                    }
                });
            }
            Pieces::Rook(_, _) => {
                side.mutate(|side| {
                    side.rooks &= !origin_bb;
                    side.rooks |= destination_bb;
                    side.unmoved_rooks &= !origin_bb;
                    opponent.en_passant_target = EMPTY;
                });
            }
            Pieces::Knight(_, _) => {
                side.mutate(|side| {
                    side.knights &= !origin_bb;
                    side.knights |= destination_bb;
                    opponent.en_passant_target = EMPTY;
                });
            }
            Pieces::Bishop(_, _) => {
                side.mutate(|side| {
                    side.bishops &= !origin_bb;
                    side.bishops |= destination_bb;
                    opponent.en_passant_target = EMPTY;
                });
            }
            Pieces::Queen(_, _) => {
                side.mutate(|side| {
                    side.queens &= !origin_bb;
                    side.queens |= destination_bb;
                    opponent.en_passant_target = EMPTY;
                });
            }
            Pieces::King(_, _) => {
                side.mutate(|side| {
                    side.king &= !origin_bb;
                    side.king |= destination_bb;
                    opponent.en_passant_target = EMPTY;
                    side.king_has_moved = true;

                    // Check if it was a castling move
                    if let Some(Move(rook_origin, rook_destination, _)) = get_castling_rook_move(&m)
                    {
                        // Move the rook
                        let rook_origin_bb = BitBoard::from(rook_origin);
                        let rook_destination_b = BitBoard::from(rook_destination);

                        // Ensure we are actually moving a rook
                        assert_eq!((rook_origin_bb & side.rooks).popcnt(), 1);
                        // Ensure the rook there hasn't moved before
                        assert_eq!((rook_origin_bb & side.unmoved_rooks).popcnt(), 1);

                        side.rooks &= !rook_origin_bb;
                        side.rooks |= rook_destination_b;
                        side.unmoved_rooks &= !rook_origin_bb;
                    }
                });
            }
        }

        // If a piece was taken, remove it from the opponent's side
        if let Some(piece_taken) = piece_taken {
            if piece_taken.get_color() == piece_moved.get_color() {
                panic!(
                    "Tried to friendly-fire: {:?} took {:?}",
                    piece_moved, piece_taken
                );
            } else {
                let remove_bb = BitBoard::from(piece_taken.get_position());
                match piece_taken {
                    Pieces::Pawn(_, _) => {
                        opponent.mutate(|opponent| opponent.pawns &= !remove_bb);
                    }
                    Pieces::Rook(_, _) => {
                        opponent.mutate(|opponent| opponent.rooks &= !remove_bb);
                    }
                    Pieces::Knight(_, _) => {
                        opponent.mutate(|opponent| opponent.knights &= !remove_bb);
                    }
                    Pieces::Bishop(_, _) => {
                        opponent.mutate(|opponent| opponent.bishops &= !remove_bb);
                    }
                    Pieces::Queen(_, _) => {
                        opponent.mutate(|opponent| opponent.queens &= !remove_bb);
                    }
                    Pieces::King(_, _) => {
                        opponent.mutate(|opponent| opponent.king &= !remove_bb);
                    }
                }
            }
        }

        match color {
            Color::White => {
                self.white = side;
                self.black = opponent;
            }
            Color::Black => {
                self.black = side;
                self.white = opponent;
            }
        }
    }

    /// Get a list of pawns of the given color.
    pub fn get_pawns(&self, color: Color) -> Vec<Pieces> {
        let side = self.get_side(color);
        side.pawns
            .into_iter()
            .map(move |position| Pieces::Pawn(color, position))
            .collect()
    }

    /// Get the piece at the given position if any.
    pub fn get_piece(&self, position: Position) -> Option<Pieces> {
        self.white
            .get_piece(position)
            .or_else(|| self.black.get_piece(position))
    }

    /// Get the piece at the given position if any.
    pub fn get_piece_value(&self, position: Position) -> u8 {
        match self.get_piece(position) {
            Some(Pieces::Pawn(_, _)) => 1,
            Some(Pieces::Bishop(_, _)) => 3,
            Some(Pieces::Knight(_, _)) => 3,
            Some(Pieces::Rook(_, _)) => 5,
            Some(Pieces::Queen(_, _)) => 8,
            Some(Pieces::King(_, _)) => 100,
            None => 0,
        }
    }

    /// Returns a bitboard for all the pieces in the board.
    pub fn get_bitboard(&self) -> BitBoard {
        self.white.pieces | self.black.pieces
    }

    /// Mutates and then refresh inherited properties.
    pub fn mutate<F: FnOnce(&mut Self)>(&mut self, f: F) {
        f(self);
        self.white.refresh();
        self.black.refresh();
    }

    pub fn get_side(&self, color: Color) -> BoardSide {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    pub fn get_side_mut(&mut self, color: Color) -> &mut BoardSide {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        let white = BoardSide::new(Color::White, |side| {
            side.pawns = BitBoard::from_position("a2".into())
                | BitBoard::from_position("b2".into())
                | BitBoard::from_position("c2".into())
                | BitBoard::from_position("d2".into())
                | BitBoard::from_position("e2".into())
                | BitBoard::from_position("f2".into())
                | BitBoard::from_position("g2".into())
                | BitBoard::from_position("h2".into());
            side.rooks =
                BitBoard::from_position("a1".into()) | BitBoard::from_position("h1".into());
            side.knights =
                BitBoard::from_position("b1".into()) | BitBoard::from_position("g1".into());
            side.bishops =
                BitBoard::from_position("c1".into()) | BitBoard::from_position("f1".into());
            side.queens = BitBoard::from_position("d1".into());
            side.king = BitBoard::from_position("e1".into());
            side.unmoved_rooks = side.rooks;
        });

        let black = white.flip();

        Self { white, black }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_board() {
        let board = Board::default();

        // White Pawns
        assert!(board.get_piece("a2".into()).unwrap().is_pawn());
        assert!(board.get_piece("a2".into()).unwrap().is_white());
        assert!(board.get_piece("b2".into()).unwrap().is_pawn());
        assert!(board.get_piece("b2".into()).unwrap().is_white());
        assert!(board.get_piece("c2".into()).unwrap().is_pawn());
        assert!(board.get_piece("c2".into()).unwrap().is_white());
        assert!(board.get_piece("d2".into()).unwrap().is_pawn());
        assert!(board.get_piece("d2".into()).unwrap().is_white());
        assert!(board.get_piece("e2".into()).unwrap().is_pawn());
        assert!(board.get_piece("e2".into()).unwrap().is_white());
        assert!(board.get_piece("f2".into()).unwrap().is_pawn());
        assert!(board.get_piece("f2".into()).unwrap().is_white());
        assert!(board.get_piece("g2".into()).unwrap().is_pawn());
        assert!(board.get_piece("g2".into()).unwrap().is_white());
        assert!(board.get_piece("h2".into()).unwrap().is_pawn());
        assert!(board.get_piece("h2".into()).unwrap().is_white());

        // White Rooks
        assert!(board.get_piece("a1".into()).unwrap().is_rook());
        assert!(board.get_piece("a1".into()).unwrap().is_white());
        assert!(board.get_piece("h1".into()).unwrap().is_rook());
        assert!(board.get_piece("h1".into()).unwrap().is_white());

        // White Knights
        assert!(board.get_piece("b1".into()).unwrap().is_knight());
        assert!(board.get_piece("b1".into()).unwrap().is_white());
        assert!(board.get_piece("g1".into()).unwrap().is_knight());
        assert!(board.get_piece("g1".into()).unwrap().is_white());

        // White Bishops
        assert!(board.get_piece("c1".into()).unwrap().is_bishop());
        assert!(board.get_piece("c1".into()).unwrap().is_white());
        assert!(board.get_piece("f1".into()).unwrap().is_bishop());
        assert!(board.get_piece("f1".into()).unwrap().is_white());

        // White Queen
        assert!(board.get_piece("d1".into()).unwrap().is_queen());
        assert!(board.get_piece("d1".into()).unwrap().is_white());

        // White King
        assert!(board.get_piece("e1".into()).unwrap().is_king());
        assert!(board.get_piece("e1".into()).unwrap().is_white());

        // Black Pawns
        assert!(board.get_piece("a7".into()).unwrap().is_pawn());
        assert!(board.get_piece("a7".into()).unwrap().is_black());
        assert!(board.get_piece("b7".into()).unwrap().is_pawn());
        assert!(board.get_piece("b7".into()).unwrap().is_black());
        assert!(board.get_piece("c7".into()).unwrap().is_pawn());
        assert!(board.get_piece("c7".into()).unwrap().is_black());
        assert!(board.get_piece("d7".into()).unwrap().is_pawn());
        assert!(board.get_piece("d7".into()).unwrap().is_black());
        assert!(board.get_piece("e7".into()).unwrap().is_pawn());
        assert!(board.get_piece("e7".into()).unwrap().is_black());
        assert!(board.get_piece("f7".into()).unwrap().is_pawn());
        assert!(board.get_piece("f7".into()).unwrap().is_black());
        assert!(board.get_piece("g7".into()).unwrap().is_pawn());
        assert!(board.get_piece("g7".into()).unwrap().is_black());
        assert!(board.get_piece("h7".into()).unwrap().is_pawn());
        assert!(board.get_piece("h7".into()).unwrap().is_black());

        // Black Rooks
        assert!(board.get_piece("a8".into()).unwrap().is_rook());
        assert!(board.get_piece("a8".into()).unwrap().is_black());
        assert!(board.get_piece("h8".into()).unwrap().is_rook());
        assert!(board.get_piece("h8".into()).unwrap().is_black());

        // Black Knights
        assert!(board.get_piece("b8".into()).unwrap().is_knight());
        assert!(board.get_piece("b8".into()).unwrap().is_black());
        assert!(board.get_piece("g8".into()).unwrap().is_knight());
        assert!(board.get_piece("g8".into()).unwrap().is_black());

        // Black Bishops
        assert!(board.get_piece("c8".into()).unwrap().is_bishop());
        assert!(board.get_piece("c8".into()).unwrap().is_black());
        assert!(board.get_piece("f8".into()).unwrap().is_bishop());
        assert!(board.get_piece("f8".into()).unwrap().is_black());

        // Black Queen
        assert!(board.get_piece("d8".into()).unwrap().is_queen());
        assert!(board.get_piece("d8".into()).unwrap().is_black());

        // Black King
        assert!(board.get_piece("e8".into()).unwrap().is_king());
        assert!(board.get_piece("e8".into()).unwrap().is_black());
    }

    #[test]
    fn move_and_capture_and_promote() {
        let mut board = Board::default();

        // Move white pawn and take b7 pawn
        board.apply_move(("a2", "a4").into());
        board.apply_move(("a4", "a5").into());
        board.apply_move(("a5", "a6").into());
        board.apply_move(("a6", "b7").into());

        let pawn = board.get_piece("b7".into()).expect("should have piece");
        assert_eq!(pawn.get_color(), Color::White);
        assert!(pawn.is_pawn());

        // Then take rook, which should promote
        board.apply_move(("b7", "a8", Promotion::Queen).into());

        let promoted = board.get_piece("a8".into()).expect("should have piece");
        assert_eq!(promoted.get_color(), Color::White);
        assert!(promoted.is_queen());
    }

    #[test]
    fn test_queenside_castling() {
        // Initialize with a board that has the queenside rooks and kings only.
        let board_fen = "r3k3/8/8/8/8/8/8/R3K3 w Qq - 0 1";
        let mut board = Board::from_fen(board_fen).unwrap();

        // White does queenside castle
        board.apply_move(("e1", "c1").into());
        assert_eq!(
            board.get_piece("c1".into()),
            Some(Pieces::King(Color::White, "c1".into()))
        );
        assert_eq!(
            board.get_piece("d1".into()),
            Some(Pieces::Rook(Color::White, "d1".into()))
        );

        // Black does queenside castle
        board.apply_move(("e8", "c8").into());
        assert_eq!(
            board.get_piece("c8".into()),
            Some(Pieces::King(Color::Black, "c8".into()))
        );
        assert_eq!(
            board.get_piece("d8".into()),
            Some(Pieces::Rook(Color::Black, "d8".into()))
        );
    }

    #[test]
    fn test_kingside_castling() {
        // Initialize with a board that has the kingside rooks and kings only.
        let board_fen = "4k2r/8/8/8/8/8/8/4K2R w Kk - 0 1";
        let mut board = Board::from_fen(board_fen).unwrap();

        // White does kingside castle
        board.apply_move(("e1", "g1").into());
        assert_eq!(
            board.get_piece("g1".into()),
            Some(Pieces::King(Color::White, "g1".into()))
        );
        assert_eq!(
            board.get_piece("f1".into()),
            Some(Pieces::Rook(Color::White, "f1".into()))
        );

        // Black does kingside castle
        board.apply_move(("e8", "g8").into());
        assert_eq!(
            board.get_piece("g8".into()),
            Some(Pieces::King(Color::Black, "g8".into()))
        );
        assert_eq!(
            board.get_piece("f8".into()),
            Some(Pieces::Rook(Color::Black, "f8".into()))
        );
    }

    #[test]
    fn test_black_en_passant() {
        // Initialize with a board with imminent en passant by black
        let board_fen = "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let mut board = Board::from_fen(board_fen).unwrap();

        // Do the en passant
        board.apply_move(("d4", "e3").into());
        assert_eq!(
            board.get_piece("e3".into()),
            Some(Pieces::Pawn(Color::Black, "e3".into()))
        );
        assert_eq!(board.get_piece("e4".into()), None);
    }

    #[test]
    fn test_white_en_passant() {
        // Initialize with a board with imminent en passant by white
        let board_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
        let mut board = Board::from_fen(board_fen).unwrap();

        // Do the en passant
        board.apply_move(("e5", "d6").into());
        assert_eq!(
            board.get_piece("d6".into()),
            Some(Pieces::Pawn(Color::White, "d6".into()))
        );
        assert_eq!(board.get_piece("d5".into()), None);
    }

    #[test]
    fn test_squares_attacked_by_pawns() {
        // White pieces attacking
        let board = Board::from_fen("8/8/8/8/p2p3p/1PP1P1P1/8/8 b - - 0 1").unwrap();
        let attacked = board.get_squares_attacked_by_pawns(Color::Black);
        assert_eq!(
            attacked
                .into_iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<String>>(),
            vec!["b3", "c3", "e3", "g3"]
        );
        // Black pieces attacking
        let board = Board::from_fen("8/8/8/1pp1p1p1/P2P3P/8/8/8 w - - 0 1").unwrap();
        let attacked = board.get_squares_attacked_by_pawns(Color::White);
        assert_eq!(
            attacked
                .into_iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<String>>(),
            vec!["b5", "c5", "e5", "g5"]
        );
    }

    #[test]
    fn test_pawn_double_step_white() {
        let board = Board::default();
        let double_steps = board.get_pawn_double_steps(Color::White);
        assert_eq!(
            double_steps,
            vec![
                ("a2", "a4").into(),
                ("b2", "b4").into(),
                ("c2", "c4").into(),
                ("d2", "d4").into(),
                ("e2", "e4").into(),
                ("f2", "f4").into(),
                ("g2", "g4").into(),
                ("h2", "h4").into(),
            ]
        );
    }

    #[test]
    fn test_pawn_double_step_black() {
        let board = Board::default();
        let double_steps = board.get_pawn_double_steps(Color::Black);
        assert_eq!(
            double_steps,
            vec![
                ("a7", "a5").into(),
                ("b7", "b5").into(),
                ("c7", "c5").into(),
                ("d7", "d5").into(),
                ("e7", "e5").into(),
                ("f7", "f5").into(),
                ("g7", "g5").into(),
                ("h7", "h5").into(),
            ]
        );
    }

    #[test]
    fn test_pawn_double_step_obstructed() {
        let board =
            Board::from_fen("rn2kbnr/pp2pppp/2p5/8/1b3P2/3qP1Pp/PPPP3P/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let double_steps = board.get_pawn_double_steps(Color::White);
        assert_eq!(
            double_steps,
            vec![("a2", "a4").into(), ("c2", "c4").into(),]
        );
    }
}
