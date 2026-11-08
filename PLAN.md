# PLAN.md: Rust UCI Chess Engine (Windows)

Instructions for the coding agent. Read the whole file first, then work **phase by phase**.

## Goal

Build a fast, strong chess engine in **Rust** on **native Windows** that speaks the **UCI protocol**, so it can be:

1. Played locally in a chess GUI (Cute Chess, Arena, Lucas Chess)
2. Played online as a **Lichess bot** via `lichess-bot`

Target strength: ~2400-2800 Elo after Phase 5; 3000+ with optional NNUE (Phase 8).

---

## Environment (Windows)

- **OS / shell:** Windows 10/11, **PowerShell** (use PowerShell syntax for every command; no bash).
- **Editor:** VS Code with the `rust-analyzer` extension.
- **Toolchain:** Rust stable, target `x86_64-pc-windows-msvc`, with MSVC Build Tools installed.
- **Project path:** a short local path such as `C:\dev\engine`. **Do not** use OneDrive or other synced folders (slow builds, file-lock errors).
- **Binary:** `target\release\engine.exe`.

### Step 0: Verify the toolchain before anything else

Run and record the results in `STATUS.md`:

```powershell
rustc -vV          # host must be x86_64-pc-windows-msvc
cargo --version
where.exe link.exe # must print a path (MSVC linker)
```

If `link.exe` is missing or the host is `-gnu`, **stop** and tell the user. Do not try to patch around it with other linkers.

---

## Global Rules

1. **One phase at a time.** Do not start the next phase until the current phase's **acceptance test** passes.
2. **Commit after every phase** (and after every accepted feature in Phase 5) with a clear message.
3. **Tests first** where possible (perft, unit tests, tactics).
4. **Never merge a search or eval change without a strength test** (Phase 6). If SPRT fails, revert.
5. **One feature at a time.** Never bundle several search changes in one commit.
6. **One module per concern:** `board`, `eval`, `search`, `tt`, `ordering`, `uci`, `timeman`.
7. **Check crate docs** (`cozy-chess`) before using an API; do not guess method names.
8. Keep a `STATUS.md` log: current phase, what passed, known issues, latest bench NPS and Elo estimate.
9. If blocked after a few attempts, write the blocker into `STATUS.md` and ask the user.
10. **Windows rule:** never run the engine from `target\release` inside a GUI or match while rebuilding (Windows locks the `.exe`). Copy builds to `tools\bin\engine-<commit>.exe` and test those.

---

## Tech Stack

| Item | Choice |
|---|---|
| Language | Rust (stable, MSVC target) |
| Build tool | `cargo` |
| Move generation | `cozy-chess` crate (replaceable later with own bitboards) |
| Match testing | `fastchess` or `cutechess-cli` (Windows `.exe` release) |
| Online play | `lichess-bot` (GitHub, needs Python 3) |
| Reference opponent | Stockfish Windows `.exe` (`UCI_LimitStrength`, `UCI_Elo`) |

`Cargo.toml` release profile (required):

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
```

`.cargo/config.toml` (local fast builds; remove or change `target-cpu` for a binary you share):

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

Build:

```powershell
cargo build --release
```

---

## Project Structure

```
engine\
├── Cargo.toml
├── .cargo\config.toml
├── PLAN.md
├── STATUS.md
├── src\
│   ├── main.rs        # entry point, starts UCI loop
│   ├── uci.rs         # UCI parsing and output
│   ├── board.rs       # wrapper around cozy-chess, perft, repetition history
│   ├── eval.rs        # evaluation
│   ├── search.rs      # negamax, quiescence, pruning, iterative deepening
│   ├── tt.rs          # transposition table
│   ├── ordering.rs    # move ordering, killers, history
│   └── timeman.rs     # time allocation
├── tests\             # perft, tactics, draw detection
└── tools\
    ├── bin\           # saved engine builds for testing
    ├── books\         # opening book (EPD/PGN)
    └── *.ps1          # sprt, gauntlet, bench scripts
