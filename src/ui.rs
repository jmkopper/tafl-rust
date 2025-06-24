use std::io::{Read, Write};

use crate::board::{rc_to_index, Board, Move, PieceType};
use crate::engine::EngineBenchmark;

pub trait UI {
    fn get_move(&mut self, b: &Board) -> Move;
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
    fn get_move(&mut self, b: &Board) -> Move {
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
                let m = parse_move(&move_str, b.attacker_move);
                match m {
                    Some(mv) => {
                        return mv;
                    }
                    None => {
                        print!("Unable to parse move!\n");
                        return self.get_move(b);
                    }
                }
            }
            Err(_) => {
                print!("Unable to parse move!\n");
                return self.get_move(b);
            }
        }
    }

    fn render_board(&self, b: &Board) {
        println!("{}", b.to_string());
    }

    fn render_eval(&self, benchmark: &EngineBenchmark) {
        println!(
            "Recommended Move: {}",
            benchmark.recommendation.best_move.to_string()
        );
        println!(
            "Evaluation: {} ({} nodes) ({:.2?})",
            benchmark.recommendation.evaluation, benchmark.recommendation.nnodes, benchmark.elapsed
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
fn validate_move_num(c: Option<char>, sub_val: usize) -> Option<usize> {
    c.and_then(|x| {
        let value = x as usize;
        if value >= sub_val {
            Some(value - sub_val)
        } else {
            None
        }
    })
}

fn parse_num(c: &mut std::str::Chars, dir: char) -> Option<usize> {
    if let Some(ch) = c.next() {
        validate_move_num(Some(ch), dir as usize)
    } else {
        None
    }
}

pub fn parse_move(s: &String, attacker_move: bool) -> Option<Move> {
    let mut m = Move {
        start_index: 0,
        end_index: 0,
        piece_type: PieceType::Defender,
    };

    let mut c = s.chars();
    if c.clone().nth(0) == Some('k') {
        m.piece_type = PieceType::King;
        c.next(); // Consume 'k'
    }

    if attacker_move {
        m.piece_type = PieceType::Attacker;
    }

    let end_row: usize;
    let end_col: usize;
    let start_row: usize;
    let start_col: usize;

    start_col = parse_num(&mut c, 'a')?;
    start_row = parse_num(&mut c, '1')?;
    end_col = parse_num(&mut c, 'a')?;
    end_row = parse_num(&mut c, '1')?;

    m.start_index = rc_to_index(start_row, start_col);
    m.end_index = rc_to_index(end_row, end_col);

    Some(m)
}
