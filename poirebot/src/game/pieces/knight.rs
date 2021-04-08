use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILE_A, FILE_B, FILE_G, FILE_H};
use crate::game::position::Position;
use crate::game::Board;

/// The list of movement squares around a knight, indexed by center square (0 = a1, 63 = h8).
/// Generated at compile-time.
const KNIGHT_MOVES: [BitBoard; 64] = compile_knight_moves();

/// Generates a bitboard with the moves (steps) that can be performed by the knight at the given position.
///
/// Note: this doesn't verify if the move is 100% legal, i.e. it could put the knight in check.
/// However, it does prevent the knight from capturing its own pieces.
pub fn get_knight_moves(board: &Board, color: Color, origin: Position) -> BitBoard {
    let side = board.get_side(color);
    let own_pieces = &side.pieces;
    let grid = &KNIGHT_MOVES[origin.to_int() as usize];
    grid & !own_pieces
}

/// Generates the BitBoard map for all possible knight move grids.
const fn compile_knight_moves() -> [BitBoard; 64] {
    let mut moves: [BitBoard; 64] = [BitBoard(0); 64];
    let mut i = 0usize;
    loop {
        // Generates the 64 possible positions for a knight, with the possible moves
        let center = 1u64 << i;

        // North-North-East
        moves[i].0 |= (center << 17) & !FILE_A.0;

        // North-East-East
        moves[i].0 |= (center << 10) & !FILE_A.0 & !FILE_B.0;

        // South-East-East
        moves[i].0 |= (center >> 6) & !FILE_A.0 & !FILE_B.0;

        // South-South-East
        moves[i].0 |= (center >> 15) & !FILE_A.0;

        // South-South-West
        moves[i].0 |= (center >> 17) & !FILE_H.0;

        // South-West-West
        moves[i].0 |= (center >> 10) & !FILE_H.0 & !FILE_G.0;

        // North-West-West
        moves[i].0 |= (center << 6) & !FILE_H.0 & !FILE_G.0;

        // North-North-West
        moves[i].0 |= (center << 15) & !FILE_H.0;

        i += 1;
        if i == 64 {
            break;
        }
    }
    moves
}
