use std::io::{Read, Write};

use crate::board::{Board, Move};
use crate::engine::EngineBenchmark;

pub trait UI {
    fn get_move(&mut self) -> Move;
    fn render_board(&self, b: &Board);
    fn render_eval(&self, benchmark: &EngineBenchmark);
    fn invalid_move(&self);
    fn attacker_win(&self);
    fn defender_win(&self);
    fn stalemate(&self);
}

pub struct ConsoleUI {
    stdin: std::io::Stdin,
}

impl ConsoleUI {
    pub fn new() -> ConsoleUI {
        return ConsoleUI {
            stdin: std::io::stdin(),
        };
    }
}

impl UI for ConsoleUI {
    fn get_move(&mut self) -> Move {
        print!("Make a move: ");
        std::io::stdout().flush().unwrap();
        let mut buf = [0; 10];
        let t = self.stdin.read(&mut buf);

        match t {
            Ok(n) => {
                let mut move_str = String::new();
                for i in 0..n {
                    move_str.push(buf[i] as char);
                }
                let m = parse_move(&move_str);
                match m {
                    Some(mv) => {
                        return mv;
                    }
                    None => {
                        print!("Invalid Move!\n");
                        return self.get_move();
                    }
                }
            }
            Err(_) => {
                print!("Invalid Move!\n");
                return self.get_move();
            }
        }
    }

    fn render_board(&self, b: &Board) {
        println!("{}", b.to_string());
    }

    fn render_eval(&self, benchmark: &EngineBenchmark) {
        println!("Recommended Move: {}", benchmark.recommendation.best_move.to_string());
        println!(
            "Evaluation: {} ({} nodes) ({:.2?})",
            benchmark.recommendation.evaluation,
            benchmark.recommendation.nnodes,
            benchmark.elapsed
        );
    }

    fn invalid_move(&self) {
        println!("Invalid Move!");
    }

    fn attacker_win(&self) {
        println!("Attacker Wins!");
    }

    fn defender_win(&self) {
        println!("Defender Wins!");
    }

    fn stalemate(&self) {
        println!("Stalemate!");
    }
}
fn validate_move_num(c: Option<char>, sub_val: u64) -> Option<u64> {
    c.and_then(|x| {
        let value = x as u64;
        if value >= sub_val {
            Some(value - sub_val)
        } else {
            None
        }
    })
}

fn parse_num(c: &mut std::str::Chars, dir: char) -> Option<u64> {
    if let Some(ch) = c.next() {
        validate_move_num(Some(ch), dir as u64)
    } else {
        None
    }
}

pub fn parse_move(s: &String) -> Option<Move> {
    let mut m = Move {
        start_row: 0,
        start_col: 0,
        end_row: 0,
        end_col: 0,
        king_move: false,
    };

    let mut c = s.chars();
    if c.clone().nth(0) == Some('k') {
        m.king_move = true;
        c.next(); // Consume 'k'
    }

    if m.king_move {
        m.end_col = parse_num(&mut c, 'a')?;
        m.end_row = parse_num(&mut c, '1')?;
    } else {
        m.start_col = parse_num(&mut c, 'a')?;
        m.start_row = parse_num(&mut c, '1')?;
        m.end_col = parse_num(&mut c, 'a')?;
        m.end_row = parse_num(&mut c, '1')?;
    }

    Some(m)
}
