use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use cozy_chess::{Board, GameStatus, Move, Piece};

use crate::board::Position;
use crate::eval::evaluate;
use crate::timeman::TimeManager;

pub const MATE: i32 = 30_000;
pub const MATE_BOUND: i32 = MATE - 1_000;

#[derive(Clone, Debug, Default)]
pub struct SearchLimits {
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub movetime: Option<u64>,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
    pub infinite: bool,
}

pub struct SearchContext<'a> {
    pub nodes: u64,
    pub stop_flag: Arc<AtomicBool>,
    pub start_time: Instant,
    pub time_limit_ms: Option<u64>,
    pub max_nodes: Option<u64>,
    pub position_history: &'a [u64],
}

impl<'a> SearchContext<'a> {
    pub fn new(
        stop_flag: Arc<AtomicBool>,
        time_limit_ms: Option<u64>,
        max_nodes: Option<u64>,
        position_history: &'a [u64],
    ) -> Self {
        Self {
            nodes: 0,
            stop_flag,
            start_time: Instant::now(),
            time_limit_ms,
            max_nodes,
            position_history,
        }
    }

    #[inline(always)]
    pub fn should_stop(&mut self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        // Check periodic clock/nodes every 2048 nodes
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
    position: &Position,
    limits: SearchLimits,
    stop_flag: Arc<AtomicBool>,
) -> (Option<Move>, u64) {
    let board = position.board();
    let moves = legal_moves(board);
    if moves.is_empty() {
        return (None, 0);
    }

    // Determine time allocation
    let time_limit = if let Some(mt) = limits.movetime {
        Some(mt.saturating_sub(10).max(1))
    } else {
        let (my_time, my_inc) = match board.side_to_move() {
            cozy_chess::Color::White => (limits.wtime, limits.winc),
            cozy_chess::Color::Black => (limits.btime, limits.binc),
        };
        TimeManager::allocate(my_time, my_inc, limits.movestogo).map(|d| d.as_millis() as u64)
    };

    let max_depth = limits.depth.unwrap_or(if limits.infinite || time_limit.is_some() { 64 } else { 4 });
    let mut ctx = SearchContext::new(stop_flag, time_limit, limits.nodes, position.history());

    let mut best_move = Some(moves[0]);
    let mut history_stack = Vec::with_capacity(64);

    // Iterative deepening
    for d in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        let mut current_best = None;
        let mut best_score = -MATE;
        let mut alpha = -MATE;
        let beta = MATE;

        // Order the best move from the previous iteration first
        let mut ordered_moves = moves.clone();
        if let Some(prev_best) = best_move {
            if let Some(idx) = ordered_moves.iter().position(|&m| m == prev_best) {
                ordered_moves.swap(0, idx);
            }
        }

        for &mv in &ordered_moves {
            let mut next = board.clone();
            next.play(mv);
            ctx.nodes += 1;

            history_stack.push(next.hash());
            let score = -negamax(&next, d - 1, 1, -beta, -alpha, &mut ctx, &mut history_stack);
            history_stack.pop();

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
            let score_str = if best_score.abs() > MATE_BOUND {
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

            // If we found a forced mate score at root, we can conclude search
            if best_score.abs() > MATE_BOUND && d >= ((MATE - best_score.abs()) as u8) {
                break;
            }
        } else {
            break;
        }
    }

    (best_move, ctx.nodes)
}

fn is_draw(board: &Board, ctx: &SearchContext, search_history: &[u64]) -> bool {
    // 50-move rule
    if board.halfmove_clock() >= 100 {
        return true;
    }

    // Insufficient material
    let pieces = board.occupied();
    if pieces.len() == 2 {
        return true; // King vs King
    }
    if pieces.len() == 3 {
        let knights = board.pieces(Piece::Knight);
        let bishops = board.pieces(Piece::Bishop);
        if !knights.is_empty() || !bishops.is_empty() {
            return true; // King + Minor vs King
        }
    }

    // Repetition check against prior game history
    let hash = board.hash();
    if search_history.len() > 1 && search_history[..search_history.len() - 1].iter().any(|&h| h == hash) {
        return true;
    }
    if ctx.position_history.iter().filter(|&&h| h == hash).count() >= 2 {
        return true;
    }

    false
}

fn negamax(
    board: &Board,
    depth: u8,
    ply: usize,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchContext,
    search_history: &mut Vec<u64>,
) -> i32 {
    if ctx.should_stop() {
        return 0;
    }

    // Check game termination (Checkmate / Stalemate)
    if board.status() != GameStatus::Ongoing {
        return if board.checkers().is_empty() {
            0
        } else {
            -MATE + (ply as i32) // Shorter mate is favored
        };
    }

    // Draw detection (repetition, 50-move rule, insufficient material)
    if ply > 0 && is_draw(board, ctx, search_history) {
        return 0;
    }

    // Quiescence search at leaf nodes
    if depth == 0 {
        return quiescence(board, alpha, beta, ctx, 0);
    }

    let moves = legal_moves(board);
    if moves.is_empty() {
        return if board.checkers().is_empty() {
            0
        } else {
            -MATE + (ply as i32)
        };
    }

    let mut best = -MATE;
    for mv in moves {
        let mut next = board.clone();
        next.play(mv);
        ctx.nodes += 1;

        search_history.push(next.hash());
        let score = -negamax(&next, depth - 1, ply + 1, -beta, -alpha, ctx, search_history);
        search_history.pop();

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

// Quiescence search evaluates tactical captures until a quiet position is reached
fn quiescence(board: &Board, mut alpha: i32, beta: i32, ctx: &mut SearchContext, q_depth: u8) -> i32 {
    if ctx.should_stop() {
        return 0;
    }

    // Stand-pat evaluation
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    alpha = alpha.max(stand_pat);

    // Depth limit to prevent qsearch explosion
    if q_depth >= 8 {
        return alpha;
    }

    // Generate only captures / promotions
    let mut captures = Vec::new();
    let enemy = !board.side_to_move();
    let enemy_occupied = board.colors(enemy);

    board.generate_moves(|piece_moves| {
        for mv in piece_moves {
            // Delta pruning: if stand_pat + captured piece max value + margin < alpha, skip
            let target_piece = board.piece_on(mv.to);
            let is_ep = match board.en_passant() {
                Some(ep_file) => board.pieces(Piece::Pawn).has(mv.from) && mv.to.file() == ep_file && (mv.to.rank() == cozy_chess::Rank::Third || mv.to.rank() == cozy_chess::Rank::Sixth),
                None => false,
            };
            let is_capture = enemy_occupied.has(mv.to) || is_ep;
            let is_promotion = mv.promotion.is_some();
            if is_capture || is_promotion {
                // Delta pruning: don't search if capture has no chance to raise alpha
                let gain = if is_promotion {
                    900
                } else if let Some(p) = target_piece {
                    match p {
                        Piece::Pawn => 100,
                        Piece::Knight => 320,
                        Piece::Bishop => 330,
                        Piece::Rook => 500,
                        Piece::Queen => 900,
                        Piece::King => 0,
                    }
                } else {
                    100
                };
                if stand_pat + gain + 200 >= alpha {
                    captures.push((mv, gain));
                }
            }
        }
        false
    });

    // MVV-LVA move ordering in quiescence: sort captures by captured piece value desc
    captures.sort_unstable_by_key(|&(_, gain)| -gain);

    for (mv, _) in captures {
        let mut next = board.clone();
        next.play(mv);
        ctx.nodes += 1;

        let score = -quiescence(&next, -beta, -alpha, ctx, q_depth + 1);

        if ctx.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);
    }

    alpha
}

pub fn legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    board.generate_moves(|piece_moves| {
        moves.extend(piece_moves);
        false
    });
    moves
}
