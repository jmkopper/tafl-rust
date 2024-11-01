use crate::board::{Board, Move, NULL_MOVE};
use crate::eval::naive_eval;
use crate::movegen::MoveGenerator;
use crate::ttable::TranspositionTable;

pub struct TaflAI {
    pub max_depth: u8,
    pub ttable: TranspositionTable,
}

impl TaflAI {
    pub fn find_best_move(&mut self, b: &Board) -> EngineRecommendation {
        return alphabeta_search(self, b, self.max_depth - 1);
    }
}

pub struct EngineRecommendation {
    pub evaluation: i16,
    pub best_move: Move,
    pub nnodes: usize,
}

pub struct EngineBenchmark {
    pub recommendation: EngineRecommendation,
    pub elapsed: std::time::Duration,
}

fn alphabeta_search(tafl_ai: &mut TaflAI, b: &Board, max_depth: u8) -> EngineRecommendation {
    let mut nnodes = 1;
    let mut best_move = NULL_MOVE;
    let mut alpha = i16::MIN;
    let mut beta = i16::MAX;
    let mut best_eval: i16;

    if b.attacker_move {
        best_eval = i16::MIN;
        for m in MoveGenerator::new(b) {
            let new_board = b.make_move(m);
            let rec_val = alphabeta_min(tafl_ai, &new_board, max_depth, &mut nnodes, alpha, beta);
            if rec_val > best_eval {
                best_move = m;
                best_eval = rec_val;
            }

            if rec_val > alpha {
                alpha = rec_val;
            }

            if rec_val >= beta {
                break;
            }
        }
    } else {
        best_eval = i16::MAX;
        for m in MoveGenerator::new(b) {
            let new_board = b.make_move(m);
            let rec_val = alphabeta_max(tafl_ai, &new_board, max_depth, &mut nnodes, alpha, beta);
            if rec_val < best_eval {
                best_move = m;
                best_eval = rec_val;
            }

            if rec_val < beta {
                beta = best_eval;
            }
            if rec_val <= alpha {
                break;
            }
        }
    }

    return EngineRecommendation {
        evaluation: best_eval,
        best_move,
        nnodes,
    };
}

fn alphabeta_max(
    tafl_ai: &mut TaflAI,
    b: &Board,
    depth: u8,
    nnodes: &mut usize,
    mut alpha: i16,
    beta: i16,
) -> i16 {
    *nnodes += 1;

    if depth == 0 {
        return naive_eval(b);
    }

    if b.defender_win {
        return -10000 - depth as i16;
    }

    if let Some(entry) = tafl_ai.ttable.retrieve(b) {
        if entry.depth >= depth as usize {
            return entry.evaluation;
        }
    }

    let mut max_eval = i16::MIN;
    let mut move_count = 0;

    for m in MoveGenerator::new(b) {
        move_count += 1;
        let new_board = b.make_move(m);
        let rec_val = alphabeta_min(tafl_ai, &new_board, depth - 1, nnodes, alpha, beta);

        if rec_val > max_eval {
            max_eval = rec_val;
            if rec_val > alpha {
                alpha = max_eval;
            }
        }

        if rec_val >= beta {
            tafl_ai.ttable.store(b, max_eval, depth as usize);
            return rec_val;
        }
    }

    if move_count == 0 {
        return 0;
    }

    tafl_ai.ttable.store(b, max_eval, depth as usize);
    return max_eval;
}

fn alphabeta_min(
    tafl_ai: &mut TaflAI,
    b: &Board,
    depth: u8,
    nnodes: &mut usize,
    alpha: i16,
    mut beta: i16,
) -> i16 {
    *nnodes += 1;

    if depth == 0 {
        return naive_eval(b);
    }

    if b.attacker_win {
        return 10000 + depth as i16;
    }

    if let Some(entry) = tafl_ai.ttable.retrieve(b) {
        if entry.depth >= depth as usize {
            return entry.evaluation;
        }
    }

    let mut min_eval = i16::MAX;
    let mut move_count = 0;

    for m in MoveGenerator::new(b) {
        move_count += 1;
        let new_board = b.make_move(m);
        let rec_val = alphabeta_max(tafl_ai, &new_board, depth - 1, nnodes, alpha, beta);

        if rec_val < min_eval {
            min_eval = rec_val;
            if rec_val < beta {
                beta = min_eval;
            }
        }

        if rec_val <= alpha {
            tafl_ai.ttable.store(b, min_eval, depth as usize);
            return rec_val;
        }
    }

    if move_count == 0 {
        return 0;
    }
    tafl_ai.ttable.store(b, min_eval, depth as usize);
    return min_eval;
}
