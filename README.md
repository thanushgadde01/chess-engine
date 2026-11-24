<div align="center">

# ♟️ Rust Chess Engine

### A fast, native Windows chess engine written in Rust and powered by the UCI protocol.

[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D4?logo=windows&logoColor=white)](https://www.microsoft.com/windows)
[![Protocol](https://img.shields.io/badge/Protocol-UCI-2ea44f)](https://www.wbec-ridderkerk.nl/html/UCIProtocol.html)
[![Stars](https://img.shields.io/github/stars/thanushgadde01/chess-engine?style=flat&logo=github)](https://github.com/thanushgadde01/chess-engine/stargazers)
[![Last Commit](https://img.shields.io/github/last-commit/thanushgadde01/chess-engine?logo=git)](https://github.com/thanushgadde01/chess-engine/commits/master)

[🚀 Features](#-features) · [📊 Performance](#-performance) · [🛠️ Build](#️-build-from-source) · [🎮 Use in a GUI](#-use-with-a-chess-gui) · [🤖 Lichess Bot](#-run-as-a-lichess-bot) · [🧭 Roadmap](#-roadmap)

</div>

---

## 📖 Overview

This project is a high-performance chess engine designed for local analysis, engine matches, and online play through a Lichess bot. It implements the **Universal Chess Interface (UCI)** and can be connected to popular chess interfaces such as [Cute Chess](https://cutechess.com/), [Arena](http://www.playwitharena.de/), and [Lucas Chess](https://lucaschess.pythonanywhere.com/).

> **Status:** Active development · Native Windows build · Rust stable

## ✨ Features

- **Fast board and move generation** using `cozy-chess` bitboards, FEN parsing, legal move generation, and game-history tracking.
- **PeSTO evaluation** with tapered midgame/endgame piece-square tables and game-phase interpolation.
- **Modern alpha-beta search** with iterative deepening, aspiration windows, and Principal Variation Search (PVS).
- **Transposition table** using a configurable power-of-two Zobrist hash table with exact, upper-bound, and lower-bound entries.
- **Strong move ordering** using TT moves, MVV-LVA captures, killer moves, counter-moves, and history heuristics.
- **Search pruning and reductions** including null-move pruning, late-move reductions, futility pruning, reverse futility pruning, razoring, and mate-distance pruning.
- **Tactical stability** through check extensions, quiescence search, and static exchange evaluation.
- **Draw detection** for threefold repetition, the 50-move rule, and insufficient material.
- **Adaptive time management** with safety buffers to reduce time forfeits.
- **UCI compatibility** for use with chess GUIs, scripts, and automated matches.

<details>
<summary>🔍 Search architecture</summary>

| Area | Techniques |
| --- | --- |
| Search | Iterative deepening, aspiration windows, PVS |
| Pruning | Null-move pruning, futility pruning, reverse futility pruning, razoring |
| Reductions | Late-move reductions with full-depth verification searches |
| Ordering | TT move, MVV-LVA, killer moves, counter-move, history heuristic |
| Evaluation | PeSTO piece-square tables, tapered evaluation, game phase |
| Stability | Quiescence search, check extensions, static exchange evaluation |
| State | Zobrist hashing, repetition history, mate-distance normalization |

</details>

## 📊 Performance

The included six-position benchmark currently reports:

- **25,887 nodes** in approximately **20 ms**
- Approximately **1.30 million nodes per second**
- An estimated strength milestone of **2600+ Elo** *(benchmark estimate; not an official rating)*

Run the benchmark with the project tooling:

```powershell
.\tools\bench.ps1
```

## 🛠️ Build from Source

### Requirements

- Windows 10 or Windows 11
- [Rust stable](https://www.rust-lang.org/tools/install) with the `x86_64-pc-windows-msvc` target
- Visual Studio 2022 C++ Build Tools, including `link.exe`

### Compile and test

```powershell
# Clone the repository
git clone https://github.com/thanushgadde01/chess-engine.git
cd chess-engine

# Build the optimized release binary
cargo build --release

# Run unit, UCI, tactical, timing, and perft tests
cargo test --release
```

The release executable is generated at:

```text
target\release\engine.exe
```

## 🎮 Use with a Chess GUI

For development, use the standalone binary to avoid Windows executable file-locking issues:

```text
tools\bin\engine-play.exe
```

### Cute Chess, Arena, or Lucas Chess

1. Open your chess GUI's engine management screen.
2. Add the engine executable, for example `tools\bin\engine-play.exe`.
3. Select **UCI** as the communication protocol.
4. Start an analysis session or engine match.

The engine accepts standard UCI options, including the transposition-table size:

```text
setoption name Hash value 256
```

## 🤖 Run as a Lichess Bot

> Create a dedicated Lichess BOT account. Do not play manual games on a bot account.

1. Create a bot account and generate an API token with the `bot:play` permission.
2. Upgrade the account using PowerShell:

   ```powershell
   $env:LICHESS_TOKEN = "your_token_here"
   curl.exe -d "" https://lichess.org/api/bot/account/upgrade `
     -H "Authorization: Bearer $env:LICHESS_TOKEN"
   ```

3. Install and configure [`lichess-bot`](https://github.com/ShashCode/lichess-bot):

   ```powershell
   git clone https://github.com/ShashCode/lichess-bot.git
   cd lichess-bot
   python -m venv venv
   .\venv\Scripts\Activate.ps1
   pip install -r requirements.txt
   ```

4. Point `config.yml` to the engine:

   ```yaml
   token: "your_token_here"
   engine:
     dir: "D:/chess/tools/bin"
     name: "engine-play.exe"
     protocol: "uci"
   ```

5. Start the bot:

   ```powershell
   python lichess-bot.py
   ```

## 🗂️ Project Structure

| Path | Purpose |
| --- | --- |
| [`src/board.rs`](src/board.rs) | Board state, FEN handling, and game history |
| [`src/movegen.rs`](src/movegen.rs) | Legal move generation |
| [`src/eval.rs`](src/eval.rs) | Position evaluation |
| [`src/search.rs`](src/search.rs) | Main search, pruning, and reductions |
| [`src/ordering.rs`](src/ordering.rs) | Move-ordering heuristics |
| [`src/tt.rs`](src/tt.rs) | Transposition-table implementation |
| [`src/timeman.rs`](src/timeman.rs) | Search time management |
| [`src/uci.rs`](src/uci.rs) | UCI command handling |
| [`tests/`](tests/) | Perft, tactical, UCI, and timing tests |
| [`tools/`](tools/) | Benchmarking and match scripts |

## 🧭 Roadmap

- [ ] Continue tuning evaluation and search heuristics
- [ ] Expand automated regression and strength testing
- [ ] Improve cross-platform support
- [ ] Add richer analysis and match tooling

## 🤝 Contributing

Issues, benchmarks, bug reports, and pull requests are welcome. When reporting a playing-strength or search issue, include the position in **FEN**, the command sequence, and reproducible test details whenever possible.

1. Fork the repository.
2. Create a focused branch.
3. Make and test your change with `cargo test --release`.
4. Open a pull request with a concise explanation and benchmark impact.

## 📄 License

A license has not yet been published for this repository. Add a `LICENSE` file before distributing the engine as open-source software.

<div align="center">

[⬆️ Back to top](#rust-chess-engine)

</div>
