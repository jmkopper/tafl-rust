pub const BOARD_SIZE: usize = 7;
pub const DIRS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
pub type Bitboard = u64;
pub const EMPTY_BOARD: Bitboard = 0;

use crate::ttable::{
    TranspositionTable, PIECE_TYPE_ATTACKER_IDX, PIECE_TYPE_DEFENDER_IDX, PIECE_TYPE_KING_IDX,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    Attacker,
    Defender,
    King,
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub start_index: usize,
    pub end_index: usize,
    pub piece_type: PieceType,
}

#[derive(Clone, Debug)]
struct MoveHistoryElement {
    prev_move: Move,
    captured_piece_indices: [usize; 4],
    num_captured_pieces: usize,
}

pub const NULL_MOVE: Move = Move {
    start_index: 0,
    end_index: 0,
    piece_type: PieceType::Attacker, // arbitrary
};

impl Move {
    pub fn to_string(&self) -> String {
        let (start_row, start_col) = index_to_rc(self.start_index);
        let (end_row, end_col) = index_to_rc(self.end_index);
        let mut s = String::with_capacity(5); // "k" + "a1" + "a1" = 5 chars max
        if self.piece_type == PieceType::King {
            s.push('k');
        } else {
            s.push((start_col as u8 + b'a') as char);
            s.push_str(&(start_row + 1).to_string());
        }
        s.push((end_col as u8 + b'a') as char);
        s.push_str(&(end_row + 1).to_string());
        s
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index
            && self.end_index == other.end_index
            && self.piece_type == other.piece_type
    }

    fn ne(&self, other: &Self) -> bool {
        self.start_index != other.start_index
            || self.end_index != other.end_index
            || self.piece_type != other.piece_type
    }
}

#[derive(Clone)]
pub struct Board {
    pub attacker_board: Bitboard,
    pub defender_board: Bitboard,
    pub king_board: Bitboard,
    pub offlimits_board: Bitboard,
    pub attacker_move: bool,
    pub attacker_win: bool,
    pub defender_win: bool,
    pub stalemate: bool,
    pub current_hash: usize,
    history: Vec<MoveHistoryElement>,
}

pub const STARTING_BOARD: Board = Board {
    attacker_board: 123437837206556,
    defender_board: 7558594560,
    king_board: 16777216,
    offlimits_board: 285873039999041,
    attacker_move: false,
    attacker_win: false,
    defender_win: false,
    stalemate: false,
    current_hash: 0,
    history: Vec::new(),
};

#[inline]
pub const fn rc_to_index(row: usize, col: usize) -> usize {
    row * BOARD_SIZE + col
}

#[inline]
pub const fn index_to_rc(index: usize) -> (usize, usize) {
    (index / BOARD_SIZE, index % BOARD_SIZE)
}

#[inline]
pub fn inbounds(row: isize, col: isize) -> bool {
    return row >= 0 && row < BOARD_SIZE as isize && col >= 0 && col < BOARD_SIZE as isize;
}

