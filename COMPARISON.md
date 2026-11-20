# COMPARISON.md: Reference Architecture Analysis & Feature Comparison

## Reference Repositories & Licensing Notice

1. **shakmaty** (https://github.com/niklasf/shakmaty)
   - **License:** **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.
   - **shakmaty-syzygy License:** **GPL-3.0-or-later**.
   - **Usage Notice:** Read-only for architectural ideas and protocol specifications. No code has been or will be copied, transcribed, or integrated into our codebase. All implementations in our engine are written completely from scratch in our own idiomatic Rust style.

2. **timecat-rs** (https://github.com/Gourab-Ghosh/timecat-rs)
   - **License:** **GNU General Public License v3.0 (GPL-3.0)**.
   - **Usage Notice:** Read-only for engine design patterns and heuristic ideas. No code has been or will be copied or transcribed. All algorithms are implemented independently.

---

## Detailed Technique Comparison Table

| Technique / Area | Our Engine (`src/`) | Timecat-rs (`C:\dev\reference\timecat-rs`) | Idea Worth Testing | Expected Gain | Implementation Risk |
|---|---|---|---|---|---|
| **Principal Variation Search (PVS)** | Full window on PV move, null-window `[-alpha-1, -alpha]` on non-PV moves, re-search on fail-high. | Similar PVS structure; uses a dedicated `PVTable` per thread for explicit triangular PV tracking. | Triangular PV table or TT-driven PV extraction to avoid leaf probe overheads and clean up UCI `pv` output. | Small (cleanliness / stability +5 Elo) | Low |
| **Aspiration Windows** | Symmetrical window $\pm 50$ cp around previous depth score; expands $\times 2$ on fail-low/high up to 1000, then fallback to $[-\infty, \infty]$. | Asymmetric / dynamic window widening; returns immediately when outside window to initiate re-search. | Asymmetric widening: narrow on fail-high, wider on fail-low. | Medium (+10-15 Elo in endgames) | Low |
| **Null-Move Pruning (NMP)** | Adaptive $R=2$ ($R=3$ for depth $\ge 6$); requires non-pawn material; skips if in check or consecutive null. | Adaptive $R = 1920 + \text{depth} \times 2368 / 4096 \approx 1.5 + 0.58 \times \text{depth}$; verification search at shallow depth. | Dynamic formula for $R$ scaling smoothly with depth instead of a discrete step at depth 6. | Medium (+15-20 Elo) | Low |
| **Late Move Reductions (LMR)** | Discrete step reduction ($R=1$ for move $\ge 3$ and depth $\ge 3$; $R=2$ for move $\ge 6$ and depth $\ge 6$); re-search at full depth if $> \alpha$. | Logarithmic formula: $R = \text{base} + \frac{\ln(\text{depth}) \times \ln(\text{move\_index})}{\text{divider}}$; PV nodes get reduced reduction ($R \times 2/3$). | Logarithmic LMR table: precalculated 2D array $[depth][move\_index]$ scaling continuously, with smaller reduction for tactical / PV nodes. | High (+30-50 Elo) | Medium |
| **Futility & Reverse Futility Pruning (RFP)** | RFP: static eval margin $120 \times \text{depth}$ for depth $\le 3$. Futility: margin $150 \times \text{depth}$ for depth $\le 2$. | RFP margin $((6 \times \text{Pawn}) / 5) \times \text{depth} = 120 \times \text{depth}$ for depth $< 3$. Razoring at depth $\le 3$ with quiescence verify. | Add Razoring at depth 1-3 (if static eval is far below beta, verify with quiescence before doing full search). | Medium (+15-25 Elo) | Medium |
| **Search Extensions** | Check extension: $+1$ ply if in check. | Check extension $+1$ ply; singular extensions candidate. | Keep check extension capped to avoid tactical explosion in deep endgame searches. | Small (+5 Elo) | Low |
| **Static Exchange Evaluation (SEE)** | Not implemented yet (captures ordered by pure MVV-LVA; delta pruning only in QS). | Recursive least-attacker `see(square)` and `see_capture(square)`; prunes losing captures in QS (`weight.is_negative()`). | Implement `see(board, move)` to prune losing captures in QS (`see < 0`) and demote bad captures in move ordering below killers. | **Very High (+50-80 Elo)** | Low-Medium |
| **Quiescence Search (QS)** | Generates all pseudo-legal moves then filters captures and promotions on a stack buffer; delta pruning with $+200$ margin; depth limit 8. | Generates weighted capture moves sorted with SEE; delta pruning with queen value + promotion delta. | **Capture-only move generation** directly in cozy-chess/board wrapper, skipping all quiet moves up front; SEE pruning. | **High (+40-60% QS speedup, +25 Elo)** | Low |
| **Move Ordering** | TT move $\to$ MVV-LVA $\to$ 2 Killers per ply $\to$ Countermoves $\to$ History heuristic. | TT move $\to$ Good captures (positive SEE) $\to$ Killers $\to$ Countermoves $\to$ History $\to$ Bad captures (negative SEE). | Differentiate winning/equal captures from losing captures using SEE; order losing captures after quiet killer/history moves. | **High (+35-50 Elo)** | Low |
| **Continuation History** | Not implemented. | Tracks history keyed by piece and destination, with decay / dampening. | Continuation history table: $[piece][to]$ indexed by the previous move. | Medium (+15-20 Elo) | Low |
| **Transposition Table (TT)** | 16 MB direct-mapped array; stores Zobrist hash, depth, score (mate adjusted), bound, best_move, age. Replaces if deeper or older. | Custom `CacheTable` with non-zero hash tags and power-of-two mask indexing; configurable size. | Dynamic TT sizing via UCI `Hash` option (e.g. 1 MB to 1024 MB); bucketed 2-way or 4-way replacement. | Medium (+20 Elo with larger hash) | Low |
| **Time Management** | Soft limit $\approx \text{remaining}/25 + \text{inc}/2$; hard limit $\approx 2.5 \times \text{soft}$; periodic check every 2048 nodes in negamax & QS. | Dynamic time divider scaling with move number ($(20 - \text{fullmove}/2).\text{max}(5)$); advantage bonus based on clock delta. | Scale move divider adaptively with game phase and move number; dynamic time extension if root score fluctuates. | Medium (+15-25 Elo in rapid/blitz) | Low |
| **Evaluation: PSQT vs NNUE** | Tapered PeSTO Piece-Square Tables (middlegame/endgame interpolation via non-pawn material phase); zero allocation. | Dual evaluation support: HalfKP NNUE (`evaluate_nnue.rs`) and classical fallback (`evaluate_non_nnue.rs`). | 1. Incrementally update PSQT score in make/unmake to save ~30% eval time.<br>2. Optional NNUE (Phase 8). | **Very High (+30-40% NPS for incremental PSQT; +200-300 Elo for NNUE in Phase 8)** | Low for incremental PSQT; High for NNUE |
| **Allocators & Unsafe Speed** | Pure safe Rust; zero heap allocations in search/eval using fixed-size stack arrays (`[Move; 256]`, `[(Move, i32); 64]`). | `unsafe push_unchecked`, custom allocator hooks (`allocator.rs`), jemalloc support. | Keep memory safe stack buffers; optimize compiler intrinsics via `target-cpu=native`. | Small (+5-10% NPS) | Low |
| **UCI Protocol & Castling (shakmaty)** | Native UCI e1g1 castling converter; handles standard UCI commands (`isready`, `ucinewgame`, `position`, `go`, `stop`, `bench`, `quit`). | Full UCI command loop with custom colorized CLI output, options parser, and thread pool controller. | Strict UCI option registration (`Hash`, `Clear Hash`, `Threads`), robust FEN & move list parser. | Cleanliness / GUI compliance | Low |
| **Endgame Tablebases (shakmaty-syzygy)** | Not implemented (material draws detected: KK, KNK, KBK). | Integrated Syzygy tablebase probing (`shakmaty-syzygy` wrapper in `syzygy.rs`). | Syzygy 3-4-5 piece WDL/DTZ probing at root and search leaves (Phase 7+). Note: `shakmaty-syzygy` is GPL-3.0; independent probing or optional feature flag. | High in endgames (+30-50 Elo in endings) | Medium |

---

## Top Ideas Ranked by Expected Gain vs Risk

1. **SEE (Static Exchange Evaluation) for Captures & Move Ordering** (Expected Gain: +50-80 Elo, Risk: Low)
   - **Result: ACCEPTED.** Match vs previous accepted build: 42W / 36L / 22D (53.0%), **+20.9 Elo**. Promoted to accepted baseline.
2. **Capture-Only Move Generation in Quiescence** (Expected Gain: +40% QS speedup, Risk: Low)
   - **Result: REVERTED.** Tested 100 games vs accepted SEE: 20W / 24L / 56D (48.0%), **-13.9 Elo**. Pre-filtering `piece_moves.to &= capture_targets` introduced movegen overhead per callback that slightly slowed down nodes compared to direct cozy-chess iteration. Reverted per Rule 4 ("If SPRT fails, revert").
3. **Logarithmic / Smooth Late Move Reductions (LMR)** (Expected Gain: +30-40 Elo, Risk: Low-Medium)
   - **Result: REVERTED.** Tested 100 games vs accepted baseline: 11W / 40L / 49D (35.5%), **-103.7 Elo**. The deeper reductions caused tactical blunders on quiet moves without history-based reduction adjustments. Reverted per Rule 4 to preserve engine strength.
4. **Incremental PSQT & Phase Updates in make/unmake** (Expected Gain: +30-40% NPS, Risk: Low)
5. **Configurable UCI `Hash` Option & TT Bucketing** (Expected Gain: +20-30 Elo, Risk: Low)
   - **Result: ACCEPTED.** Added standard UCI `option name Hash type spin default 16 min 1 max 1024` and `option name Clear Hash type button`. Verified dynamic resizing, table clearing, and UCI protocol compatibility across test harness.
