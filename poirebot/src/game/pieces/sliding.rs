use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{get_anti_diagonal, get_main_diagonal, Color, FILES, RANKS};
use crate::game::Board;

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
        let file_mask = &FILES[origin.file_x as usize];
        let rank_mask = &RANKS[origin.rank_y as usize];

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

/// Generates a bitboard with the diagonal moves that can be performed by the sliding pieces in the given bitboard.
///
/// Note that multiple pieces can be passed in the bitboard; to get the moves for individual pieces,
/// iterate over the pieces and call this function with the singular bitboard for each piece.
pub fn get_sliding_diagonal_moves(board: &Board, color: Color, origins: &BitBoard) -> BitBoard {
    let mut result = EMPTY;
    let all_pieces = board.white.pieces | board.black.pieces;
    let side = board.get_side(color);
    let own_pieces = side.pieces;

    // From a high-level, the operation here is to subtract the piece from the occupancy
    // The formula is o^(o-2r), and only applies to "positive rays" (moves with positive rank and/or file difference)
    // See the `get_sliding_straight_moves` function above for details about the formula
    // The difference with diagonals is simply the masks, which are based on the main diagonal and the anti-diagonal
    // NB: The formula only works on bitboards with 1 square, so we must do 1 piece at a time
    for origin in *origins {
        let main_mask = get_main_diagonal(&origin);
        let anti_mask = get_anti_diagonal(&origin);

        // Start with the positive-main diagonal (North-East)
        let blockers = all_pieces & main_mask;
        let square_board = BitBoard::from(origin);
        let difference = blockers - (square_board * 2);
        result |= (all_pieces ^ difference) & main_mask & !own_pieces;

        // Then positive-anti diagonal (North-West)
        {
            let blockers = all_pieces & anti_mask;
            let difference = blockers - (square_board * 2);
            result |= (all_pieces ^ difference) & anti_mask & !own_pieces;

            // Then negative-anti diagonal (South-East)
            let blockers = blockers.swap();
            let all_pieces = all_pieces.swap();
            let own_pieces = own_pieces.swap();
            let square_board = square_board.swap();

            let difference = blockers - (square_board * 2);
            result |= ((all_pieces ^ difference) & anti_mask.swap() & !own_pieces).swap();
        }

        // Then negative-main diagonal (South-West)
        let blockers = blockers.swap();
        let all_pieces = all_pieces.swap();
        let own_pieces = own_pieces.swap();
        let square_board = square_board.swap();

        let difference = blockers - (square_board * 2);
        result |= ((all_pieces ^ difference) & main_mask.swap() & !own_pieces).swap();
    }
    result
}
