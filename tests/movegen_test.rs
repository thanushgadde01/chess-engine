use cozy_chess::Board;
use engine::board::Position;
use engine::movegen::{move_to_uci, parse_uci_move};
use engine::search::{self, SearchLimits};
use engine::tt::{TranspositionTable, DEFAULT_TT_SIZE_MB};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[test]
fn test_castling_roundtrip_both_colors() {
    // White and Black both have kingside and queenside castling rights
    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let board: Board = fen.parse().unwrap();

    // White kingside: engine move is e1h1, UCI is e1g1
    let e1g1 = parse_uci_move(&board, "e1g1").expect("e1g1 should parse");
    assert_eq!(move_to_uci(&board, e1g1), "e1g1");

    // White queenside: engine move is e1a1, UCI is e1c1
    let e1c1 = parse_uci_move(&board, "e1c1").expect("e1c1 should parse");
    assert_eq!(move_to_uci(&board, e1c1), "e1c1");

    // Now test Black
    let fen_b = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
    let board_b: Board = fen_b.parse().unwrap();

    let e8g8 = parse_uci_move(&board_b, "e8g8").expect("e8g8 should parse");
    assert_eq!(move_to_uci(&board_b, e8g8), "e8g8");

    let e8c8 = parse_uci_move(&board_b, "e8c8").expect("e8c8 should parse");
    assert_eq!(move_to_uci(&board_b, e8c8), "e8c8");
}

#[test]
fn test_promotions_and_en_passant() {
    // Promotion position
    let fen_promo = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
    let board_promo: Board = fen_promo.parse().unwrap();
    for promo_str in ["e7e8q", "e7e8r", "e7e8b", "e7e8n"] {
        let mv = parse_uci_move(&board_promo, promo_str).expect("Valid promotion");
        assert_eq!(move_to_uci(&board_promo, mv), promo_str);
    }

    // En passant position: 1. e4 e5 2. Nf3 Nf6 3. e5 d5 -> White can exd6 (e5d6)
    let fen_ep = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let board_ep: Board = fen_ep.parse().unwrap();
    let mv_ep = parse_uci_move(&board_ep, "e5d6").expect("En passant valid");
    assert_eq!(move_to_uci(&board_ep, mv_ep), "e5d6");
}

#[test]
fn test_roundtrip_all_legal_moves_across_positions() {
    let test_positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
        "8/4P3/8/8/8/8/8/4K2k w - - 0 1",
        "8/4p3/8/8/8/8/8/4k2K b - - 0 1",
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
        "rnbqkbnr/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 3",
    ];

    let mut total_tested = 0;
    for fen in test_positions {
        let board: Board = fen.parse().expect("valid fen");
        board.generate_moves(|moves| {
            for mv in moves {
                let uci = move_to_uci(&board, mv);
                let parsed = parse_uci_move(&board, &uci).unwrap_or_else(|e| {
                    panic!("Failed round-trip parse of '{uci}' in FEN {fen}: {e}")
                });
                assert_eq!(mv, parsed, "Move round-trip mismatch for {uci} in {fen}");
                total_tested += 1;
            }
            false
        });
    }
    assert!(total_tested >= 200, "Tested {total_tested} legal moves");
}

#[test]
fn test_bestmove_is_always_legal_including_after_stop() {
    let test_positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    ];

    let tt = Arc::new(Mutex::new(TranspositionTable::new(DEFAULT_TT_SIZE_MB)));

    for fen in test_positions {
        let pos = Position::from_fen(fen).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let limits = SearchLimits {
            depth: Some(4),
            ..Default::default()
        };
        let (best, _) = search::search(&pos, limits, Arc::clone(&stop), Arc::clone(&tt));
        assert!(best.is_some());
        let m = best.unwrap();
        assert!(
            pos.board().is_legal(m),
            "Bestmove {m} must be legal in {fen}"
        );

        // Also test immediate stop condition: must still return a legal move!
        let stop_early = Arc::new(AtomicBool::new(true)); // Already stopped!
        let limits_early = SearchLimits {
            depth: Some(4),
            ..Default::default()
        };
        let (early_best, _) = search::search(&pos, limits_early, stop_early, Arc::clone(&tt));
        assert!(early_best.is_some());
        let early_m = early_best.unwrap();
        assert!(
            pos.board().is_legal(early_m),
            "Early-stopped move {early_m} must be legal in {fen}"
        );
    }
}
