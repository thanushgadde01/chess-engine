use std::io::{self, BufRead, Write};

use cozy_chess::Move;

use crate::board::Position;
use crate::search;

pub fn run() {
    let stdin = io::stdin();
    let mut position = Position::startpos();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };

        let mut parts = line.split_whitespace();
        match parts.next().unwrap_or_default() {
            "uci" => {
                println!("id name RustChess 0.1");
                println!("id author thanushgadde01");
                println!("option name Hash type spin default 16 min 1 max 1024");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => position = Position::startpos(),
            "position" => {
                if let Err(error) = parse_position(&mut position, parts) {
                    eprintln!("info string position error: {error}");
                }
            }
            "go" => {
                let depth = parse_depth(parts).unwrap_or(3);
                if let Some(best) = search::search(position.board(), depth) {
                    println!("bestmove {best}");
                } else {
                    println!("bestmove 0000");
                }
            }
            "stop" => {}
            "bench" => {
                let start = std::time::Instant::now();
                let _ = search::search(position.board(), 4);
                let elapsed = start.elapsed().as_millis().max(1);
                println!("info string bench time {elapsed} ms");
            }
            "quit" => break,
            _ => {}
        }
        let _ = io::stdout().flush();
    }
}

fn parse_position<'a>(
    position: &mut Position,
    parts: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    let tokens: Vec<&str> = parts.collect();
    match tokens.first().copied() {
        Some("startpos") => *position = Position::startpos(),
        Some("fen") => {
            let moves_at = tokens.iter().position(|part| *part == "moves");
            let fen_end = moves_at.unwrap_or(tokens.len());
            *position = Position::from_fen(&tokens[1..fen_end].join(" "))?;
        }
        _ => return Err("expected startpos or fen".to_string()),
    }

    if let Some(index) = tokens.iter().position(|part| *part == "moves") {
        for notation in &tokens[index + 1..] {
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

fn parse_depth<'a>(parts: impl Iterator<Item = &'a str>) -> Option<u8> {
    let mut parts = parts.peekable();
    while let Some(part) = parts.next() {
        if part == "depth" {
            return parts.next()?.parse().ok();
        }
    }
    None
}
