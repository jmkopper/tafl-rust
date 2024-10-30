use crate::board::{Board, Move, NULL_MOVE};
use crate::eval::naive_eval;
use crate::movegen::MoveGenerator;

pub struct TaflAI {
    pub max_depth: u8,
}

impl TaflAI {
    pub fn find_best_move(&mut self, b: &Board) -> EngineRecommendation {
        let mut nnodes = 0usize;
        if b.attacker_move {
            return alphabeta_max(b, self.max_depth, &mut nnodes, i16::MIN, i16::MAX);
        } else {
            return alphabeta_min(b, self.max_depth, &mut nnodes, i16::MIN, i16::MAX);
        }
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

fn alphabeta_max(
    b: &Board,
    depth: u8,
    nnodes: &mut usize,
    mut alpha: i16,
    beta: i16,
) -> EngineRecommendation {
    *nnodes += 1;

    if depth == 0 {
        return EngineRecommendation {
            evaluation: naive_eval(&b),
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.attacker_win {
        return EngineRecommendation {
            evaluation: 10000 + depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.defender_win {
        return EngineRecommendation {
            evaluation: -10000 - depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    let mut max_eval = i16::MIN;
    let mut best_move = NULL_MOVE;

    for m in MoveGenerator::new(b) {
        let new_board = b.make_move(m);
        let rec_val = alphabeta_min(&new_board, depth - 1, nnodes, alpha, beta);

        if rec_val.evaluation > max_eval {
            max_eval = rec_val.evaluation;
            if max_eval == 10000 {
                max_eval += depth as i16;
            }
            best_move = m;
            if rec_val.evaluation > alpha {
                alpha = max_eval;
            }
        }

        if rec_val.evaluation >= beta {
            return EngineRecommendation {
                evaluation: rec_val.evaluation,
                best_move: m,
                nnodes: *nnodes,
            };
        }
    }

    return EngineRecommendation {
        evaluation: max_eval,
        best_move: best_move,
        nnodes: *nnodes,
    };
}

fn alphabeta_min(
    b: &Board,
    depth: u8,
    nnodes: &mut usize,
    alpha: i16,
    mut beta: i16,
) -> EngineRecommendation {
    *nnodes += 1;

    if depth == 0 {
        return EngineRecommendation {
            evaluation: naive_eval(&b),
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.attacker_win {
        return EngineRecommendation {
            evaluation: 10000 + depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.defender_win {
        return EngineRecommendation {
            evaluation: -10000 - depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    let mut min_eval = i16::MAX;
    let mut best_move = NULL_MOVE;

    for m in MoveGenerator::new(b) {
        let new_board = b.make_move(m);
        let rec_val = alphabeta_max(&new_board, depth - 1, nnodes, alpha, beta);

        if rec_val.evaluation < min_eval {
            min_eval = rec_val.evaluation;
            if min_eval == -10000 {
                min_eval -= depth as i16;
            }
            best_move = m;
            if rec_val.evaluation < beta {
                beta = min_eval;
            }
        }

        if rec_val.evaluation <= alpha {
            return EngineRecommendation {
                evaluation: rec_val.evaluation,
                best_move: m,
                nnodes: *nnodes,
            };
        }
    }

    return EngineRecommendation {
        evaluation: min_eval,
        best_move: best_move,
        nnodes: *nnodes,
    };
}

fn minimax(b: &Board, depth: u8, max_player: bool, nnodes: &mut usize) -> EngineRecommendation {
    *nnodes += 1;
    if depth == 0 {
        return EngineRecommendation {
            evaluation: naive_eval(&b),
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.attacker_win {
        return EngineRecommendation {
            evaluation: 10000 + depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }

    if b.defender_win {
        return EngineRecommendation {
            evaluation: -10000 - depth as i16,
            best_move: NULL_MOVE,
            nnodes: *nnodes,
        };
    }
    if max_player {
        let mut max_eval = i16::MIN;
        let mut best_move = NULL_MOVE;

        for m in MoveGenerator::new(&b) {
            let new_board = b.make_move(m);
            let eval = minimax(&new_board, depth - 1, false, nnodes);
            if eval.evaluation > max_eval {
                max_eval = eval.evaluation;
                best_move = m;
            }
        }
        return EngineRecommendation {
            evaluation: max_eval,
            best_move: best_move,
            nnodes: *nnodes,
        };
    } else {
        let mut min_eval = i16::MAX;
        let mut best_move = NULL_MOVE;

        for m in MoveGenerator::new(&b) {
            let new_board = b.make_move(m);
            let eval = minimax(&new_board, depth - 1, true, nnodes);
            if eval.evaluation < min_eval {
                min_eval = eval.evaluation;
                best_move = m;
            }
            min_eval = min_eval.min(eval.evaluation);
        }
        return EngineRecommendation {
            evaluation: min_eval,
            best_move: best_move,
            nnodes: *nnodes,
        };
    }
}
