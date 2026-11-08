use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

use cozy_chess::Move;

use crate::board::Position;
use crate::search::{self, SearchLimits};

enum SearchCommand {
    Go(Position, SearchLimits),
    Stop,
    Quit,
}

pub fn run() {
    let stdin = io::stdin();
    let mut position = Position::startpos();

    // Background search worker thread
    let (tx, rx) = channel::<SearchCommand>();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_worker = Arc::clone(&stop_flag);

    let worker_handle = thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                SearchCommand::Go(pos, limits) => {
                    stop_flag_worker.store(false, Ordering::Relaxed);
                    let (best, _nodes) = search::search(&pos, limits, Arc::clone(&stop_flag_worker));
                    let mv_str = best.map(|m| m.to_string()).unwrap_or_else(|| "0000".to_string());
                    println!("bestmove {mv_str}");
                    let _ = io::stdout().flush();
                }
                SearchCommand::Stop => {
                    stop_flag_worker.store(true, Ordering::Relaxed);
                }
                SearchCommand::Quit => break,
            }
        }
    });

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let cmd = parts.next().unwrap_or_default();

        match cmd {
            "uci" => {
                println!("id name RustChess 0.1");
                println!("id author thanushgadde01");
                println!("option name Hash type spin default 16 min 1 max 1024");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                stop_flag.store(true, Ordering::Relaxed);
                position = Position::startpos();
            }
            "position" => {
                let rest: Vec<&str> = parts.collect();
                if let Err(error) = parse_position(&mut position, &rest) {
                    eprintln!("info string position error: {error}");
                }
            }
            "go" => {
                let rest: Vec<&str> = parts.collect();
                let limits = parse_go(&rest);
                stop_flag.store(true, Ordering::Relaxed);
                let _ = tx.send(SearchCommand::Go(position.clone(), limits));
            }
            "stop" => {
                stop_flag.store(true, Ordering::Relaxed);
                let _ = tx.send(SearchCommand::Stop);
            }
            "bench" => {
                run_bench();
            }
            "setoption" => {
                // e.g., setoption name Hash value 32
                // options can be handled here as we implement TT in Phase 4
            }
            "quit" => {
                stop_flag.store(true, Ordering::Relaxed);
                let _ = tx.send(SearchCommand::Quit);
                break;
            }
            _ => {}
        }
        let _ = io::stdout().flush();
    }

    let _ = tx.send(SearchCommand::Quit);
    let _ = worker_handle.join();
}

fn parse_position(position: &mut Position, tokens: &[&str]) -> Result<(), String> {
    if tokens.is_empty() {
        return Err("missing position arguments".to_string());
    }

    match tokens[0] {
        "startpos" => *position = Position::startpos(),
        "fen" => {
            let moves_idx = tokens.iter().position(|&p| p == "moves");
            let fen_end = moves_idx.unwrap_or(tokens.len());
            if fen_end <= 1 {
                return Err("missing FEN tokens".to_string());
            }
            *position = Position::from_fen(&tokens[1..fen_end].join(" "))?;
        }
        _ => return Err("expected startpos or fen".to_string()),
    }

    if let Some(index) = tokens.iter().position(|&p| p == "moves") {
        for &notation in &tokens[index + 1..] {
            let mv = notation
                .parse::<Move>()
                .map_err(|_| format!("invalid move {notation}"))?;
            if !position.board().is_legal(mv) {
                return Err(format!("illegal move {notation}"));
            }
            position.play(mv);
        }
    }
    Ok(())
}

fn parse_go(tokens: &[&str]) -> SearchLimits {
    let mut limits = SearchLimits::default();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.depth = Some(val);
                    i += 1;
                }
            }
            "nodes" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.nodes = Some(val);
                    i += 1;
                }
            }
            "movetime" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.movetime = Some(val);
                    i += 1;
                }
            }
            "wtime" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.wtime = Some(val);
                    i += 1;
                }
            }
            "btime" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.btime = Some(val);
                    i += 1;
                }
            }
            "winc" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.winc = Some(val);
                    i += 1;
                }
            }
            "binc" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.binc = Some(val);
                    i += 1;
                }
            }
            "movestogo" => {
                if let Some(val) = tokens.get(i + 1).and_then(|s| s.parse().ok()) {
                    limits.movestogo = Some(val);
                    i += 1;
                }
            }
            "infinite" => {
                limits.infinite = true;
            }
            _ => {}
        }
        i += 1;
    }
    limits
}

pub fn run_bench() {
    let bench_positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ];

    let start = std::time::Instant::now();
    let mut total_nodes = 0u64;

    for (idx, fen) in bench_positions.iter().enumerate() {
        if let Ok(pos) = Position::from_fen(fen) {
            let limits = SearchLimits {
                depth: Some(5),
                ..Default::default()
            };
            let stop = Arc::new(AtomicBool::new(false));
            let (_mv, nodes) = search::search(&pos, limits, stop);
            total_nodes += nodes;
            eprintln!("Bench [{}/{}] nodes: {}", idx + 1, bench_positions.len(), nodes);
        }
    }

    let elapsed = start.elapsed().as_millis().max(1);
    let nps = (total_nodes as u128 * 1000) / elapsed;
    println!("{total_nodes} nodes {nps} nps in {elapsed} ms");
    let _ = io::stdout().flush();
}
