use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{Color, FILES, RANKS};
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

pub fn get_rook_north_moves_o2r(board: Board, color: Color) -> Vec<(Move, u8)> {
    let mut own_rooks = board.get_side(color).rooks;
    let mut all_pieces = board.black.pieces | board.white.pieces;
    let mut own_pieces = board.get_side(color).pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_rooks = own_rooks.rotate();
        all_pieces = all_pieces.rotate();
        own_pieces = own_pieces.rotate();
    }

    let mut moves = Vec::new();

    // o^(o-2r) method
    for rook in own_rooks {
        let file_mask: BitBoard = FILES[rook.file_x as usize];
        let blockers = all_pieces & file_mask;
        let difference = BitBoard(blockers.0 - BitBoard(BitBoard::from(rook).0 * 2).0);
        let changed = difference ^ all_pieces;
        let attacks = changed & file_mask & !own_pieces;

        if attacks.popcnt() > 0 {
            for attack in attacks {
                moves.push((Move(rook, attack, Promotion::None), 0));
            }
        }
    }

    moves
        .into_iter()
        .map(|(m, v)| {
            let mut m = m;

            if color == Color::Black {
                // De-normalize
                m.0 = BitBoard::from(m.0).rotate().to_position();
                m.1 = BitBoard::from(m.1).rotate().to_position();
            }

            (m, v)
        })
        .collect()
}

pub fn get_rook_north_moves(board: Board, color: Color) -> Vec<(Move, u8)> {
    let mut own_rooks = board.get_side(color).rooks;
    let mut all_pieces = board.black.pieces | board.white.pieces;
    let mut own_pieces = board.get_side(color).pieces;

    // Normalize by flipping
    if color == Color::Black {
        own_rooks = own_rooks.rotate();
        all_pieces = all_pieces.rotate();
        own_pieces = own_pieces.rotate();
    }

    let mut moves: Vec<Move> = Vec::with_capacity(own_rooks.popcnt() as usize * 7);

    for rook in own_rooks {
        // First generate the north mask from the rook
        let file_mask = FILES[rook.file_x as usize];
        let mut rank_mask = EMPTY;
        for i in (rook.rank_y + 1)..7 {
            rank_mask |= RANKS[i as usize];
        }
        let ray = file_mask & rank_mask;
        let blockers = ray & all_pieces;

        let first_blocker: Position = blockers.to_position();

        // Then generate the north mask from the first blocker
        let mut rank_mask = EMPTY;
        for i in (first_blocker.rank_y + 1..7) {
            rank_mask |= RANKS[i as usize];
        }

        // The potential moves are from the original ray, minus the north mask from the first blocker
        // minus own pieces to prevent eating our own
        let sliding = ray & !rank_mask & !own_pieces;
        for pos in sliding {
            moves.push(Move(rook, pos, Promotion::None));
        }
    }

    moves
        .into_iter()
        .map(|m| {
            let mut m = m;
            let mut v = 0;

            if color == Color::Black {
                // De-normalize
                m.0 = BitBoard::from(m.0).rotate().to_position();
                m.1 = BitBoard::from(m.1).rotate().to_position();
            }

            let target = m.1;
            if (BitBoard::from(target) & board.get_side(color.opposite()).pieces).popcnt() == 1 {
                v = board.get_piece_value(target);
            }

            (m, v)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::Board;

    #[test]
    fn test_get_rook_north_moves() {
        let board = Board::from_fen("2b1kbnr/rppppppp/n7/Pp5P/2P4q/R7/2PPPPP1/1NBQKBNR w Kk - 0 1")
            .unwrap();
        panic!("{:?}", super::get_rook_north_moves(board, Color::White));
    }
}
