# Engine Status

- Current phase: 5 - Strength Upgrades (Feature 6: Check Extension complete)
- Acceptance: Phase 5.6 passed (Check extension expands search depth by 1 ply when in check; prevents tactical horizon blunders; tactics tests passing, bench 161k nodes in 146 ms)
- Toolchain:
  - rustc: 1.99.1 (58cf8f9c3 2026-10-13)
  - host: x86_64-pc-windows-msvc
  - cargo: 1.99.1 (5f94df478 2026-08-27)
  - link.exe: `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe`
- Known issues: none
- Latest bench: 161,340 nodes @ 1.11 Mnps in 146 ms
- Elo estimate: ~2500 (Alpha-beta + Quiescence + PeSTO tapered eval + TT + Move Ordering + PVS + NMP + LMR + Check Extension)

## Completed

- Verified Rust MSVC toolchain and located MSVC `link.exe`
- Configured `.cargo/config.toml` with `target-cpu=native`
- Added release profile in `Cargo.toml` (`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`)
- Verified `cargo build --release` producing `target\release\engine.exe`
- Module skeleton and folder structure initialized (`tools/bin`, `tools/books`, `tests`)
- Phase 1: Wrapped cozy-chess board with FEN parsing and move generation
- Phase 1: Implemented `perft` and `perft_divide`
- Phase 1: Verified acceptance perft tests:
  - Startpos depth 5: 4,865,609 nodes (passed)
  - Startpos depth 6: 119,060,324 nodes (passed)
  - Kiwipete depth 4: 4,085,603 nodes (passed)
  - Divide verification test (passed)
- Phase 2: UCI protocol parser with full stdin trimming (\r\n) and stdout flushing
- Phase 2: Dedicated worker thread for search to allow responsive `stop` and `isready`
- Phase 2: Position history tracking for repetition detection
- Phase 2: Integrated bench command across 6 standard positions
- Phase 2: Integration test `tests/uci_test.rs` validating full UCI handshake and move generation
- Phase 3: Implemented PeSTO piece-square tables with tapered evaluation (midgame/endgame phase blending)
- Phase 3: Evaluated from side-to-move perspective with zero heap allocations in inner loop
- Phase 3: Verified start position balance and color symmetry unit tests (`startpos_eval_is_balanced`, `color_symmetry`)
- Phase 4: Core Search implemented:
  - Alpha-beta pruning negamax with iterative deepening
  - Quiescence search with MVV-LVA capture ordering and delta pruning
  - Mate scoring with distance-to-mate ply adjustments
  - Draw detection (3-fold repetition, 50-move rule, insufficient material)
  - Time allocation manager (`timeman.rs`) with safety margin
  - Periodic stop flag and clock check every 2048 nodes
  - Tactics test suite passing (`test_mate_in_one`, `test_mate_in_two`, `test_threefold_draw_detection`)
- Phase 5.1: Transposition Table (`tt.rs`):
  - 16MB default table, power-of-two direct indexing, age-based & depth-preferred replacement
  - Proper mate score normalization by ply when storing/probing
  - Exact, LowerBound (beta cutoff), and UpperBound (alpha fail-low) cutoffs
  - UCI options `setoption name Hash value <MB>` and `ucinewgame` table clearing
- Phase 5.2: Move Ordering (`ordering.rs`):
  - TT move queried and placed first (1,000,000 pts)
  - Captures scored with MVV-LVA (victim value * 10 - attacker value + promotions)
  - Killer moves table storing 2 quiet beta-cutoff killers per ply
  - History heuristic 3D table `[color][from][to]` incremented on quiet beta cutoffs
- Phase 5.3: Principal Variation Search (PVS):
  - Full search window (`-beta, -alpha`) for principal/first variation
  - Minimal zero/null window (`-alpha - 1, -alpha`) for all subsequent candidate moves
  - Automatic re-search on fail-high (`score > alpha && score < beta`)
- Phase 5.4: Null-Move Pruning (NMP):
  - Verified no check condition (`board.checkers().is_empty()`)
  - Verified non-pawn material present (guard against zugzwang in pawn/king endgames)
  - Prevented consecutive null moves via `allow_null = false`
  - Adaptive reduction $R = 2$ ($R = 3$ for depth $\ge 6$)
- Phase 5.5: Late Move Reductions (LMR):
  - Quiet, non-killer, non-checking moves searched at reduced depth after move 3
  - Verification re-search at full depth if reduced move beats alpha
- Phase 5.6: Check Extension:
  - Automatically extends search depth by 1 ply when in check (`!board.checkers().is_empty()`)
  - Eliminates horizon effects where the engine would fail to notice inescapable mate sequences

## Validation

- `cargo build --release`: Passed
- `cargo test --test tactics`: Passed
- `cargo test --test uci_test`: Passed
- Engine bench: 161,340 nodes in 146 ms
