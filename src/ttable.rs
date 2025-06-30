use rand::Rng;

use crate::board::{Bitboard, Board, BOARD_SIZE};
const TTABLE_SIZE: usize = 1 << 20;
const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

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
    pub flag: Flag,
}

pub const PIECE_TYPE_ATTACKER_IDX: usize = 0;
pub const PIECE_TYPE_DEFENDER_IDX: usize = 1;
pub const PIECE_TYPE_KING_IDX: usize = 2;

pub struct TranspositionTable {
    pub table: Vec<TranspositionTableEntry>,
    pub init_hash: [[usize; 3]; NUM_SQUARES], // attacker: 0, defender: 1, king: 2
    pub attacker_bits_seed: usize,
    pub capacity: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let init_entry = TranspositionTableEntry {
            evaluation: 0,
            depth: 0,
            key: 0,
            flag: Flag::EXACT,
        };
        return TranspositionTable {
            table: vec![init_entry; TTABLE_SIZE],
            init_hash: make_init_hash(),
            attacker_bits_seed: rand::thread_rng().gen::<usize>(),
            capacity: TTABLE_SIZE,
        };
    }

    #[inline]
    pub fn haskey(&self, b: &Board) -> bool {
        let hash = b.current_hash;
        let index = hash & (self.capacity - 1);
        self.table[index].key == hash
    }

    #[inline]
    pub fn retrieve(&self, b: &Board) -> Option<&TranspositionTableEntry> {
        let key = b.current_hash;
        let entry = &self.table[key & (self.capacity - 1)];
        if entry.key == key {
            Some(entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn store(&mut self, b: &Board, evaluation: i16, depth: u8, flag: Flag) {
        let key = b.current_hash;
        debug_assert_eq!(b.current_hash, self.hash_from_board(b));
        let index = key & (self.capacity - 1);
        self.table[index] = TranspositionTableEntry {
            evaluation,
            depth,
            key,
            flag,
        };
    }

    pub fn hash_from_board(&self, b: &Board) -> usize {
        zobrist_hash(b, self.attacker_bits_seed, &self.init_hash)
    }
}

// Zobrist hashing
fn update_hash_with_board(
    h: &mut usize,
    board: Bitboard,
    init_board: &[[usize; 3]; NUM_SQUARES],
    piece_type_idx: usize,
) {
    let mut bb = board;
    while bb != 0 {
        let idx = bb.trailing_zeros() as usize;
        *h ^= init_board[idx][piece_type_idx];
        bb &= bb - 1;
    }
}

fn zobrist_hash(b: &Board, attacker_bits: usize, init_board: &[[usize; 3]; NUM_SQUARES]) -> usize {
    let mut hash = 0;
    if b.attacker_move {
        hash ^= attacker_bits;
    }
    update_hash_with_board(
        &mut hash,
        b.attacker_board,
        init_board,
        PIECE_TYPE_ATTACKER_IDX,
    );
    update_hash_with_board(
        &mut hash,
        b.defender_board,
        init_board,
        PIECE_TYPE_DEFENDER_IDX,
    );
    update_hash_with_board(&mut hash, b.king_board, init_board, PIECE_TYPE_KING_IDX);
    hash
}

fn make_init_hash() -> [[usize; 3]; NUM_SQUARES] {
    let mut init_board = [[0; 3]; NUM_SQUARES];
    let mut rng = rand::thread_rng();
    for i in 0..NUM_SQUARES {
        for j in 0..3 {
            init_board[i][j] = rng.gen::<usize>();
        }
    }

    init_board
}
