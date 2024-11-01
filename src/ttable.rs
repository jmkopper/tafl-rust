use crate::board::{Board, BOARD_SIZE};
const TTABLE_SIZE: usize = (1 << 16) - 1;
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
    pub depth: usize,
    pub key: usize,
}

pub struct TranspositionTable {
    pub table: [TranspositionTableEntry; TTABLE_SIZE],
    pub init_hash: [[usize; 3]; NUM_SQUARES],
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
        let hash = zobrist_hash(b, self.attacker_bits, self.init_hash);
        let index = hash % self.capacity;
        return self.table[index].key == hash;
    }

    pub fn retrieve(&self, b: &Board) -> Option<&TranspositionTableEntry> {
        let key = zobrist_hash(b, self.attacker_bits, self.init_hash);
        let entry = &self.table[key % self.capacity];
        if entry.key == key {
            return Some(entry);
        } else {
            return None;
        }
    }

    pub fn store(&mut self, b: &Board, evaluation: i16, depth: usize) {
        let key = zobrist_hash(b, self.attacker_bits, self.init_hash);
        self.table[key % self.capacity] = TranspositionTableEntry {
            evaluation,
            depth,
            key,
        };
    }
}

fn zobrist_hash(b: &Board, attacker_bits: usize, init_board: [[usize; 3]; NUM_SQUARES]) -> usize {
    let mut hash = 0;
    if b.attacker_move {
        hash ^= attacker_bits;
    }

    for index in 0..NUM_SQUARES {
        if b.attacker_board & (1 << index) != 0 {
            hash ^= init_board[index][0];
        }
        if b.defender_board & (1 << index) != 0 {
            hash ^= init_board[index][1];
        }
        if b.king_board & (1 << index) != 0 {
            hash ^= init_board[index][2];
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

    return init_board;
}
