use crate::board::{inbounds, rc_to_index, Bitboard, Board, BOARD_SIZE, DIRS};

fn total_board(b: Bitboard) -> i16 {
    return b.count_ones() as i16;
}

fn attackers_next_to_king(b: &Board) -> i16 {
    let mut s = 0;
    let (king_row, king_col) = b.king_coordinates();
    for &(dr, dc) in DIRS.iter() {
        let new_row = king_row as isize + dr;
        let new_col = king_col as isize + dc;
        if !inbounds(new_row, new_col) {
            continue;
        }
        let new_index = rc_to_index(new_row as usize, new_col as usize);
        if b.attacker_board & (1 << new_index) != 0 {
            s += 1;
        }
    }
    return s;
}

fn dist_to_corner(b: &Board) -> i16 {
    let (mut king_row, mut king_col) = b.king_coordinates();

    if king_row > BOARD_SIZE / 2 {
        king_row = BOARD_SIZE - king_row;
    }
    if king_col > BOARD_SIZE / 2 {
        king_col = BOARD_SIZE - king_col;
    }

    return (king_row + king_col) as i16;
}

pub fn naive_eval(b: &Board) -> i16 {
    if b.attacker_win {
        return 10000;
    }
    if b.defender_win {
        return -10000;
    }
    if b.stalemate {
        return 0;
    }

    let attacker_weight = 1;
    let defender_weight = 1;
    let mut attack_score = total_board(b.attacker_board);
    let mut defender_score = total_board(b.defender_board) + total_board(b.king_board);

    let attackers_next_to_king = attackers_next_to_king(b);
    let dist_to_corner = dist_to_corner(b);
    defender_score -= dist_to_corner;
    attack_score += attackers_next_to_king;

    return attacker_weight * attack_score - defender_weight * defender_score;
}
