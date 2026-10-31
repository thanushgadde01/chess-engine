use cozy_chess::{Board, GameStatus, Move};

use crate::eval::evaluate;

const MATE: i32 = 30_000;

pub fn search(board: &Board, depth: u8) -> Option<Move> {
    let mut moves = legal_moves(board);
    let first = moves.pop()?;
    let mut best = first;
    let mut best_score = i32::MIN + 1;
    let mut candidates = vec![first];
    candidates.extend(moves);

    for mv in candidates {
        let mut next = board.clone();
        next.play(mv);
        let score = -negamax(&next, depth.saturating_sub(1), -MATE, MATE);
        if score > best_score {
            best_score = score;
            best = mv;
        }
    }
    Some(best)
}

fn negamax(board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }
    if board.status() != GameStatus::Ongoing {
        return if board.checkers().is_empty() {
            0
        } else {
            -MATE
        };
    }

    let moves = legal_moves(board);
    if moves.is_empty() {
        return evaluate(board);
    }
    let mut best = i32::MIN + 1;
    for mv in moves {
        let mut next = board.clone();
        next.play(mv);
        let score = -negamax(&next, depth - 1, -beta, -alpha);
        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }
    best
}

fn legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    board.generate_moves(|piece_moves| {
        moves.extend(piece_moves);
        false
    });
    moves
}
