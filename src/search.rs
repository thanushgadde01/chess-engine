use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use cozy_chess::{Board, GameStatus, Move};

use crate::eval::evaluate;

const MATE: i32 = 30_000;

#[derive(Clone, Debug, Default)]
pub struct SearchLimits {
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub movetime: Option<u64>, // milliseconds
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
    pub infinite: bool,
}

pub struct SearchContext {
    pub nodes: u64,
    pub stop_flag: Arc<AtomicBool>,
    pub start_time: Instant,
    pub time_limit_ms: Option<u64>,
    pub max_nodes: Option<u64>,
}

impl SearchContext {
    pub fn new(stop_flag: Arc<AtomicBool>, time_limit_ms: Option<u64>, max_nodes: Option<u64>) -> Self {
        Self {
            nodes: 0,
            stop_flag,
            start_time: Instant::now(),
            time_limit_ms,
            max_nodes,
        }
    }

    pub fn should_stop(&mut self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        // Check time every 2048 nodes to avoid clock query overhead
        if (self.nodes & 2047) == 0 {
            if let Some(limit) = self.time_limit_ms {
                if self.start_time.elapsed().as_millis() as u64 >= limit {
                    self.stop_flag.store(true, Ordering::Relaxed);
                    return true;
                }
            }
            if let Some(max_n) = self.max_nodes {
                if self.nodes >= max_n {
                    self.stop_flag.store(true, Ordering::Relaxed);
                    return true;
                }
            }
        }

        false
    }
}

pub fn search(
    board: &Board,
    limits: SearchLimits,
    stop_flag: Arc<AtomicBool>,
) -> (Option<Move>, u64) {
    let moves = legal_moves(board);
    if moves.is_empty() {
        return (None, 0);
    }

    // Determine time limit
    let time_limit = if let Some(mt) = limits.movetime {
        Some(mt.saturating_sub(10).max(1)) // 10ms safety buffer
    } else {
        let (my_time, my_inc) = match board.side_to_move() {
            cozy_chess::Color::White => (limits.wtime, limits.winc),
            cozy_chess::Color::Black => (limits.btime, limits.binc),
        };
        if let Some(t) = my_time {
            let moves_to_go = limits.movestogo.unwrap_or(30).clamp(1, 50);
            let inc = my_inc.unwrap_or(0);
            let allocated = (t / moves_to_go) + (inc * 3 / 4);
            Some(allocated.min(t.saturating_sub(50)).max(10))
        } else {
            None
        }
    };

    let max_depth = limits.depth.unwrap_or(if limits.infinite || time_limit.is_some() { 64 } else { 4 });
    let mut ctx = SearchContext::new(stop_flag, time_limit, limits.nodes);

    let mut best_move = Some(moves[0]);

    // Iterative deepening
    for d in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        let mut current_best = None;
        let mut best_score = -MATE;
        let mut alpha = -MATE;
        let beta = MATE;

        for &mv in &moves {
            let mut next = board.clone();
            next.play(mv);
            ctx.nodes += 1;

            let score = -negamax(&next, d - 1, -beta, -alpha, &mut ctx);

            if ctx.stop_flag.load(Ordering::Relaxed) {
                break;
            }

            if score > best_score {
                best_score = score;
                current_best = Some(mv);
            }
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        if !ctx.stop_flag.load(Ordering::Relaxed) && current_best.is_some() {
            best_move = current_best;
            let elapsed_ms = ctx.start_time.elapsed().as_millis().max(1);
            let nps = (ctx.nodes as u128 * 1000) / elapsed_ms;
            let score_str = if best_score.abs() > MATE - 100 {
                let mate_in = (MATE - best_score.abs() + 1) / 2;
                if best_score > 0 {
                    format!("mate {mate_in}")
                } else {
                    format!("mate -{mate_in}")
                }
            } else {
                format!("cp {best_score}")
            };
            println!(
                "info depth {d} score {score_str} nodes {} time {elapsed_ms} nps {nps} pv {}",
                ctx.nodes,
                best_move.unwrap()
            );
        } else {
            break;
        }
    }

    (best_move, ctx.nodes)
}

fn negamax(board: &Board, depth: u8, mut alpha: i32, beta: i32, ctx: &mut SearchContext) -> i32 {
    if ctx.should_stop() {
        return 0;
    }

    if depth == 0 {
        return evaluate(board);
    }

    if board.status() != GameStatus::Ongoing {
        return if board.checkers().is_empty() {
            0
        } else {
            -MATE + (100 - depth as i32) // Prefer shorter mate
        };
    }

    let moves = legal_moves(board);
    if moves.is_empty() {
        return evaluate(board);
    }

    let mut best = -MATE;
    for mv in moves {
        let mut next = board.clone();
        next.play(mv);
        ctx.nodes += 1;

        let score = -negamax(&next, depth - 1, -beta, -alpha, ctx);

        if ctx.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    best
}

pub fn legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    board.generate_moves(|piece_moves| {
        moves.extend(piece_moves);
        false
    });
    moves
}
