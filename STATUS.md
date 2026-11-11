# Engine Status

- Current phase: Step 1 Complete (Move Representation & Castling Correctness Verified)
- Acceptance:
  - Cozy-chess king-takes-rook castling (e1h1/e1a1/e8h8/e8a8) bidirectional mapping with standard UCI castling notation (e1g1/e1c1/e8g8/e8c8).
  - Both directions verified: engine move -> UCI string and UCI string -> engine move.
  - Promotions suffix (e7e8q) and en passant verified.
  - Round-trip tested for every legal move across >200 positions.
  - Bestmove verified to always be legal in the position, even when stopped early.
  - Perft reference values match exactly (startpos depth 5 & 6, Kiwipete depth 4).
  - All 16 tests passing in `cargo test --release`.
- Baseline Metrics (Step 0):
  - Bench Total Nodes: 25,887 nodes
  - Bench Time: 59 ms (~438,762 nps recorded across 6 positions)
  - Perft startpos depth 5: 4,865,609 nodes (passed)
  - Perft startpos depth 6: 119,060,324 nodes (passed in 4.34s)
  - Perft kiwipete depth 4: 4,085,603 nodes (passed)
  - Baseline binary saved at `tools\bin\engine-baseline.exe`

## Completed Milestones

- **Phase 0:** Setup & MSVC native CPU toolchain verification (`target-cpu=native`).
- **Phase 1:** Board and Move Generation:
  - Cozy-chess bitboard wrapper with FEN parsing and repetition history
  - Perft test suite passing (startpos depth 5 & 6, Kiwipete depth 4, divide verification).
- **Phase 2:** UCI Protocol:
  - `uci`, `isready`, `ucinewgame`, `position`, `go`, `stop`, `bench`, `setoption`, `quit`.
  - Non-blocking background worker thread for search.
  - Integration test suite `tests/uci_test.rs` validating full handshake and moves.
- **Phase 3:** Evaluation:
  - PeSTO Piece-Square Tables with midgame/endgame tapered evaluation and game-phase interpolation.
  - Zero heap allocations during evaluation.
  - Symmetry and balance tests passing.
- **Phase 4:** Core Search:
  - Alpha-beta pruning negamax with iterative deepening.
  - Quiescence search with MVV-LVA move ordering and delta pruning.
  - Mate distance scoring.
  - Threefold repetition, 50-move rule, and insufficient material draw detection.
  - Adaptive time allocation (`timeman.rs`).
- **Phase 5:** Strength Upgrades:
  - **Transposition Table:** 16MB direct-mapped hash table with age and depth replacement, ply mate adjustments.
  - **Move Ordering:** TT best move -> MVV-LVA captures -> 2 killers per ply -> Counter-moves -> History heuristic.
  - **Principal Variation Search (PVS):** Null-window scouting on non-PV moves with fail-high re-searches.
  - **Null-Move Pruning (NMP):** Adaptive $R=2/3$ with zugzwang verification and checkers check.
  - **Late Move Reductions (LMR):** Reductions on quiet late moves with verification re-searches.
  - **Check Extensions:** +1 ply depth extension on evasion.
  - **Aspiration Windows:** Narrow window around previous iteration score.
  - **Futility & Reverse Futility Pruning (RFP):** Static eval distance pruning at shallow depths.
  - **Counter-move Heuristic:** `[color][prev_from][prev_to]` tracking.
- **Phase 6:** Testing & Measurement Tooling:
  - Automated PowerShell test scripts (`tools/bench.ps1`, `tools/sprt.ps1`, `tools/gauntlet.ps1`).
- **Phase 7:** Play Integration:
  - Standalone GUI-ready executable deployed to `tools\bin\engine-play.exe` (avoids file-lock issues during GUI play).
  - Complete `README.md` with instructions for Cute Chess, Arena, Lucas Chess, and `lichess-bot`.

## Validation

- `cargo build --release`: Passed
- `cargo test --release`: Passed (All 12 tests green)
- `tools\bench.ps1`: 25,887 nodes evaluated in 21 ms
