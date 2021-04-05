use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILES};
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

/// The main diagonal, from a1 to h8
const MAIN_DIAGONAL: BitBoard = BitBoard(9241421688590303745);

/// The anti-diagonal, from a8 to h1
#[allow(dead_code)]
const ANTI_DIAGONAL: BitBoard = BitBoard(72624976668147840);

/// Generates moves that slide vertically (north/south) from the original position.
/// Also returns the worth of the piece being taken, if any.
pub fn get_sliding_vertical_moves(board: Board, color: Color, origin: Position) -> Vec<(Move, u8)> {
    let real_origin = origin;

    let mut all_pieces = board.black.pieces | board.white.pieces;
    let mut own_pieces = board.get_side(color).pieces;

    let mut moves = Vec::new();
    let mut origin = origin;
    let mut origin_bb = BitBoard::from(origin);

    // North (from white's perspective)
    let file_mask: BitBoard = FILES[origin.file_x as usize];
    let blockers = all_pieces & file_mask;
    let difference = BitBoard(
        blockers
            .0
            .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
    );
    let changed = difference ^ all_pieces;
    let mut attacks = changed & file_mask & !own_pieces;
    while attacks.popcnt() > 0 {
        let attack = attacks.next().unwrap();
        if attacks.popcnt() == 0 {
            let val = board.get_piece_value(attack);
            moves.push((Move(real_origin, attack, Promotion::None), val));
        } else {
            moves.push((Move(real_origin, attack, Promotion::None), 0));
        }
    }

    // South (from white's perspective)
    origin = origin.flip();
    origin_bb = origin_bb.reverse_colors();
    all_pieces = all_pieces.reverse_colors();
    own_pieces = own_pieces.reverse_colors();

    let file_mask: BitBoard = FILES[origin.file_x as usize];
    let blockers = all_pieces & file_mask;
    let difference = BitBoard(
        blockers
            .0
            .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
    );
    let changed = difference ^ all_pieces;
    let mut attacks = changed & file_mask & !own_pieces;
    while attacks.popcnt() > 0 {
        let attack = attacks.next().unwrap().flip();
        if attacks.popcnt() == 0 {
            let val = board.get_piece_value(attack);
            moves.push((Move(real_origin, attack, Promotion::None), val));
        } else {
            moves.push((Move(real_origin, attack, Promotion::None), 0));
        }
    }

    moves
}

/// Generates moves that slide towards the east from the original position.
/// Also returns the worth of the piece being taken, if any.
pub fn get_sliding_east_moves(board: Board, color: Color, origin: Position) -> Vec<(Move, u8)> {
    let real_origin = origin;

    // Quick-return when we're at the eastern edge already
    if real_origin.file_x == 7 {
        return Vec::new();
    }

    let mut all_pieces = board.black.pieces | board.white.pieces;
    let mut own_pieces = board.get_side(color).pieces;

    all_pieces = all_pieces.flip_diagonally();
    own_pieces = own_pieces.flip_diagonally();

    let origin_bb = BitBoard::from(origin).flip_diagonally();
    let origin = origin_bb.to_position();

    let mut moves = Vec::new();

    let file_mask: BitBoard = FILES[origin.file_x as usize];
    let blockers = all_pieces & file_mask;
    let difference = BitBoard(
        blockers
            .0
            .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
    );
    let changed = difference ^ all_pieces;
    let attacks = changed & file_mask & !own_pieces;
    let mut attacks = attacks;
    while attacks.popcnt() > 0 {
        let attack = attacks.next().unwrap();
        let attack = BitBoard::from(attack).flip_diagonally().to_position();
        if attacks.popcnt() == 0 {
            let val = board.get_piece_value(attack);
            moves.push((Move(real_origin, attack, Promotion::None), val));
        } else {
            moves.push((Move(real_origin, attack, Promotion::None), 0));
        }
    }

    moves
}

/// Generates moves that slide towards the west from the original position.
/// Also returns the worth of the piece being taken, if any.
pub fn get_sliding_west_moves(board: Board, color: Color, origin: Position) -> Vec<(Move, u8)> {
    let real_origin = origin;

    // Quick-return when we're at the western edge already
    if real_origin.file_x == 0 {
        return Vec::new();
    }

    let mut all_pieces = board.black.pieces | board.white.pieces;
    let mut own_pieces = board.get_side(color).pieces;

    all_pieces = all_pieces.flip_anti_diagonally();
    own_pieces = own_pieces.flip_anti_diagonally();

    let origin_bb = BitBoard::from(origin).flip_anti_diagonally();
    let origin = origin_bb.to_position();

    let mut moves = Vec::new();

    let file_mask: BitBoard = FILES[origin.file_x as usize];
    let blockers = all_pieces & file_mask;
    let difference = BitBoard(
        blockers
            .0
            .wrapping_sub(BitBoard(origin_bb.0.wrapping_mul(2)).0),
    );
    let changed = difference ^ all_pieces;
    let attacks = changed & file_mask & !own_pieces;
    let mut attacks = attacks;
    while attacks.popcnt() > 0 {
        let attack = attacks.next().unwrap();
        let attack = BitBoard::from(attack).flip_anti_diagonally().to_position();
        if attacks.popcnt() == 0 {
            let val = board.get_piece_value(attack);
            moves.push((Move(real_origin, attack, Promotion::None), val));
        } else {
            moves.push((Move(real_origin, attack, Promotion::None), 0));
        }
    }

    moves
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
