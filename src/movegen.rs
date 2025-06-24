use crate::board::{
    inbounds, index_to_rc, rc_to_index, Bitboard, Board, Move, PieceType, BOARD_SIZE, DIRS,
};

pub struct MoveGenerator {
    pub cached_moves: Vec<Move>,
}

impl MoveGenerator {
    pub fn new(board: &Board) -> Self {
        let mut all_moves = Vec::new();
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            if board.attacker_move {
                if board.attacker_board & (1u64 << i) != 0 {
                    all_moves.extend(gen_attacker_piece_moves(board, i));
                }
            } else {
                // Defender's turn
                if board.defender_board & (1u64 << i) != 0 {
                    all_moves.extend(gen_defender_piece_moves(board, i));
                }
                if board.king_board & (1u64 << i) != 0 {
                    all_moves.extend(gen_king_piece_moves(board, i));
                }
            }
        }
        MoveGenerator {
            cached_moves: all_moves,
        }
    }
}

impl Iterator for MoveGenerator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.cached_moves.pop()
    }
}

#[derive(Debug)]
struct RawMove {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
}

fn gen_simple_adjacent_moves(
    index: usize,
    occupied_squares: Bitboard,
    offlimits_squares: Bitboard,
) -> Vec<RawMove> {
    let mut moves = Vec::new();
    let (row, col) = index_to_rc(index);

    for &(dr, dc) in DIRS.iter() {
        let new_row = row as isize + dr;
        let new_col = col as isize + dc;

        if !inbounds(new_row, new_col) {
            continue;
        }

        let new_index = rc_to_index(new_row as usize, new_col as usize);

        // A move is valid if the target square is not occupied and not off-limits for movement
        if (occupied_squares & (1u64 << new_index)) == 0
            && (offlimits_squares & (1u64 << new_index)) == 0
        {
            moves.push(RawMove {
                from_row: row,
                from_col: col,
                to_row: new_row as usize,
                to_col: new_col as usize,
            });
        }
    }
    moves
}

fn gen_attacker_piece_moves(board: &Board, index: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let occupied =
        board.attacker_board | board.defender_board | board.king_board | board.offlimits_board;
    let piece_raw_moves = gen_simple_adjacent_moves(index, occupied, board.offlimits_board);

    for m in piece_raw_moves {
        moves.push(Move {
            start_index: rc_to_index(m.from_row, m.from_col),
            end_index: rc_to_index(m.to_row, m.to_col),
            piece_type: PieceType::Attacker,
        });
    }
    moves
}

fn gen_defender_piece_moves(board: &Board, index: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let occupied =
        board.attacker_board | board.defender_board | board.king_board | board.offlimits_board;
    let piece_raw_moves = gen_simple_adjacent_moves(index, occupied, board.offlimits_board);

    for m in piece_raw_moves {
        moves.push(Move {
            start_index: rc_to_index(m.from_row, m.from_col),
            end_index: rc_to_index(m.to_row, m.to_col),
            piece_type: PieceType::Defender,
        });
    }
    moves
}

fn gen_king_piece_moves(board: &Board, index: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let occupied = board.attacker_board | board.defender_board | board.king_board;
    let piece_raw_moves = gen_simple_adjacent_moves(index, occupied, 0);

    for m in piece_raw_moves {
        moves.push(Move {
            start_index: rc_to_index(m.from_row, m.from_col),
            end_index: rc_to_index(m.to_row, m.to_col),
            piece_type: PieceType::King,
        });
    }
    moves
}
