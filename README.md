# RustChess

RustChess is a UCI chess engine written in Rust.

## Build

Install the stable Rust toolchain, then run:

```text
cargo build --release
```

The optimized binary is written to `target/release/engine.exe` on Windows.

## UCI smoke test

```text
uci
isready
quit
```

The engine is being implemented phase by phase; see `STATUS.md` for current progress.
