use crate::board::{
    inbounds, index_to_rc, rc_to_index, valid_capture, Board, Move, PieceType, BOARD_SIZE, DIRS,
    EMPTY_BOARD,
};

const KING_ESCAPE_SCORE: i16 = 5000;
const MOVE_TO_KING_SCORE: i16 = 1000;
const CAPTURE_SCORE: i16 = 1000;
const NORMAL_MOVE_SCORE: i16 = 0;

#[derive(Debug)]
struct ScoredMove {
    mv: Move,
    score: i16,
}

impl Eq for ScoredMove {}

impl PartialEq for ScoredMove {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

pub struct MoveGenerator {
    pub cached_moves: Vec<Move>,
}

impl MoveGenerator {
    pub fn new(board: &Board) -> Self {
        let mut all_moves = Vec::with_capacity(32);
        let occupied = board.attacker_board | board.defender_board | board.king_board;

        if board.attacker_move {
            let mut current_attackers = board.attacker_board;
            while current_attackers != 0 {
                let start_index = current_attackers.trailing_zeros() as usize;
                gen_piece_moves(
                    board,
                    start_index,
                    occupied,
                    board.offlimits_board,
                    PieceType::Attacker,
                    &mut all_moves,
                );
                current_attackers &= !(1 << start_index);
            }
        } else {
            let mut current_defenders = board.defender_board;
            while current_defenders != 0 {
                let start_index = current_defenders.trailing_zeros() as usize;
                gen_piece_moves(
                    board,
                    start_index,
                    occupied,
                    board.offlimits_board,
                    PieceType::Defender,
                    &mut all_moves,
                );
                current_defenders &= !(1 << start_index);
            }
            gen_piece_moves(
                board,
                board.king_index(),
                occupied,
                EMPTY_BOARD,
                PieceType::King,
                &mut all_moves,
            );
        }

        all_moves.sort_unstable();
        Self {
            cached_moves: all_moves.into_iter().map(|sm| sm.mv).collect(),
        }
    }
}

impl Iterator for MoveGenerator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.cached_moves.pop()
    }
}

fn gen_piece_moves(
    board: &Board,
    start_index: usize,
    occupied: u64,
    offlimits: u64,
    piece_type: PieceType,
    moves: &mut Vec<ScoredMove>,
) {
    let (start_row, start_col) = index_to_rc(start_index);
    for &(dr, dc) in DIRS.iter() {
        let end_row = start_row as isize + dr;
        let end_col = start_col as isize + dc;

        if !inbounds(end_row, end_col) {
            continue;
        }
        let end_index = rc_to_index(end_row as usize, end_col as usize);
        // A move is valid if the target square is not occupied and not off-limits for movement
        if (occupied | offlimits) & (1u64 << end_index) == 0 {
            let mv = Move {
                start_index,
                end_index,
                piece_type,
            };
            let sm = ScoredMove {
                mv,
                score: score_move(board, &mv),
            };
            moves.push(sm);
        }
    }
}

fn score_move(board: &Board, m: &Move) -> i16 {
    let mut score = NORMAL_MOVE_SCORE;
    let (end_row, end_col) = index_to_rc(board.king_index());

    if m.piece_type == PieceType::King {
        if (end_row == 0 || end_row == BOARD_SIZE - 1)
            && (end_col == 0 || end_col == BOARD_SIZE - 1)
        {
            return KING_ESCAPE_SCORE;
        }
    }

    if m.piece_type == PieceType::Attacker {
        let (king_row, king_col) = board.king_coordinates();
        let (end_row, end_col) = index_to_rc(m.end_index);
        if king_row.abs_diff(end_row) + king_col.abs_diff(end_col) < 2 {
            score += MOVE_TO_KING_SCORE;
        }
    }

    let capturer_board: u64;
    let capturee_board: u64;
    match m.piece_type {
        PieceType::Attacker => {
            capturer_board = board.attacker_board;
            capturee_board = board.defender_board;
        }
        _ => {
            capturer_board = board.defender_board | board.king_board;
            capturee_board = board.attacker_board;
        }
    }

    for dir in DIRS {
        let capturee_row = end_row as isize + dir.0;
        let capturee_col = end_col as isize + dir.1;
        if valid_capture(
            capturer_board,
            capturee_board,
            (end_row as isize, end_col as isize),
            (capturee_row, capturee_col),
        ) {
            score += CAPTURE_SCORE;
        }
    }

    score
}
