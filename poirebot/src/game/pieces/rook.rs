use crate::bitboard::{BitBoard, EMPTY};
use crate::game::pieces::{Color, FILES, RANKS};
use crate::game::position::Position;
use crate::game::{Board, Move, Promotion};

pub fn get_rook_vertical_moves(board: Board, color: Color) -> Vec<(Move, u8)> {
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
        let mut rook_pos = rook;
        let mut rook = BitBoard::from(rook);

        // North
        let file_mask: BitBoard = FILES[rook_pos.file_x as usize];
        let blockers = all_pieces & file_mask;
        let difference = BitBoard(blockers.0.wrapping_sub(BitBoard(rook.0.wrapping_mul(2)).0));
        let changed = difference ^ all_pieces;
        let attacks = changed & file_mask & !own_pieces;

        let mut attacks = attacks.into_iter();
        while attacks.popcnt() > 0 {
            let attack = attacks.next().unwrap();
            if attacks.popcnt() == 0 {
                let val = board.get_piece_value(attack);
                moves.push((Move(rook_pos, attack, Promotion::None), val));
            } else {
                moves.push((Move(rook_pos, attack, Promotion::None), 0));
            }
        }

        // South
        rook_pos = rook_pos.flip();
        rook = rook.reverse_colors();
        all_pieces = all_pieces.reverse_colors();
        own_pieces = own_pieces.reverse_colors();

        let file_mask: BitBoard = FILES[rook_pos.file_x as usize];
        let blockers = all_pieces & file_mask;
        let difference = BitBoard(blockers.0.wrapping_sub(BitBoard(rook.0.wrapping_mul(2)).0));
        let changed = difference ^ all_pieces;
        let attacks = changed & file_mask & !own_pieces;

        let mut attacks = attacks.into_iter();
        let real_rook_pos = rook_pos.flip();
        while attacks.popcnt() > 0 {
            let attack = attacks.next().unwrap().flip();
            if attacks.popcnt() == 0 {
                let val = board.get_piece_value(attack);
                moves.push((Move(real_rook_pos, attack, Promotion::None), val));
            } else {
                moves.push((Move(real_rook_pos, attack, Promotion::None), 0));
            }
        }

        // Reset flipping
        all_pieces = all_pieces.reverse_colors();
        own_pieces = own_pieces.reverse_colors();
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

#[cfg(test)]
mod tests {
    use crate::game::pieces::Color;
    use crate::game::Board;

    #[test]
    fn test_get_rook_north_moves() {
        let board = Board::from_fen("2b1kbnr/rppppppp/n7/Pp5P/2P4q/R7/2PPPPP1/1NBQKBNR w Kk - 0 1")
            .unwrap();
        panic!("{:?}", super::get_rook_vertical_moves(board, Color::White));
    }
}
