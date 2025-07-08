use crate::board::{Board, Move, NULL_MOVE};
use crate::eval::naive_eval;
use crate::movegen::MoveGenerator;
use crate::ttable::{Flag, TranspositionTable};

const ASPIRATION_WINDOW: i16 = 300;

pub struct TaflAI {
    pub max_depth: u8,
    pub ttable: TranspositionTable,
}

impl TaflAI {
    pub fn find_best_move(&mut self, b: &mut Board) -> EngineRecommendation {
        let mut nnodes = 1;
        let mut best_move = NULL_MOVE;
        let mut best_eval = i16::MIN + 1;
        let color = if b.attacker_move { 1 } else { -1 };

        let mut root_moves = MoveGenerator::new(b).cached_moves;
        for current_depth in 1..=self.max_depth {   
            let mut nnodes_this_iter = 0;
            let mut best_move_this_iter = NULL_MOVE;
            let mut best_eval_this_iter = i16::MIN;
            let mut alpha = best_eval.saturating_sub(ASPIRATION_WINDOW);
            let mut beta = best_eval.saturating_add(ASPIRATION_WINDOW);

            if current_depth > 1 {
                if let Some(pos) = root_moves.iter().position(|&m| m == best_move) {
                    let pv_move = root_moves.remove(pos);
                    root_moves.insert(0, pv_move);
                }
            }

            loop {
                for &m in root_moves.iter() {
                    b.make_move(m, &self.ttable);
                    let eval = -negamax(
                        self,
                        b,
                        current_depth - 1,
                        &mut nnodes_this_iter,
                        alpha,
                        beta,
                        -color,
                    );
                    b.unmake_move();

                    if eval > best_eval_this_iter {
                        best_eval_this_iter = eval;
                        best_move_this_iter = m;
                    }
                }

                if best_eval_this_iter <= alpha {
                    alpha = i16::MIN + 1;
                } else if best_eval_this_iter >= beta {
                    beta = i16::MAX - 1;
                } else {
                    break;
                }
            }

            nnodes += nnodes_this_iter;
            best_eval = best_eval_this_iter;
            best_move = best_move_this_iter;
        }

        EngineRecommendation {
            evaluation: best_eval * color,
            best_move: best_move,
            nnodes: nnodes,
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

fn negamax(
    tafl_ai: &mut TaflAI,
    b: &mut Board,
    depth: u8,
    nnodes: &mut usize,
    mut alpha: i16,
    beta: i16,
    color: i16,
) -> i16 {
    *nnodes += 1;
    if depth == 0 {
        return naive_eval(b) * color;
    }

    if b.attacker_win {
        return (10000 + depth as i16) * color;
    } else if b.defender_win {
        return (-10000 - depth as i16) * color;
    }

    let original_alpha = alpha;
    if let Some(entry) = tafl_ai.ttable.retrieve(b) {
        if entry.depth >= depth {
            match entry.flag {
                Flag::EXACT => return entry.evaluation,
                Flag::LOWERBOUND => {
                    if entry.evaluation >= beta {
                        return entry.evaluation;
                    }
                }
                Flag::UPPERBOUND => {
                    if entry.evaluation <= alpha {
                        return entry.evaluation;
                    }
                }
            }
        }
    }

    let moves = MoveGenerator::new(b);
    let mut value = i16::MIN;
    for mv in moves {
        b.make_move(mv, &tafl_ai.ttable);
        let eval = -negamax(tafl_ai, b, depth - 1, nnodes, -beta, -alpha, -color);
        b.unmake_move();

        value = value.max(eval);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }

    let flag = if value <= original_alpha {
        Flag::UPPERBOUND
    } else if value >= beta {
        Flag::LOWERBOUND
    } else {
        Flag::EXACT
    };

    tafl_ai.ttable.store(b, value, depth, flag);

    return value;
}
