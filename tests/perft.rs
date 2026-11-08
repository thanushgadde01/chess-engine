use engine::board::Position;

#[test]
fn test_perft_startpos_depth_5() {
    let pos = Position::startpos();
    assert_eq!(pos.perft(5), 4_865_609);
}

#[test]
fn test_perft_startpos_depth_6() {
    let pos = Position::startpos();
    assert_eq!(pos.perft(6), 119_060_324);
}

#[test]
fn test_perft_kiwipete_depth_4() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .expect("Valid Kiwipete FEN");
    assert_eq!(pos.perft(4), 4_085_603);
}

#[test]
fn test_perft_divide() {
    let pos = Position::startpos();
    let divide = pos.perft_divide(1);
    let total: u64 = divide.iter().map(|(_, n)| *n).sum();
    assert_eq!(total, 20);
    assert_eq!(divide.len(), 20);
}

