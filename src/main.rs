use std::time::Instant;

use movegen::MoveGenerator;
use ui::UI;

mod board;
mod engine;
mod eval;
mod movegen;
mod ttable;
mod ui;

fn main() {
    let mut b = board::STARTING_BOARD;
    let mut tafl_ai = engine::TaflAI {
        max_depth: 9,
        ttable: ttable::TranspositionTable::new(),
    };
    b.current_hash = tafl_ai.ttable.hash_from_board(&b);
    let mut console_ui = ui::ConsoleUI::new();

    loop {
        if b.defender_win {
            console_ui.defender_win();
            break;
        } else if b.attacker_win {
            console_ui.attacker_win();
            break;
        } else if b.stalemate {
            console_ui.stalemate();
            break;
        }

        let now = Instant::now();
        let mut b_for_eval = b.clone();
        let recommendation = tafl_ai.find_best_move(&mut b_for_eval);
        let elapsed = now.elapsed();
        let benchmark = engine::EngineBenchmark {
            recommendation,
            elapsed,
        };
        console_ui.render_eval(&benchmark);
        console_ui.render_board(&b);
        let mut mv = console_ui.get_move(&b);

        let legal_moves = MoveGenerator::new(&b).collect::<Vec<_>>();

        if legal_moves.len() == 0 {
            console_ui.stalemate();
            break;
        }
        while !legal_moves.contains(&mv) {
            console_ui.invalid_move();
            mv = console_ui.get_move(&b);
        }
        b.make_move(mv, &tafl_ai.ttable);
    }
}
