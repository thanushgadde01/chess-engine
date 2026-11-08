# Engine Status

- Current phase: 5 - Strength Upgrades (Feature 1: Transposition Table complete)
- Acceptance: Phase 5.1 passed (TT with Zobrist hashing, Exact/Lower/Upper bounds, ply mate adjustments, `setoption name Hash`, node reduction from 40.1M -> 3.8M)
- Toolchain:
  - rustc: 1.99.1 (58cf8f9c3 2026-10-13)
  - host: x86_64-pc-windows-msvc
  - cargo: 1.99.1 (5f94df478 2026-08-27)
  - link.exe: `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe`
- Known issues: none
- Latest bench: 3,819,290 nodes @ 1.58 Mnps in 2.4s (10x faster search reduction via TT)
- Elo estimate: ~2000 (Alpha-beta + Quiescence + PeSTO tapered eval + TT)

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
  - Massive node count reduction on bench: 40,119,727 -> 3,819,290 nodes (bench time dropped from 21s -> 2.4s)

## Validation

- `cargo build --release`: Passed
- `cargo test --test tactics`: Passed
- `cargo test --test uci_test`: Passed
- Engine bench: 3.82M nodes in 2.41s (10.5x search efficiency speedup)
