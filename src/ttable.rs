use crate::board::{Board, BOARD_SIZE};
const TTABLE_SIZE: usize = 1 << 16;
const NUM_SQUARES: usize = (BOARD_SIZE * BOARD_SIZE) as usize;

#[derive(PartialEq, Clone, Copy)]
pub enum Flag {
    EXACT,
    LOWERBOUND,
    UPPERBOUND,
}

#[derive(Clone, Copy, PartialEq)]
pub struct TranspositionTableEntry {
    pub evaluation: i16,
    pub depth: u8,
    pub key: usize,
}

pub const PIECE_TYPE_ATTACKER_IDX: usize = 0;
pub const PIECE_TYPE_DEFENDER_IDX: usize = 1;
pub const PIECE_TYPE_KING_IDX: usize = 2;

pub struct TranspositionTable {
    pub table: [TranspositionTableEntry; TTABLE_SIZE],
    pub init_hash: [[usize; 3]; NUM_SQUARES], // attacker: 0, defender: 1, king: 2
    pub attacker_bits: usize,
    pub capacity: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let table = [TranspositionTableEntry {
            evaluation: 0,
            depth: 0,
            key: 0,
        }; TTABLE_SIZE];
        return TranspositionTable {
            table,
            init_hash: make_init_hash(),
            attacker_bits: rand::random(),
            capacity: TTABLE_SIZE,
        };
    }

    pub fn haskey(&self, b: &Board) -> bool {
        let hash = b.current_hash;
        let index = hash & (self.capacity - 1);
        self.table[index].key == hash
    }

    pub fn retrieve(&self, b: &Board) -> Option<&TranspositionTableEntry> {
        // let key = b.current_hash;
        let key = self.hash_from_board(b);
        let entry = &self.table[key & (self.capacity - 1)];
        if entry.key == key {
            return Some(entry);
        } else {
            return None;
        }
    }

    pub fn store(&mut self, b: &Board, evaluation: i16, depth: u8) {
        // let key = b.current_hash;
        let key = self.hash_from_board(b);
        self.table[key & (self.capacity - 1)] = TranspositionTableEntry {
            evaluation,
            depth,
            key,
        };
    }

    pub fn hash_from_board(&self, b: &Board) -> usize {
        zobrist_hash(b, self.attacker_bits, self.init_hash)
    }
}

fn zobrist_hash(b: &Board, attacker_bits: usize, init_board: [[usize; 3]; NUM_SQUARES]) -> usize {
    let mut hash = 0;
    if b.attacker_move {
        hash ^= attacker_bits;
    }

    for index in 0..NUM_SQUARES {
        if b.attacker_board & (1 << index) != 0 {
            hash ^= init_board[index][PIECE_TYPE_ATTACKER_IDX];
        }
        if b.defender_board & (1 << index) != 0 {
            hash ^= init_board[index][PIECE_TYPE_DEFENDER_IDX];
        }
        if b.king_board & (1 << index) != 0 {
            hash ^= init_board[index][PIECE_TYPE_KING_IDX];
        }
    }

    return hash;
}

fn make_init_hash() -> [[usize; 3]; NUM_SQUARES] {
    let mut init_board = [[0; 3]; NUM_SQUARES];
    for i in 0..NUM_SQUARES {
        for j in 0..3 {
            init_board[i][j] = rand::random();
        }
    }

    init_board
}
