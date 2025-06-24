pub const BOARD_SIZE: usize = 7;
pub const DIRS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
pub type Bitboard = u64;
use crate::ttable::{
    TranspositionTable, PIECE_TYPE_ATTACKER_IDX, PIECE_TYPE_DEFENDER_IDX, PIECE_TYPE_KING_IDX,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Attacker,
    Defender,
    King,
}

#[derive(Clone, Copy)]
pub struct Move {
    pub start_index: usize,
    pub end_index: usize,
    pub piece_type: PieceType,
}

#[derive(Clone, Copy)]
pub struct MoveHistoryElement {
    prev_move: Move,
    captured_piece_index: Option<usize>,
}

pub const NULL_MOVE: Move = Move {
    start_index: 0,
    end_index: 0,
    piece_type: PieceType::Attacker,
};

impl Move {
    pub fn to_string(&self) -> String {
        let mut s: String;
        let (start_row, start_col) = index_to_rc(self.start_index);
        let (end_row, end_col) = index_to_rc(self.end_index);
        if self.piece_type == PieceType::King {
            s = "k".to_string();
        } else {
            s = ((start_col as u8 + 'a' as u8) as char).to_string();
            s.push_str(&(start_row + 1).to_string());
        }

        s.push((end_col as u8 + 'a' as u8) as char);
        s.push_str(&(end_row + 1).to_string());

        return s;
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
    pub history: Vec<MoveHistoryElement>,
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

pub fn rc_to_index(row: usize, col: usize) -> usize {
    row * BOARD_SIZE + col
}

pub fn index_to_rc(index: usize) -> (usize, usize) {
    (index / BOARD_SIZE, index % BOARD_SIZE)
}

pub fn inbounds(row: isize, col: isize) -> bool {
    return row >= 0 && row < BOARD_SIZE as isize && col >= 0 && col < BOARD_SIZE as isize;
}

impl Clone for Board {
    fn clone(&self) -> Self {
        return Board {
            attacker_board: self.attacker_board,
            defender_board: self.defender_board,
            king_board: self.king_board,
            offlimits_board: self.offlimits_board,
            attacker_move: self.attacker_move,
            attacker_win: self.attacker_win,
            stalemate: self.stalemate,
            defender_win: self.defender_win,
            current_hash: self.current_hash,
            history: self.history.clone(),
        };
    }
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

            if !self.attacker_board & (1 << new_index) != 0 {
                return false;
            }
        }

        return true;
    }

    pub fn king_coordinates(&self) -> (usize, usize) {
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            if self.king_board & (1 << i) != 0 {
                return index_to_rc(i);
            }
        }

