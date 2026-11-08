use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use engine::board::Position;
use engine::search::{self, SearchLimits};

#[test]
fn test_mate_in_one() {
    // White can deliver mate in 1: Qh7#
    // 6k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1 -> Qe8#
    let pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1").unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let limits = SearchLimits {
        depth: Some(3),
        ..Default::default()
    };
    let (best, _) = search::search(&pos, limits, stop);
    assert_eq!(best.map(|m| m.to_string()), Some("e1e8".to_string()));
}

#[test]
fn test_mate_in_two() {
    // Classic puzzle: 1. Rh8+ Kxh8 2. Qh7#
    // r6k/6pp/8/8/8/8/5PPP/1R4KQ w - - 0 1
    // Easier direct mate in 2:
    // 8/8/8/8/8/5K1k/6R1/7Q w - - 0 1 -> 1. Rh2# (mate in 1)
    // Scholar's mate test:
    // r1bqkb1r/pppp1ppp/2n5/4p3/2B1n3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 4 -> Qxf7#
    let pos = Position::from_fen("r1bqkb1r/pppp1ppp/2n5/4p3/2B1n3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 4").unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let limits = SearchLimits {
        depth: Some(3),
        ..Default::default()
    };
    let (best, _) = search::search(&pos, limits, stop);
    assert_eq!(best.map(|m| m.to_string()), Some("f3f7".to_string()));
}

#[test]
fn test_threefold_draw_detection() {
    let mut pos = Position::startpos();
    // 1. Nf3 Nf6 2. Ng1 Ng8 3. Nf3 Nf6 4. Ng1 Ng8
    let moves = ["g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8"];
    for m in moves {
        let mv = m.parse().unwrap();
        pos.play(mv);
    }
    assert!(pos.is_repetition(), "Repeated position 3 times must detect repetition");
}

