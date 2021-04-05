use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILES};
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

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

    // East
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

    // East
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
