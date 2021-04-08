use crate::bitboard::BitBoard;
use crate::game::pieces::{Color, FILES, RANKS};
use crate::game::position::Position;
use crate::game::Board;

/// The list of movement squares around a king, indexed by center square (0 = a1, 63 = h8).
/// Generated at compile-time.
const KING_MOVES: [BitBoard; 64] = compile_king_moves();

/// Generates a bitboard with the moves (steps) that can be performed by the king at the given position.
///
/// Note: this doesn't verify if the move is 100% legal, i.e. it could put the king in check.
/// However, it does prevent the king from capturing its own pieces.
///
/// Note: castling moves are not generated by this routine.
pub fn get_king_steps(board: &Board, color: Color, origin: Position) -> BitBoard {
    let side = board.get_side(color);
    let own_pieces = &side.pieces;
    let grid = &KING_MOVES[origin.to_int() as usize];
    grid & !own_pieces
}

/// Generates the BitBoard map for all possible king move grids.
const fn compile_king_moves() -> [BitBoard; 64] {
    let mut moves: [BitBoard; 64] = [BitBoard(0); 64];
    let mut i = 0usize;
    loop {
        // Generates the 64 possible positions for a king, with the surrounding steps
        // To generate the steps, it does the following bitwise operation:
        // (file | file-1 | file+1) & (rank | rank-1 | rank+1) &!center
        // ... where file-1 is the file directly West of the current one, if any
        // ... where file+1 is the file directly East of the current one, if any
        // ... where rank-1 is the rank directly South of the current one, if any
        // ... where rank+1 is the rank directly North of the current one, if any

        let file = i % 8_usize;
        let rank = i / 8_usize;
        let center = 1u64 << i;

        let mut files = FILES[file];
        let mut ranks = RANKS[rank];

        if file != 0 {
            files = BitBoard(files.0 | FILES[file - 1].0);
        }

        if file != 7 {
            files = BitBoard(files.0 | FILES[file + 1].0);
        }

        if rank != 0 {
            ranks = BitBoard(ranks.0 | RANKS[rank - 1].0);
        }

        if rank != 7 {
            ranks = BitBoard(ranks.0 | RANKS[rank + 1].0);
        }

        moves[i] = BitBoard(files.0 & ranks.0 & !center);

        i += 1;
        if i == 64 {
            break;
        }
    }
    moves
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::position::Position;
    use crate::game::Board;

    #[test]
    fn test_generate_king_moves() {
        let board =
            Board::from_fen("rnbqk2r/4Pppp/5bn1/4p3/8/p3P3/P1Q2PPP/RNB1KBNR b KQkq - 0 1").unwrap();
        let king = board.black.king.to_position();
        let moves = super::get_king_steps(&board, Color::Black, king);

        assert_eq!(
            moves.collect::<Vec<Position>>(),
            vec![
                Position::from("d7"),
                Position::from("e7"),
                Position::from("f8"),
            ]
        )
    }
}