```

---

## Phase 0: Setup

1. Run the toolchain checks above and log them.
2. `cargo new engine`; add `cozy-chess`.
3. Add the release profile and `.cargo\config.toml`.
4. Create the module skeleton and `STATUS.md`.
5. `git init` with a `.gitignore` containing `target/` and `tools/bin/`.
6. **Optional but recommended:** add the project folder to **Windows Defender exclusions** (Defender can slow builds and match runs significantly). Ask the user before changing it; do not do it automatically.

**Acceptance:** `cargo build --release` succeeds and produces `target\release\engine.exe`.

---

## Phase 1: Board and Move Generation

1. Wrap the `cozy-chess` board; support FEN parsing and making moves.
2. Implement `perft(depth)` and a `perft divide` debug mode.
3. Add perft tests (`cargo test --release`).

| Position | Depth | Expected nodes |
|---|---|---|
| Start position | 5 | 4,865,609 |
| Start position | 6 | 119,060,324 |
| Kiwipete `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -` | 4 | 4,085,603 |

**Acceptance:** all perft numbers match **exactly**.

---

## Phase 2: UCI Protocol

1. Read commands line by line from stdin. **Trim `\r\n` and whitespace** (Windows GUIs send CRLF).
2. Handle: `uci`, `isready`, `ucinewgame`, `position [startpos | fen <fen>] [moves ...]`, `go`, `stop`, `quit`, `setoption`.
3. Output: `id name`, `id author`, `option` lines (Hash, Threads later), `uciok`, `readyok`, `bestmove`.
4. **Flush stdout after every line** written (a missing flush makes GUIs hang).
5. Support `go` parameters: `wtime`, `btime`, `winc`, `binc`, `movestogo`, `depth`, `nodes`, `movetime`, `infinite`.
6. Initially return a **random legal move**.
7. Run search in a **separate thread** with an atomic stop flag so `stop` and `isready` stay responsive.
8. Store the **position history** (hashes) from `position ... moves` for repetition detection.
9. Add a `bench` command (fixed positions, fixed depth) that prints total nodes and NPS.
10. Write only UCI output to stdout; send debug text to stderr.

**Acceptance:** the engine loads in **Cute Chess** (Windows) and plays a complete legal game without crashing or illegal moves.

---

## Phase 3: Evaluation

1. Material values: P=100, N=320, B=330, R=500, Q=900.
2. **Piece-square tables** with **tapered eval** (midgame/endgame blended by game phase). Start from the PeSTO tables.
3. Return scores from the **side-to-move perspective** (required for negamax).
4. Keep eval fast: no heap allocation, no repeated board scans where avoidable.

**Acceptance:**
- Start position eval is about 0.
- A color-mirrored position gives the same score for the other side.
- Unit tests cover both.

---

## Phase 4: Core Search

1. **Negamax with alpha-beta pruning.**
2. **Quiescence search** (captures, plus promotions); use stand-pat.
3. **Iterative deepening**; search the previous iteration's best move first.
4. **Mate scoring:** `MATE = 30000`; return `-MATE + ply` when mated so shorter mates are preferred.
5. **Draw detection:** threefold repetition (treat a single repetition inside the search as a draw), 50-move rule, insufficient material.
6. **Time management** (`timeman.rs`):
   - Allocate roughly `remaining / 25 + increment / 2`, with a hard cap well below remaining time.
   - Subtract a safety margin (e.g. 50 ms) for GUI and process overhead.
   - Check the clock and stop flag every ~2048 nodes, **including inside quiescence**.
   - Always have a legal best move ready; never lose on time.
7. Print `info depth .. score cp|mate .. nodes .. nps .. time .. pv ..` after each iteration.
8. Run the search on a thread with a larger stack (e.g. 16-64 MB) to avoid stack overflow at depth.

**Acceptance:**
- Solves a set of mate-in-2 and mate-in-3 puzzles.
- Beats a random mover 100-0.
- Never makes an illegal move or flags in a 100-game fast match.

---

## Phase 5: Strength Upgrades

Add **one at a time**, in this order. After each, run the Phase 6 SPRT; **keep only if it passes**, commit, then proceed.

| # | Feature | Implementation notes |
|---|---|---|
| 1 | **Transposition table** | Zobrist hash (use `cozy-chess` hash if available); fixed-size array sized by `Hash` option; store depth, score, bound type, best move; depth-preferred or age-aware replacement; **adjust mate scores by ply** when storing and probing; clear on `ucinewgame` |
| 2 | **Move ordering** | TT move, then captures by MVV-LVA, then killer moves (2 per ply), then history heuristic |
| 3 | **Principal Variation Search** | Full window for first move, null window for the rest, re-search on fail-high |
| 4 | **Null-move pruning** | R = 2-3; skip when in check, in pawn/king-only endings, or right after a null move |
| 5 | **Late move reductions** | Log-based reduction table; reduce late quiet moves; re-search at full depth if the result beats alpha |
| 6 | **Check extension** | Extend by 1 ply when in check |
| 7 | **Aspiration windows** | Narrow window around the previous iteration's score; widen on fail |
| 8 | **Futility and reverse futility pruning** | Shallow depths only; never when in check or near mate scores |
| 9 | **SEE** (static exchange evaluation) | Prune losing captures in quiescence and ordering |
| 10 | **Counter-move and continuation history** | Optional, once the above are stable |

---

## Phase 6: Testing and Strength Measurement

Set this up **before or alongside Phase 5**. All scripts are **PowerShell (`.ps1`)** in `tools\`.

### Tools to download (ask the user to confirm each download)

- **fastchess** (preferred) or **cutechess-cli**: Windows release `.exe` from GitHub; put in `tools\fastchess\`.
- **Stockfish**: Windows `.exe` (e.g. the `avx2` build) from stockfishchess.org; put in `tools\stockfish\`.
- **Opening book**: balanced EPD/PGN set; put in `tools\books\`.

### Procedure

1. Save each accepted build: `Copy-Item target\release\engine.exe tools\bin\engine-<commit>.exe`.
2. **SPRT** self-play, new build vs previous accepted build:
   - Time control about `10+0.1`, 1 thread, fixed Hash
   - Bounds about `[0, 5]` Elo, alpha = beta = 0.05
   - Concurrency = logical cores minus 2 (leave room for the OS)
3. **Accept** a change only if SPRT passes. Otherwise revert.
4. Gauge absolute strength vs Stockfish with `UCI_LimitStrength=true` and `UCI_Elo` set; log in `STATUS.md`.
5. Track **NPS** and total nodes with `bench` after every change. A large unexpected NPS drop is a regression.
6. Add a tactics suite (WAC-style EPD) as a sanity check only.
7. Use quoted paths in scripts (`"C:\dev\engine\..."`) and `$PSScriptRoot` for relative paths.

Provide scripts: `tools\sprt.ps1`, `tools\gauntlet.ps1`, `tools\bench.ps1`.

---

## Phase 7: Connect to Play

### Local GUI (Windows)

1. Build: `cargo build --release`.
2. Copy to `tools\bin\engine-play.exe` (avoids the file lock while you keep developing).
3. In **Cute Chess**: Tools > Settings > Engines > Add; set the command to that `.exe`, protocol **UCI**.
4. Alternatively add the same `.exe` as a UCI engine in **Arena** or **Lucas Chess**.
5. Play a game against it.

### Lichess bot

1. Create a **new** Lichess account (an account that has played games cannot be upgraded to BOT).
2. Create an API token with the **`bot:play`** scope.
3. Store the token in an environment variable, **never in git**:
   ```powershell
   $env:LICHESS_TOKEN = "<token>"
   ```
4. Upgrade the account (once):
   ```powershell
   curl.exe -d "" https://lichess.org/api/bot/account/upgrade -H "Authorization: Bearer $env:LICHESS_TOKEN"
   ```
5. Install **Python 3**, clone `lichess-bot`, create a virtual environment and install `requirements.txt`:
   ```powershell
   python -m venv venv
   .\venv\Scripts\Activate.ps1
   pip install -r requirements.txt
   ```
6. In `config.yml` set:
   - `token` (or load it from the environment variable as the README describes)
   - `engine.dir` (folder holding the `.exe`, use forward slashes or escaped backslashes)
   - `engine.name` (e.g. `engine-play.exe`)
   - `engine.protocol: "uci"`
   - supported time controls; standard chess only
7. Run `python lichess-bot.py`, then challenge the bot from your main account.

**Acceptance:** you can play a complete game against it (locally and/or on Lichess).

---

## Phase 8: Advanced (Optional)

1. **NNUE evaluation:** biggest strength jump. Train with `bullet`; implement incremental updates and SIMD inference.
2. **Lazy SMP:** multi-threaded search with a shared TT and a `Threads` option.
3. **Syzygy tablebases:** perfect endgame play (`SyzygyPath` option).
4. **Pondering** (`ponder` option, `go ponder`, `ponderhit`).
5. **Opening book:** small Polyglot book.
6. **Own bitboard move generator** with magic bitboards, replacing `cozy-chess`; verify with perft.

---

## Strength Milestones

| After phase | Approx. Elo |
|---|---|
| 4 | ~1800-2000 |
| 5 | ~2400-2800 |
| 8 (NNUE) | 3000+ |

---

## Common Pitfalls

### Chess logic
- Not clearing or aging the TT on `ucinewgame`
- Mixing up **side-to-move perspective** in eval, TT scores, or mate scores
- Storing **mate scores** in the TT without ply adjustment
- Not checking **time and stop** inside quiescence search
- Forgetting game history from `position ... moves`, causing missed repetitions
- Pruning (null-move, futility, LMR) while in check or near mate scores
- Adding several features at once, so regressions cannot be traced
- Testing only against a weak opponent; always run SPRT against the previous version

### Windows specific
- **CRLF input:** failing to trim `\r` breaks move parsing
- **Missing stdout flush:** GUI hangs waiting for `uciok` / `bestmove`
- **Locked `.exe`:** rebuilding while a GUI or match is using `target\release\engine.exe` fails; test from copies in `tools\bin`
- **Debug output on stdout:** corrupts the UCI stream; use stderr
- **Defender and OneDrive:** slow builds, slow matches, random file errors
- **Bash-only commands:** use PowerShell equivalents (`Copy-Item`, `$env:VAR`, `.\script.ps1`)
- **Spaces in paths:** quote them in scripts and GUI settings
- Committing API tokens

---

## Definition of Done (v1)

- [ ] Toolchain check logged (MSVC host, `link.exe` found)
- [ ] Perft matches all reference values
- [ ] Plays full games in Cute Chess with no illegal moves or time losses
- [ ] All Phase 5 features accepted via SPRT (or documented as rejected)
- [ ] `bench` command and NPS tracked in `STATUS.md`
- [ ] Runs as a Lichess bot on Windows
- [ ] `README.md` with Windows build, run, and configuration instructions
