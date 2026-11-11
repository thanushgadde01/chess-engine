use cozy_chess::{Board, Piece};

fn main() {
    let board = Board::default();
    println!("Startpos legal moves:");
    board.generate_moves(|moves| {
        for m in moves {
            let from_p = board.piece_on(m.from);
            if from_p == Some(Piece::King) {
                println!("King move: from {:?} to {:?}", m.from, m.to);
            }
        }
        false
    });

    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let b: Board = fen.parse().unwrap();
    println!("Castling rights position legal moves:");
    b.generate_moves(|moves| {
        for m in moves {
            let from_p = b.piece_on(m.from);
            if from_p == Some(Piece::King) {
                println!(
                    "King move: from {:?} to {:?} (to_string: {})",
                    m.from, m.to, m
                );
            }
        }
        false
    });
}
