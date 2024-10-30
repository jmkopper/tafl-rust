pub const BOARD_SIZE: u64 = 7;
pub const DIRS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
pub type Bitboard = u64;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub start_row: u64,
    pub start_col: u64,
    pub end_row: u64,
    pub end_col: u64,
    pub king_move: bool,
}

pub const NULL_MOVE: Move = Move {
    start_row: 0,
    start_col: 0,
    end_row: 0,
    end_col: 0,
    king_move: false,
};

impl Move {
    pub fn to_string(&self) -> String {
        let mut s: String;
        if self.king_move {
            s = "k".to_string();
        } else {
            s = ((self.start_col as u8 + 'a' as u8) as char).to_string();
            s.push_str(&(self.start_row + 1).to_string());
        }

        s.push((self.end_col as u8 + 'a' as u8) as char);
        s.push_str(&(self.end_row + 1).to_string());

        return s;
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        if self.king_move && other.king_move {
            return self.end_row == other.end_row && self.end_col == other.end_col;
        }
        return self.start_row == other.start_row
            && self.start_col == other.start_col
            && self.end_row == other.end_row
            && self.end_col == other.end_col
            && self.king_move == other.king_move;
    }

    fn ne(&self, other: &Self) -> bool {
        if self.king_move && other.king_move {
            return self.end_row != other.end_row || self.end_col != other.end_col;
        }
        return self.start_row != other.start_row
            || self.start_col != other.start_col
            || self.end_row != other.end_row
            || self.end_col != other.end_col
            || self.king_move != other.king_move;
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
}

pub fn rc_to_index(row: u64, col: u64) -> u64 {
    row * BOARD_SIZE + col
}

pub fn index_to_rc(index: u64) -> (u64, u64) {
    (index / BOARD_SIZE, index % BOARD_SIZE)
}

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

            let new_index = rc_to_index(new_row as u64, new_col as u64);
            if self.offlimits_board & (1 << new_index) != 0 {
                continue;
            }

            if !self.attacker_board & (1 << new_index) != 0 {
                return false;
            }
        }

        return true;
    }

    pub fn king_coordinates(&self) -> (u64, u64) {
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            if self.king_board & (1 << i) != 0 {
                return index_to_rc(i);
            }
        }

        return (0, 0);
    }

    pub fn make_attacker_move(&self, m: Move) -> Board {
        let mut new_board = Board {
            attacker_board: self.attacker_board,
            defender_board: self.defender_board,
            king_board: self.king_board,
            offlimits_board: self.offlimits_board,
            attacker_move: false,
            attacker_win: false,
            defender_win: false,
            stalemate: false,
        };
        let index = rc_to_index(m.start_row, m.start_col);
        let new_index = rc_to_index(m.end_row, m.end_col);
        new_board.attacker_board &= !(1 << index);
        new_board.attacker_board |= 1 << new_index;

        if new_board.king_captured() {
            new_board.attacker_win = true;
        }

        for dir in DIRS.iter() {
            let new_row = m.end_row as isize + dir.0;
            let new_col = m.end_col as isize + dir.1;
            if valid_capture(
                &self.attacker_board,
                &self.defender_board,
                (m.end_row as isize, m.end_col as isize),
                (new_row, new_col),
            ) {
                let defender_index = rc_to_index(new_row as u64, new_col as u64);
                new_board.defender_board &= !(1 << defender_index);
            }
        }

        return new_board;
    }

    fn make_defender_move(&self, m: Move) -> Board {
        let mut new_board = Board {
            attacker_board: self.attacker_board,
            defender_board: self.defender_board,
            king_board: self.king_board,
            offlimits_board: self.offlimits_board,
            attacker_move: true,
            attacker_win: false,
            defender_win: false,
            stalemate: false,
        };
        let index = rc_to_index(m.start_row, m.start_col);
        let new_index = rc_to_index(m.end_row, m.end_col);
        new_board.defender_board &= !(1 << index);
        new_board.defender_board |= 1 << new_index;

        let capturer_bitboard = &self.defender_board | &self.king_board;

        for dir in DIRS.iter() {
            let new_row = m.end_row as isize + dir.0;
            let new_col = m.end_col as isize + dir.1;
            if valid_capture(
                &capturer_bitboard,
                &self.attacker_board,
                (m.end_row as isize, m.end_col as isize),
                (new_row, new_col),
            ) {
                let attacker_index = rc_to_index(new_row as u64, new_col as u64);
                new_board.attacker_board &= !(1 << attacker_index);
            }
        }

        return new_board;
    }

    fn make_king_move(&self, m: Move) -> Board {
        let mut new_board = Board {
            attacker_board: self.attacker_board,
            defender_board: self.defender_board,
            king_board: self.king_board,
            offlimits_board: self.offlimits_board,
            attacker_move: true,
            attacker_win: false,
            defender_win: false,
            stalemate: false,
        };

        let (king_row, king_col) = self.king_coordinates();
        let index = rc_to_index(king_row, king_col);
        let new_index = rc_to_index(m.end_row, m.end_col);

        new_board.king_board &= !(1 << index);
        new_board.king_board |= 1 << new_index;

        if (m.end_col == 0 && m.end_row == 0)
            || (m.end_col == BOARD_SIZE - 1 && m.end_row == 0)
            || (m.end_col == BOARD_SIZE - 1 && m.end_row == BOARD_SIZE - 1)
            || (m.end_col == 0 && m.end_row == BOARD_SIZE - 1)
        {
            new_board.defender_win = true;
        }

        let capturer_bitboard = &self.defender_board | &self.king_board;

        for dir in DIRS.iter() {
            let new_row = m.end_row as isize + dir.0;
            let new_col = m.end_col as isize + dir.1;
            if valid_capture(
                &capturer_bitboard,
                &self.attacker_board,
                (m.end_row as isize, m.end_col as isize),
                (new_row, new_col),
            ) {
                let attacker_index = rc_to_index(new_row as u64, new_col as u64);
                new_board.attacker_board &= !(1 << attacker_index);
            }
        }

        return new_board;
    }

    pub fn make_move(&self, m: Move) -> Board {
        let new_b: Board;
        if m.king_move {
            new_b = self.make_king_move(m);
        } else if self.attacker_move {
            new_b = self.make_attacker_move(m);
        } else {
            new_b = self.make_defender_move(m);
        }
        // if new_b.legal_moves().len() == 0 {
        //     new_b.stalemate = true;
        // }
        return new_b;
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for i in (0..BOARD_SIZE).rev() {
            s.push((i + 1 + ('0' as u64)) as u8 as char);
            s.push(' ');
            for j in 0..BOARD_SIZE {
                let index = rc_to_index(i as u64, j as u64);
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
            s.push((j + ('a' as u64)) as u8 as char);
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

    let capturee_index = rc_to_index(capturee_coords.0 as u64, capturee_coords.1 as u64);
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

    let ally_index = rc_to_index(ally_coords.0 as u64, ally_coords.1 as u64);

    return capturer_bitboard & (1 << ally_index) != 0;
}

pub fn bool_array_to_bitboard(arr: [[bool; BOARD_SIZE as usize]; BOARD_SIZE as usize]) -> Bitboard {
    let mut b: Bitboard = 0;
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            if arr[i as usize][j as usize] {
                b |= 1 << rc_to_index(i, j);
            }
        }
    }
    return b;
}
