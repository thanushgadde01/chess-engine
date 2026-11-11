use engine::uci;
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "bench" {
        println!("Starting benchmark...");
        let _ = io::stdout().flush();
        uci::run_bench();
        return;
    }

    uci::run();
}
