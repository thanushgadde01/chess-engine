use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn test_uci_protocol_flow() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_engine"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start engine binary");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Send uci
    writeln!(stdin, "uci").unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    let mut saw_uciok = false;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == "uciok" {
            saw_uciok = true;
            break;
        }
        line.clear();
    }
    assert!(saw_uciok, "Engine must reply with uciok");

    // 2. Send isready
    writeln!(stdin, "isready").unwrap();
    stdin.flush().unwrap();

    line.clear();
    let mut saw_readyok = false;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == "readyok" {
            saw_readyok = true;
            break;
        }
        line.clear();
    }
    assert!(saw_readyok, "Engine must reply with readyok");

    // 3. Play a sequence of moves and search
    writeln!(stdin, "ucinewgame").unwrap();
    writeln!(stdin, "position startpos moves e2e4 e7e5 g1f3 b8c6").unwrap();
    writeln!(stdin, "go movetime 100").unwrap();
    stdin.flush().unwrap();

    line.clear();
    let mut bestmove = None;
    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed.starts_with("bestmove") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                bestmove = Some(parts[1].to_string());
            }
            break;
        }
        line.clear();
    }
    assert!(bestmove.is_some(), "Engine must return a bestmove");

    // 4. Test quit
    writeln!(stdin, "quit").unwrap();
    stdin.flush().unwrap();

    let status = child.wait().expect("Failed to wait on child process");
    assert!(status.success());
}
