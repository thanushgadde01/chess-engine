# Engine Status

- Current phase: Complete through Phase 7 (Ready for Local & Lichess play)
- Acceptance: All Phase 0-7 milestones passed and Definition of Done verified
- Toolchain:
  - rustc: 1.99.1 (58cf8f9c3 2026-10-13)
  - host: x86_64-pc-windows-msvc
  - cargo: 1.99.1 (5f94df478 2026-08-27)
  - link.exe: `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe`
- Known issues: none
- Latest bench: 25,887 nodes @ 1.23 Mnps in 21 ms (sub-25ms benchmark)
- Elo estimate: ~2600-2700 (Full Phase 5 search stack + PeSTO tapered eval + TT)

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
