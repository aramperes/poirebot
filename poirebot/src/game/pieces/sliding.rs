use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{Color, FILES, MAIN_DIAGONAL, RANKS};
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

/// Generates a bitboard with the moves that can be performed by the sliding pieces in the given bitboard, horizontally and vertically.
///
/// Note that multiple pieces can be passed in the bitboard; to get the moves for individual pieces,
/// iterate over the pieces and call this function with the singular bitboard for each piece.
pub fn get_sliding_straight_moves(board: &Board, color: Color, origins: &BitBoard) -> BitBoard {
    let mut result = EMPTY;

    let all_pieces = board.white.pieces | board.black.pieces;
    let side = board.get_side(color);
    let own_pieces = side.pieces;

    // From a high-level, the operation here is to subtract the piece from the occupancy
    // See: https://www.chessprogramming.org/Subtracting_a_Rook_from_a_Blocking_Piece
    // The formula is o^(o-2r), and only applies to "positive rays" (moves with positive rank and/or file difference)
    // NB: The formula only works on bitboards with 1 square, so we must do 1 piece at a time
    for origin in *origins {
        let file_mask: BitBoard = FILES[origin.file_x as usize];
        let rank_mask: BitBoard = RANKS[origin.rank_y as usize];

        // Start with North (positive vertical)
        // First we find the potential blockers on the same file (vertical)
        let blockers = all_pieces & file_mask;

        // The (o-2r) portion gets the squares between the piece and the closest blocker, excluding the blocker itself
        let square_board = BitBoard::from(origin);
        let difference = blockers - (square_board * 2);

        // The XOR (occupancy ^ difference) portion gives us the tiles between the closest blocker and the piece
        // that aren't occupied. It includes the closest blocker because we intentionally left it out during the
        // (o-2r) portion. We also re-use the file mask because we're not interested in the other pieces.
        // Lastly, we don't want to count the blocker if it's our own piece (it's a capture otherwise).
        result |= (all_pieces ^ difference) & file_mask & !own_pieces;

        // For South (negative vertical), we'll do the same formula but after swapping the board direction.
        // Note: this is a swap, not a rotation, because the file is preserved.
        {
            let blockers = blockers.swap();
            let all_pieces = all_pieces.swap();
            let own_pieces = own_pieces.swap();
            let square_board = square_board.swap();

            let difference = blockers - (square_board * 2);
            result |= ((all_pieces ^ difference) & file_mask & !own_pieces).swap();
        }

        // For East (positive horizontal), it's the same as the North formula, but with the rank mask instead
        let blockers = all_pieces & rank_mask;
        let difference = blockers - (square_board * 2);
        result |= (all_pieces ^ difference) & rank_mask & !own_pieces;

        // For West (negative horizontal), it's the same as the East formula, but we mirror the board horizontally
        // Note: this is a mirror, not a rotation, because the rank is preserved
        let blockers = blockers.mirror_horizontally();
        let all_pieces = all_pieces.mirror_horizontally();
        let own_pieces = own_pieces.mirror_horizontally();
        let square_board = square_board.mirror_horizontally();

        let difference = blockers - (square_board * 2);
        result |= ((all_pieces ^ difference) & rank_mask & !own_pieces).mirror_horizontally();
    }

    result
}

/// Generates moves that slide towards parallel to (or on) the main diagonal.
/// Also returns the worth of the piece being taken, if any.
pub fn get_sliding_main_diagonal_moves(
    board: Board,
    color: Color,
    origin: Position,
) -> Vec<(Move, u8)> {
    let real_origin = origin;
    let origin_bb = BitBoard::from(origin);

    let all_pieces = board.black.pieces | board.white.pieces;
    let own_pieces = board.get_side(color).pieces;

    let mut moves = Vec::new();

    // Get the diagonal parallel to the main diagonal
    let diagonal_diff = (origin.rank_y as i32 - origin.file_x as i32) * 8;
    let diagonal_mask = if diagonal_diff > 0 {
        BitBoard(MAIN_DIAGONAL.0 << diagonal_diff as u8)
    } else {
        BitBoard(MAIN_DIAGONAL.0 >> (-diagonal_diff) as u8)
    };

    if real_origin.rank_y != 7 {
        let blockers = all_pieces & diagonal_mask;
        let difference = BitBoard(
            blockers
                .0
                .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
        );
        let changed = difference ^ all_pieces;
        let attacks = changed & diagonal_mask & !own_pieces;
        let mut attacks = attacks;
        while attacks.popcnt() > 0 {
            let attack = attacks.next().unwrap();
            let attack = BitBoard::from(attack).to_position();
            if attacks.popcnt() == 0 {
                let val = board.get_piece_value(attack);
                moves.push((Move(real_origin, attack, Promotion::None), val));
            } else {
                moves.push((Move(real_origin, attack, Promotion::None), 0));
            }
        }
    }

    if real_origin.rank_y != 0 {
        let all_pieces = all_pieces.rotate();
        let own_pieces = own_pieces.rotate();
        let diagonal_mask = diagonal_mask.rotate();
        let blockers = all_pieces & diagonal_mask;
        let origin_bb = origin_bb.rotate();

        let difference = BitBoard(
            blockers
                .0
                .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
        );
        let changed = difference ^ all_pieces;
        let attacks = changed & diagonal_mask & !own_pieces;
        let mut attacks = attacks;
        while attacks.popcnt() > 0 {
            let attack = attacks.next().unwrap();
            let attack = BitBoard::from(attack).rotate().to_position();
            if attacks.popcnt() == 0 {
                let val = board.get_piece_value(attack);
                moves.push((Move(real_origin, attack, Promotion::None), val));
            } else {
                moves.push((Move(real_origin, attack, Promotion::None), 0));
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::{Board, Move};

    #[test]
    fn test_sliding_main_diagonal_north_moves() {
        let board = Board::from_fen("7P/6P1/5P2/4P1R1/3P4/2P1b3/1P6/P1r5 w - - 0 1").unwrap();
        let moves = super::get_sliding_main_diagonal_moves(board, Color::Black, "e3".into());
        assert_eq!(
            moves,
            vec![
                (Move::from_pure_notation("e3f4"), 0),
                (Move::from_pure_notation("e3g5"), 5),
                (Move::from_pure_notation("e3d2"), 0),
            ]
        );
    }
}