impl Board {
    pub fn king_captured(&self) -> bool {
        let (king_row, king_col) = self.king_coordinates();

        for dir in DIRS.iter() {
            let new_row = king_row as isize + dir.0;
            let new_col = king_col as isize + dir.1;

            if !inbounds(new_row, new_col) {
                continue;
            }

            let new_index = rc_to_index(new_row as usize, new_col as usize);
            if self.offlimits_board & (1 << new_index) != 0 {
                continue;
            }

            if self.attacker_board & (1 << new_index) == 0 {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn king_coordinates(&self) -> (usize, usize) {
        return index_to_rc(self.king_board.trailing_zeros() as usize);
    }

    #[inline]
    pub fn king_index(&self) -> usize {
        return self.king_board.trailing_zeros() as usize;
    }

    pub fn make_move(&mut self, m: Move, tt: &TranspositionTable) {
        let mut hist_move = MoveHistoryElement {
            prev_move: m,
            captured_piece_indices: [0; 4],
            num_captured_pieces: 0,
        };

        let piece_mask = 1 << m.start_index | 1 << m.end_index;
        let capturer_board: u64;
        let capturee_board: u64;
        match m.piece_type {
            PieceType::Attacker => {
                self.attacker_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[m.end_index][PIECE_TYPE_ATTACKER_IDX];
                self.current_hash ^= tt.init_hash[m.start_index][PIECE_TYPE_ATTACKER_IDX];
                capturer_board = self.attacker_board;
                capturee_board = self.defender_board;
            }
            PieceType::Defender => {
                self.defender_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[m.end_index][PIECE_TYPE_DEFENDER_IDX];
                self.current_hash ^= tt.init_hash[m.start_index][PIECE_TYPE_DEFENDER_IDX];
                capturer_board = self.defender_board | self.king_board;
                capturee_board = self.attacker_board;
            }
            PieceType::King => {
                self.king_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[m.end_index][PIECE_TYPE_KING_IDX];
                self.current_hash ^= tt.init_hash[m.start_index][PIECE_TYPE_KING_IDX];
                capturer_board = self.defender_board | self.king_board;
                capturee_board = self.attacker_board;
            }
        }

        // check for captures
        let (end_row, end_col) = index_to_rc(m.end_index);
        for dir in DIRS.iter() {
            let capturee_row = end_row as isize + dir.0;
            let capturee_col = end_col as isize + dir.1;
            if valid_capture(
                capturer_board,
                capturee_board,
                (end_row as isize, end_col as isize),
                (capturee_row, capturee_col),
            ) {
                let captured_index = rc_to_index(capturee_row as usize, capturee_col as usize);
                let captured_piece_mask = 1 << captured_index;
                match m.piece_type {
                    PieceType::Attacker => {
                        self.defender_board ^= captured_piece_mask;
                        self.current_hash ^= tt.init_hash[captured_index][PIECE_TYPE_DEFENDER_IDX]
                    }
                    _ => {
                        self.attacker_board ^= captured_piece_mask;
                        self.current_hash ^= tt.init_hash[captured_index][PIECE_TYPE_ATTACKER_IDX];
                    }
                }
                hist_move.captured_piece_indices[hist_move.num_captured_pieces] = captured_index;
                hist_move.num_captured_pieces += 1;
            }
        }

        // check for attacker win
        if m.piece_type == PieceType::Attacker && self.king_captured() {
            self.attacker_win = true;
        }

        // check for defender win
        if m.piece_type == PieceType::King {
            self.defender_win = (end_col == 0 || end_col == BOARD_SIZE - 1)
                && (end_row == 0 || end_row == BOARD_SIZE - 1);
        }

        self.history.push(hist_move);
        self.attacker_move = !self.attacker_move;
        self.current_hash ^= tt.attacker_bits_seed; // toggles for attacker's turn
    }

    pub fn unmake_move(&mut self, tt: &TranspositionTable) {
        let m = self
            .history
            .pop()
            .expect("tried to unmake move with empty history");
        let prev_move = m.prev_move;

        let piece_mask = 1 << prev_move.start_index | 1 << prev_move.end_index;
        match prev_move.piece_type {
            PieceType::Attacker => {
                self.attacker_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_ATTACKER_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_ATTACKER_IDX];
            }
            PieceType::Defender => {
                self.defender_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_DEFENDER_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_DEFENDER_IDX];
            }
            PieceType::King => {
                self.king_board ^= piece_mask;
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_KING_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_KING_IDX];
            }
        }

        for i in 0..m.num_captured_pieces {
            let captured_idx = m.captured_piece_indices[i];
            let captured_piece_mask = 1 << captured_idx;
            match prev_move.piece_type {
                PieceType::Attacker => {
                    self.defender_board |= captured_piece_mask;
                    self.current_hash ^= tt.init_hash[captured_idx][PIECE_TYPE_DEFENDER_IDX]
                }
                PieceType::Defender | PieceType::King => {
                    self.attacker_board |= captured_piece_mask;
                    self.current_hash ^= tt.init_hash[captured_idx][PIECE_TYPE_ATTACKER_IDX]
                }
            }
        }

        self.attacker_win = false;
        self.defender_win = false;
        self.stalemate = false;
        self.attacker_move = !self.attacker_move;
        self.current_hash ^= tt.attacker_bits_seed;
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for i in (0..BOARD_SIZE).rev() {
            s.push((i + 1 + ('0' as usize)) as u8 as char);
            s.push(' ');
            for j in 0..BOARD_SIZE {
                let index = rc_to_index(i as usize, j as usize);
                if self.attacker_board & (1 << index) != 0 {
                    s.push('V');
                } else if self.king_board & (1 << index) != 0 {
                    s.push('K');
                } else if self.defender_board & (1 << index) != 0 {
                    s.push('O');
                } else if self.offlimits_board & (1 << index) != 0 {
                    s.push('#');
                } else {
                    s.push('.');
                }
                s.push(' ');
            }
            s.push('\n');
        }

        s.push_str("  ");
        for j in 0..BOARD_SIZE {
            s.push((j + ('a' as usize)) as u8 as char);
            s.push(' ');
        }
        s.push('\n');

        return s;
    }
}

#[inline(always)]
pub fn valid_capture(
    capturer_bitboard: Bitboard,
    capturee_bitboard: Bitboard,
    capturer_coords: (isize, isize),
    capturee_coords: (isize, isize),
) -> bool {
    if !inbounds(capturee_coords.0, capturee_coords.1) {
        return false;
    }
    if !inbounds(capturer_coords.0, capturer_coords.1) {
        return false;
    }

    let capturee_index = rc_to_index(capturee_coords.0 as usize, capturee_coords.1 as usize);
    if capturee_bitboard & (1 << capturee_index) == 0 {
        return false;
    }

    let ally_coords = (
        2 * capturee_coords.0 - capturer_coords.0,
        2 * capturee_coords.1 - capturer_coords.1,
    );

    if !inbounds(ally_coords.0, ally_coords.1) {
        return false;
    }

    let ally_index = rc_to_index(ally_coords.0 as usize, ally_coords.1 as usize);
    capturer_bitboard & (1 << ally_index) != 0
}
