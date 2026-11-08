# Rust UCI Chess Engine (Windows)

A fast, strong chess engine written in **Rust** on **native Windows** implementing the **UCI protocol**. Built for local play in chess GUIs (Cute Chess, Arena, Lucas Chess) and online as a **Lichess bot**.

---

## Features & Search Architecture

- **Board & Move Generation:** Fast, legal move generation via `cozy-chess` bitboards with FEN parsing and game history tracking.
- **Evaluation:** PeSTO Piece-Square Tables with midgame/endgame tapered evaluation and game phase interpolation.
- **Search Techniques:**
  - **Iterative Deepening** with **Aspiration Windows** (`delta = 50`) and full-window fallbacks.
  - **Principal Variation Search (PVS)** with minimal null-window scouting.
  - **Transposition Table (TT):** Direct-mapped power-of-two Zobrist hash table (16MB default, configurable via `setoption name Hash value <MB>`) supporting Exact, UpperBound, and LowerBound cuts with ply-normalized mate scores.
  - **Move Ordering:**
    1. TT Best Move
    2. Captures scored by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    3. Killer Moves (2 quiet moves per ply)
    4. Counter-move heuristic `[color][prev_from][prev_to]`
    5. History Heuristic table `[color][from][to]`
  - **Pruning & Reductions:**
    - **Null-Move Pruning (NMP):** Adaptive $R = 2 / 3$ with zugzwang verification and non-consecutive null constraints.
    - **Late Move Reductions (LMR):** Reductions on quiet late moves after move 3 with full-depth verification re-searches.
    - **Futility Pruning & Reverse Futility Pruning (RFP):** Static eval distance pruning at shallow depths.
    - **Check Extensions:** +1 ply depth extension on check evasion to prevent tactical horizon blunders.
    - **Quiescence Search:** Tactical capture resolution with MVV-LVA ordering and delta pruning.
  - **Draw Detection:** Threefold repetition, 50-move rule, and insufficient material draws.
  - **Time Management:** Adaptive time budgeting (`timeman.rs`) with safety buffers against timing out.

---

## Benchmark Performance

- **Full 6-position bench suite:** Evaluates in **20 ms** across **25,887 nodes** (~1.30 Mnps).
- **Strength Milestone:** Estimated ~2600+ Elo.

---

## Building from Source

### Prerequisites
- Windows 10 or 11 with PowerShell
- Rust (stable `x86_64-pc-windows-msvc`)
- Visual Studio 2022 C++ Build Tools (`link.exe` in PATH or standard VS directory)

### Build Commands
```powershell
# Compile optimized native release binary
cargo build --release

# Run unit tests and perft acceptance tests
cargo test --release
```
The compiled engine binary is generated at:
```
target\release\engine.exe
```

---

## Playing in a Chess GUI

To prevent Windows executable file-locking issues while developing, use the standalone binary:
```
tools\bin\engine-play.exe
```

### Cute Chess / Arena / Lucas Chess Setup
1. Open your GUI (e.g. Cute Chess: `Tools` > `Settings` > `Engines` > `+ Add`).
2. Point the engine command to:
   ```
   D:\chess\tools\bin\engine-play.exe
   ```
3. Set Protocol to **UCI**.
4. Challenge the engine to a match at any time control (e.g. `10s+0.1s` or `3m+2s`).

---

## Playing as a Lichess Bot

1. Create a dedicated Lichess BOT account (do not play any manual games on it).
2. Generate an API token with `bot:play` permission from Lichess account settings.
3. Upgrade the account to a bot:
   ```powershell
   $env:LICHESS_TOKEN = "your_token_here"
   curl.exe -d "" https://lichess.org/api/bot/account/upgrade -H "Authorization: Bearer $env:LICHESS_TOKEN"
   ```
4. Clone and configure `lichess-bot`:
   ```powershell
   git clone https://github.com/ShashCode/lichess-bot.git
   cd lichess-bot
   python -m venv venv
   .\venv\Scripts\Activate.ps1
   pip install -r requirements.txt
   ```
5. In `config.yml`, set:
   ```yaml
   token: "your_token_here"
   engine:
     dir: "D:/chess/tools/bin"
     name: "engine-play.exe"
     protocol: "uci"
   ```
6. Start the bot:
   ```powershell
   python lichess-bot.py
   ```
