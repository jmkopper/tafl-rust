use crate::board::{inbounds, index_to_rc, rc_to_index, Bitboard, Board, Move, BOARD_SIZE, DIRS};

pub struct MoveGenerator<'a> {
    pub board: &'a Board,
    pub index: u64,
    pub cached: Vec<Move>,
}

impl<'a> MoveGenerator<'a> {
    pub fn new(board: &'a Board) -> Self {
        MoveGenerator {
            board,
            index: 0,
            cached: Vec::new(),
        }
    }
}

impl<'a> Iterator for MoveGenerator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cached.len() > 0 {
            return Some(self.cached.pop().unwrap());
        }

        let mut from_pos = Vec::new();

        while from_pos.is_empty() {
            if self.index >= BOARD_SIZE * BOARD_SIZE {
                return None;
            }
            if self.board.attacker_move {
                from_pos = gen_attacker_moves(&self.board, self.index);
            } else {
                from_pos = gen_defender_moves(&self.board, self.index);
            }
            self.index += 1
        }
        self.cached = from_pos;
        return Some(self.cached.pop().unwrap());
    }
}

struct PiecelessMove {
    from: (u64, u64),
    to: (u64, u64),
}

fn gen_moves_one_piece(index: u64, occupied: Bitboard) -> Vec<PiecelessMove> {
    let mut moves = Vec::new();
    let (row, col) = index_to_rc(index);

    for &(dr, dc) in DIRS.iter() {
        let new_row = row as isize + dr;
        let new_col = col as isize + dc;
        if inbounds(new_row, new_col) {
            let new_index = rc_to_index(new_row as u64, new_col as u64);
            if occupied & (1 << new_index) == 0 {
                moves.push(PiecelessMove {
                    from: (row, col),
                    to: (new_row as u64, new_col as u64),
                });
            }
        }
    }

    return moves;
}

fn gen_attacker_moves(board: &Board, index: u64) -> Vec<Move> {
    let mut moves = Vec::new();

    if board.attacker_board & (1 << index) != 0 {
        let occupied =
            board.attacker_board | board.defender_board | board.king_board | board.offlimits_board;
        let piece_moves = gen_moves_one_piece(index, occupied);
        for m in piece_moves {
            moves.push(Move {
                start_row: m.from.0,
                start_col: m.from.1,
                end_row: m.to.0,
                end_col: m.to.1,
                king_move: false,
            });
        }
    }

    return moves;
}

fn gen_defender_moves(board: &Board, index: u64) -> Vec<Move> {
    let mut moves = Vec::new();

    // non-king moves
    if board.defender_board & (1 << index) != 0 {
        let occupied =
            board.attacker_board | board.defender_board | board.king_board | board.offlimits_board;
        let piece_moves = gen_moves_one_piece(index, occupied);
        for m in piece_moves {
            moves.push(Move {
                start_row: m.from.0,
                start_col: m.from.1,
                end_row: m.to.0,
                end_col: m.to.1,
                king_move: false,
            });
        }
        return moves;
    }

    // king moves
    if board.king_board & (1 << index) != 0 {
        let occupied = board.attacker_board | board.defender_board | board.king_board;
        let piece_moves = gen_moves_one_piece(index, occupied);
        for m in piece_moves {
            moves.push(Move {
                start_row: m.from.0,
                start_col: m.from.1,
                end_row: m.to.0,
                end_col: m.to.1,
                king_move: true,
            });
        }
    }

    return moves;
}
