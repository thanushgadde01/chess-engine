mod board;
mod eval;
mod ordering;
mod search;
mod timeman;
mod tt;
mod uci;

fn main() {
    uci::run();
}
