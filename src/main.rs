use std::time::Instant;

use movegen::MoveGenerator;
use ui::UI;

mod board;
mod engine;
mod eval;
mod movegen;
mod ttable;
mod ui;

fn init_board() -> board::Board {
    let a = [
        [false, false, true, true, true, false, false],
        [false, false, false, true, false, false, false],
        [true, false, false, false, false, false, true],
        [true, true, false, false, false, true, true],
        [true, false, false, false, false, false, true],
        [false, false, false, true, false, false, false],
        [false, false, true, true, true, false, false],
    ];

    let d = [
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, true, true, true, false, false],
        [false, false, true, false, true, false, false],
        [false, false, true, true, true, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
    ];

    let k = [
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, true, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
    ];

    let ol = [
        [true, false, false, false, false, false, true],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, true, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [true, false, false, false, false, false, true],
    ];

    return board::Board {
        attacker_board: board::bool_array_to_bitboard(a),
        defender_board: board::bool_array_to_bitboard(d),
        king_board: board::bool_array_to_bitboard(k),
        offlimits_board: board::bool_array_to_bitboard(ol),
        attacker_move: false,
        attacker_win: false,
        defender_win: false,
        stalemate: false,
    };
}

fn init_board_2() -> board::Board {
    let a = [
        [false, false, false, false, true, false, false],
        [false, false, false, true, true, false, false],
        [true, false, false, false, false, false, true],
        [true, false, false, false, false, true, true],
        [true, false, true, false, false, false, true],
        [false, false, false, false, false, false, false],
        [false, false, true, true, true, false, false],
    ];

    let d = [
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, true, false, true, true, false, false],
        [false, false, false, false, true, false, false],
        [false, false, false, true, true, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
    ];

    let k = [
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, true, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
    ];

    let ol = [
        [true, false, false, false, false, false, true],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, true, false, false, false],
        [false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false],
        [true, false, false, false, false, false, true],
    ];

    return board::Board {
        attacker_board: board::bool_array_to_bitboard(a),
        defender_board: board::bool_array_to_bitboard(d),
        king_board: board::bool_array_to_bitboard(k),
        offlimits_board: board::bool_array_to_bitboard(ol),
        attacker_move: false,
        attacker_win: false,
        defender_win: false,
        stalemate: false,
    };
}

fn main() {
    let mut b = init_board();

    let mut tafl_ai = engine::TaflAI {
        max_depth: 6,
    };

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
        console_ui.render_board(&b);
        let now = Instant::now();
        let recommendation = tafl_ai.find_best_move(&b);
        let elapsed = now.elapsed();
        let benchmark = engine::EngineBenchmark {
            recommendation,
            elapsed,
        };
        console_ui.render_eval(&benchmark);

        let mut mv = console_ui.get_move();

        let legal_moves = MoveGenerator::new(&b).collect::<Vec<_>>();
        while !legal_moves.contains(&mv) {
            console_ui.invalid_move();
            mv = console_ui.get_move();
        }
        b = b.make_move(mv);
    }
}
