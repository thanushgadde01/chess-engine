use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cozy_chess::{Board, GameStatus, Move, Piece};

use crate::board::Position;
use crate::eval::evaluate;
use crate::ordering::MoveOrdering;
use crate::timeman::TimeManager;
use crate::tt::{Bound, TranspositionTable};

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
    pub soft_time_limit_ms: Option<u64>,
    pub hard_time_limit_ms: Option<u64>,
    pub max_nodes: Option<u64>,
    pub position_history: &'a [u64],
}

impl<'a> SearchContext<'a> {
    pub fn new(
        stop_flag: Arc<AtomicBool>,
        soft_time_limit_ms: Option<u64>,
        hard_time_limit_ms: Option<u64>,
        max_nodes: Option<u64>,
        position_history: &'a [u64],
    ) -> Self {
        Self {
            nodes: 0,
            stop_flag,
            start_time: Instant::now(),
            soft_time_limit_ms,
            hard_time_limit_ms,
            max_nodes,
            position_history,
        }
    }

    #[inline(always)]
    pub fn should_stop(&mut self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        // Check periodic clock and stop flag every 2048 nodes
        if (self.nodes & 2047) == 0 {
            if let Some(hard_limit) = self.hard_time_limit_ms {
                if self.start_time.elapsed().as_millis() as u64 >= hard_limit {
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

    #[inline(always)]
    pub fn past_soft_limit(&self) -> bool {
        if let Some(soft_limit) = self.soft_time_limit_ms {
            self.start_time.elapsed().as_millis() as u64 >= soft_limit
        } else {
            false
        }
    }
}

pub fn search(
    position: &Position,
    limits: SearchLimits,
    stop_flag: Arc<AtomicBool>,
    tt: Arc<Mutex<TranspositionTable>>,
) -> (Option<Move>, u64) {
    let board = position.board();
    let moves = legal_moves(board);
    if moves.is_empty() {
        return (None, 0);
    }

    // Determine time allocation
    let (soft_limit, hard_limit) = if let Some(mt) = limits.movetime {
        let hard = mt.saturating_sub(10).max(1);
        (Some(hard), Some(hard))
    } else {
        let (my_time, my_inc) = match board.side_to_move() {
            cozy_chess::Color::White => (limits.wtime, limits.winc),
            cozy_chess::Color::Black => (limits.btime, limits.binc),
        };
        if let Some(alloc) = TimeManager::allocate(my_time, my_inc, limits.movestogo) {
            (Some(alloc.optimum_time_ms), Some(alloc.max_time_ms))
        } else {
            (None, None)
        }
    };

    let max_depth = limits
        .depth
        .unwrap_or(if limits.infinite || hard_limit.is_some() {
            64
        } else {
            4
        });
    let mut ctx = SearchContext::new(
        stop_flag,
        soft_limit,
        hard_limit,
        limits.nodes,
        position.history(),
    );

    let mut best_move = Some(moves[0]);
    let mut history_stack = Vec::with_capacity(64);
    let mut ordering = MoveOrdering::new();
    let mut prev_score: i32 = 0;

    // Iterative deepening
    for d in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        let mut current_best = None;
        let mut best_score = -MATE;

        // Aspiration Windows for depth >= 5
        let mut delta: i32 = 50;
        let orig_alpha = if d >= 5 {
            prev_score.saturating_sub(delta).max(-MATE)
        } else {
            -MATE
        };
        let orig_beta = if d >= 5 {
            prev_score.saturating_add(delta).min(MATE)
        } else {
            MATE
        };
        let mut alpha = orig_alpha;
        let mut beta = orig_beta;

        loop {
            if ctx.should_stop() {
                break;
            }

            best_score = -MATE;
            current_best = None;

            // Probe TT at root
            let tt_move = {
                let table = tt.lock().unwrap();
                table
                    .probe(board.hash(), 0)
                    .and_then(|(entry, _)| entry.best_move)
            };

            // Order moves using MoveOrdering (TT move, MVV-LVA captures, Killers, History)
            let mut ordered_moves = moves.clone();
            let candidate_first = tt_move.or(best_move);
            ordering.order_moves(board, &mut ordered_moves, candidate_first, None, 0);

            for &mv in &ordered_moves {
                let mut next = board.clone();
                next.play(mv);
                ctx.nodes += 1;

                history_stack.push(next.hash());
                let score = -negamax(
                    &next,
                    d - 1,
                    1,
                    -beta,
                    -alpha,
                    &mut ctx,
                    &mut history_stack,
                    &tt,
                    &mut ordering,
                    Some(mv),
                    true,
                );
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

            if ctx.stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Check if score fell outside aspiration window
            if d >= 5 {
                if best_score <= orig_alpha {
                    // Fail-low: expand alpha downward, reset beta
                    delta = delta.saturating_mul(2);
                    alpha = prev_score.saturating_sub(delta).max(-MATE);
                    beta = prev_score.saturating_add(delta).min(MATE);
                } else if best_score >= orig_beta {
                    // Fail-high: expand beta upward, reset alpha
                    delta = delta.saturating_mul(2);
                    alpha = prev_score.saturating_sub(delta).max(-MATE);
                    beta = prev_score.saturating_add(delta).min(MATE);
                } else {
                    // Score is cleanly inside window
                    break;
                }

                if delta >= 1000 {
                    alpha = -MATE;
                    beta = MATE;
                }
            } else {
                break;
            }
        }

        if !ctx.stop_flag.load(Ordering::Relaxed) && current_best.is_some() {
            best_move = current_best;
            prev_score = best_score;
            // Store root evaluation into TT
            {
                let mut table = tt.lock().unwrap();
                table.store(board.hash(), d, best_score, Bound::Exact, best_move, 0);
            }
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
                crate::movegen::move_to_uci(board, best_move.unwrap())
            );

            // If we found a forced mate score at root, we can conclude search
            if best_score.abs() > MATE_BOUND && d >= ((MATE - best_score.abs()) as u8) {
                break;
            }

            // Stop starting a new depth if we reached the soft time limit
            if ctx.past_soft_limit() {
                break;
            }
        } else {
            // Interrupted mid-depth: break out and return best_move from last COMPLETED depth!
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
    if search_history.len() > 1 && search_history[..search_history.len() - 1].contains(&hash) {
        return true;
    }
    if ctx.position_history.iter().filter(|&&h| h == hash).count() >= 2 {
        return true;
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn negamax(
    board: &Board,
    depth: u8,
    ply: usize,
    mut alpha: i32,
    mut beta: i32,
    ctx: &mut SearchContext,
    search_history: &mut Vec<u64>,
    tt: &Arc<Mutex<TranspositionTable>>,
    ordering: &mut MoveOrdering,
    prev_move: Option<Move>,
    allow_null: bool,
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

    // Check extension: extend depth by 1 if in check to avoid tactical horizon effects
    let in_check = !board.checkers().is_empty();
    let depth = if in_check {
        depth.saturating_add(1)
    } else {
        depth
    };

    // Transposition Table probe
    let orig_alpha = alpha;
    let hash = board.hash();
    let mut tt_move = None;

    if let Ok(table) = tt.try_lock() {
        if let Some((entry, Some(score))) = table.probe(hash, ply) {
            tt_move = entry.best_move;
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return score,
                    Bound::LowerBound => alpha = alpha.max(score),
                    Bound::UpperBound => beta = beta.min(score),
                }
                if alpha >= beta {
                    return score;
                }
            }
        }
    }

    // Static evaluation for pruning heuristics
    let static_eval = evaluate(board);

    // Reverse Futility Pruning (RFP / Static Null Move Pruning)
    // If static evaluation is well above beta in shallow non-check nodes, prune immediately
    if !in_check && depth <= 3 && static_eval.abs() < MATE_BOUND {
        let rfp_margin = 120 * depth as i32;
        if static_eval - rfp_margin >= beta {
            return static_eval - rfp_margin;
        }
    }

    // Null-Move Pruning (NMP)
    // Conditions:
    // - allow_null is true (never consecutive null moves)
    // - depth >= 3
    // - not in check
    // - side-to-move has non-pawn material (avoid zugzwang in endgame)
    if allow_null && depth >= 3 && board.checkers().is_empty() {
        let us = board.side_to_move();
        let my_pieces = board.colors(us);
        let my_pawns = board.pieces(Piece::Pawn);
        let my_kings = board.pieces(Piece::King);
        let has_non_pawn = !(my_pieces & !(my_pawns | my_kings)).is_empty();

        if has_non_pawn {
            if let Some(null_board) = board.null_move() {
                let r = 2 + (depth >= 6) as u8; // R = 2 (or 3 for deeper depths)
                let reduced_depth = depth.saturating_sub(r + 1);
                let null_score = -negamax(
                    &null_board,
                    reduced_depth,
                    ply + 1,
                    -beta,
                    -beta + 1,
                    ctx,
                    search_history,
                    tt,
                    ordering,
                    None,
                    false, // Do not allow consecutive null moves
                );

                if null_score >= beta {
                    return beta;
                }
            }
        }
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

    // Order moves: TT move, MVV-LVA, killers, countermoves, history
    let mut ordered_moves = moves;
    ordering.order_moves(board, &mut ordered_moves, tt_move, prev_move, ply);

    let mut best = -MATE;
    let mut best_move = None;
    let mut moves_searched = 0;

    for mv in ordered_moves {
        let mut next = board.clone();
        next.play(mv);
        ctx.nodes += 1;

        search_history.push(next.hash());

        // Determine move properties for LMR / Futility
        let enemy = !board.side_to_move();
        let is_capture = board.colors(enemy).has(mv.to);
        let gives_check = !next.checkers().is_empty();
        let is_promotion = mv.promotion.is_some();
        let is_killer = ordering.killers[ply.min(crate::ordering::MAX_PLY - 1)].contains(&Some(mv));

        // Futility Pruning: skip quiet moves that have no chance of raising alpha at shallow depths
        if !in_check
            && depth <= 2
            && moves_searched > 0
            && !is_capture
            && !is_promotion
            && !gives_check
            && !is_killer
        {
            let futility_margin = 150 * depth as i32;
            if static_eval + futility_margin <= alpha {
                moves_searched += 1;
                continue;
            }
        }

        // Principal Variation Search (PVS) with Late Move Reductions (LMR)
        let score = if moves_searched == 0 {
            // First move: full window, full depth
            -negamax(
                &next,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                ctx,
                search_history,
                tt,
                ordering,
                Some(mv),
                true,
            )
        } else {
            // Late Move Reductions (LMR) for quiet, late moves
            let mut reduction = 0;
            if depth >= 3
                && moves_searched >= 3
                && !is_capture
                && !is_promotion
                && !gives_check
                && !is_killer
            {
                reduction = 1;
                if depth >= 6 && moves_searched >= 6 {
                    reduction = 2;
                }
            }

            let search_depth = (depth - 1).saturating_sub(reduction);

            let mut s = -negamax(
                &next,
                search_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                ctx,
                search_history,
                tt,
                ordering,
                Some(mv),
                true,
            );

            // If reduced search fails high, re-search at full depth with null window
            if reduction > 0 && s > alpha {
                s = -negamax(
                    &next,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    ctx,
                    search_history,
                    tt,
                    ordering,
                    Some(mv),
                    true,
                );
            }

            // Re-search with full window on fail-high
            if s > alpha && s < beta {
                s = -negamax(
                    &next,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    ctx,
                    search_history,
                    tt,
                    ordering,
                    Some(mv),
                    true,
                );
            }
            s
        };

        search_history.pop();
        moves_searched += 1;

        if ctx.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        if score > best {
            best = score;
            best_move = Some(mv);
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            // Beta cutoff: update killer, countermove, and history if quiet move
            let enemy = !board.side_to_move();
            let is_capture = board.colors(enemy).has(mv.to);
            if !is_capture && mv.promotion.is_none() {
                ordering.record_killer(mv, ply);
                ordering.record_history(board, mv, depth);
                ordering.record_countermove(board, prev_move, mv);
            }
            break;
        }
    }

    // Store in Transposition Table
    let bound = if best <= orig_alpha {
        Bound::UpperBound
    } else if best >= beta {
        Bound::LowerBound
    } else {
        Bound::Exact
    };

    if let Ok(mut table) = tt.try_lock() {
        table.store(hash, depth, best, bound, best_move, ply);
    }

    best
}

// Quiescence search evaluates tactical captures until a quiet position is reached
fn quiescence(
    board: &Board,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchContext,
    q_depth: u8,
) -> i32 {
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

    // Generate only captures / promotions using a fixed-size stack buffer
    let mut captures = [(
        Move {
            from: cozy_chess::Square::A1,
            to: cozy_chess::Square::A1,
            promotion: None,
        },
        0i32,
    ); 64];
    let mut num_captures = 0;
    let enemy = !board.side_to_move();
    let enemy_occupied = board.colors(enemy);

    board.generate_moves(|piece_moves| {
        for mv in piece_moves {
            if num_captures >= 64 {
                break;
            }
            // Delta pruning: if stand_pat + captured piece max value + margin < alpha, skip
            let target_piece = board.piece_on(mv.to);
            let is_ep = match board.en_passant() {
                Some(ep_file) => {
                    board.pieces(Piece::Pawn).has(mv.from)
                        && mv.to.file() == ep_file
                        && (mv.to.rank() == cozy_chess::Rank::Third
                            || mv.to.rank() == cozy_chess::Rank::Sixth)
                }
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
                    captures[num_captures] = (mv, gain);
                    num_captures += 1;
                }
            }
        }
        false
    });

    // MVV-LVA move ordering in quiescence: sort captures by captured piece value desc
    captures[..num_captures].sort_unstable_by_key(|&(_, gain)| -gain);

    for &(mv, _) in captures.iter().take(num_captures) {
        // SEE Pruning in Quiescence Search:
        // Skip losing captures unless it's a promotion or tactical check evasion
        if mv.promotion.is_none() && crate::see::see(board, mv) < 0 {
            continue;
        }

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