        return (0, 0);
    }

    pub fn get_piece_type_at_index(&self, index: usize) -> Option<PieceType> {
        if self.attacker_board & (1u64 << index) != 0 {
            Some(PieceType::Attacker)
        } else if self.defender_board & (1u64 << index) != 0 {
            Some(PieceType::Defender)
        } else if self.king_board & (1u64 << index) != 0 {
            Some(PieceType::King)
        } else {
            None
        }
    }

    pub fn make_attacker_move(&mut self, m: Move, tt: &TranspositionTable) {
        let mut hist_move = MoveHistoryElement {
            prev_move: m,
            captured_piece_index: None,
        };
        self.attacker_board &= !(1 << m.start_index);
        self.attacker_board |= 1 << m.end_index;

        let (king_row, king_col) = self.king_coordinates();
        let mut next_to_king = false;

        let (end_row, end_col) = index_to_rc(m.end_index);
        for dir in DIRS.iter() {
            let new_row = end_row as isize + dir.0;
            let new_col = end_col as isize + dir.1;
            if inbounds(new_row, new_col)
                && new_row == king_row as isize
                && new_col == king_col as isize
            {
                next_to_king = true;
                break;
            }
        }

        if self.king_captured() && next_to_king {
            self.attacker_win = true;
        }

        // check for captures
        for dir in DIRS.iter() {
            let new_row = end_row as isize + dir.0;
            let new_col = end_col as isize + dir.1;
            if valid_capture(
                &self.attacker_board,
                &self.defender_board,
                (end_row as isize, end_col as isize),
                (new_row, new_col),
            ) {
                let captured_index = rc_to_index(new_row as usize, new_col as usize);
                self.defender_board &= !(1 << captured_index);
                self.current_hash ^= tt.init_hash[captured_index as usize][PIECE_TYPE_DEFENDER_IDX];
                hist_move.captured_piece_index = Some(captured_index);
            }
        }

        // update hash
        self.current_hash ^= tt.init_hash[m.start_index as usize][PIECE_TYPE_ATTACKER_IDX];
        self.current_hash ^= tt.init_hash[m.end_index as usize][PIECE_TYPE_ATTACKER_IDX];
        self.history.push(hist_move);
    }

    fn make_defender_move(&mut self, m: Move, tt: &TranspositionTable) {
        let mut hist_move = MoveHistoryElement {
            prev_move: m,
            captured_piece_index: None,
        };
        self.defender_board &= !(1 << m.start_index);
        self.defender_board |= 1 << m.end_index;

        let capturer_bitboard = &self.defender_board | &self.king_board;

        let (end_row, end_col) = index_to_rc(m.start_index);

        for dir in DIRS.iter() {
            let new_row = end_row as isize + dir.0;
            let new_col = end_col as isize + dir.1;
            if valid_capture(
                &capturer_bitboard,
                &self.attacker_board,
                (end_row as isize, end_col as isize),
                (new_row, new_col),
            ) {
                let captured_index = rc_to_index(new_row as usize, new_col as usize);
                self.attacker_board &= !(1 << captured_index);
                self.current_hash ^= tt.init_hash[captured_index as usize][PIECE_TYPE_ATTACKER_IDX];
                hist_move.captured_piece_index = Some(captured_index);
            }
        }

        // update hash
        self.current_hash ^= tt.attacker_bits;
        self.current_hash ^= tt.init_hash[m.start_index as usize][PIECE_TYPE_DEFENDER_IDX];
        self.current_hash ^= tt.init_hash[m.end_index as usize][PIECE_TYPE_DEFENDER_IDX];
        self.history.push(hist_move);
    }

    fn make_king_move(&mut self, m: Move, tt: &TranspositionTable) {
        let mut hist_move = MoveHistoryElement {
            prev_move: m,
            captured_piece_index: None,
        };
        self.king_board &= !(1 << m.start_index);
        self.king_board |= 1 << m.end_index;

        let (end_row, end_col) = index_to_rc(m.end_index);

        if (end_col == 0 && end_row == 0)
            || (end_col == BOARD_SIZE - 1 && end_row == 0)
            || (end_col == BOARD_SIZE - 1 && end_row == BOARD_SIZE - 1)
            || (end_col == 0 && end_row == BOARD_SIZE - 1)
        {
            self.defender_win = true;
        }

        let capturer_bitboard = &self.defender_board | &self.king_board;

        for dir in DIRS.iter() {
            let new_row = end_row as isize + dir.0;
            let new_col = end_col as isize + dir.1;
            if valid_capture(
                &capturer_bitboard,
                &self.attacker_board,
                (end_row as isize, end_col as isize),
                (new_row, new_col),
            ) {
                let captured_index = rc_to_index(new_row as usize, new_col as usize);
                self.attacker_board &= !(1 << captured_index);
                self.current_hash ^= tt.init_hash[captured_index as usize][PIECE_TYPE_ATTACKER_IDX];
                hist_move.captured_piece_index = Some(captured_index);
            }
        }

        self.current_hash ^= tt.attacker_bits;
        self.current_hash ^= tt.init_hash[m.start_index as usize][PIECE_TYPE_KING_IDX];
        self.current_hash ^= tt.init_hash[m.end_index as usize][PIECE_TYPE_KING_IDX];
        self.history.push(hist_move);
    }

    pub fn make_move(&mut self, m: Move, tt: &TranspositionTable) {
        match m.piece_type {
            PieceType::Attacker => self.make_attacker_move(m, tt),
            PieceType::Defender => self.make_defender_move(m, tt),
            PieceType::King => self.make_king_move(m, tt),
        }
        self.attacker_move = !self.attacker_move;
    }

    pub fn unmake_move(&mut self, tt: &TranspositionTable) {
        let m = self
            .history
            .pop()
            .expect("tried to unmake move with empty history");
        let prev_move = m.prev_move;
        self.attacker_move = !self.attacker_move;
        self.current_hash ^= tt.attacker_bits;

        match prev_move.piece_type {
            PieceType::Attacker => {
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_ATTACKER_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_ATTACKER_IDX];
                self.attacker_board |= 1 << prev_move.start_index; // move the piece back
                self.attacker_board ^= 1 << prev_move.end_index; // remove
            }
            PieceType::Defender => {
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_DEFENDER_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_DEFENDER_IDX];
                self.defender_board |= 1 << prev_move.start_index;
                self.defender_board ^= 1 << prev_move.end_index;
            }
            PieceType::King => {
                self.current_hash ^= tt.init_hash[prev_move.end_index][PIECE_TYPE_KING_IDX];
                self.current_hash ^= tt.init_hash[prev_move.start_index][PIECE_TYPE_KING_IDX];
                self.king_board |= 1 << prev_move.start_index;
                self.king_board ^= 1 << prev_move.end_index;
            }
        }

        if let Some(captured_idx) = m.captured_piece_index {
            match prev_move.piece_type {
                PieceType::Attacker => {
                    self.defender_board |= 1 << captured_idx;
                    self.current_hash ^= tt.init_hash[captured_idx][PIECE_TYPE_DEFENDER_IDX]
                }
                PieceType::Defender | PieceType::King => {
                    self.attacker_board |= 1 << captured_idx;
                    self.current_hash ^= tt.init_hash[captured_idx][PIECE_TYPE_ATTACKER_IDX]
                }
            }
        }

        self.attacker_win = false;
        self.defender_win = false;
        self.stalemate = false;
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

fn valid_capture(
    capturer_bitboard: &Bitboard,
    capturee_bitboard: &Bitboard,
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
        2 * (capturee_coords.0 - capturer_coords.0) + capturer_coords.0,
        2 * (capturee_coords.1 - capturer_coords.1) + capturer_coords.1,
    );

    if !inbounds(ally_coords.0, ally_coords.1) {
        return false;
    }

    let ally_index = rc_to_index(ally_coords.0 as usize, ally_coords.1 as usize);

    return capturer_bitboard & (1 << ally_index) != 0;
}
