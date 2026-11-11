use engine::board::Position;
use engine::search::{self, SearchLimits};
use engine::timeman::TimeManager;
use engine::tt::{TranspositionTable, DEFAULT_TT_SIZE_MB};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[test]
fn test_timeman_allocation_rules() {
    // 1+0 clock: 60,000 ms remaining, 0 inc
    let alloc = TimeManager::allocate(Some(60_000), Some(0), None).unwrap();
    assert!(alloc.max_time_ms < 60_000, "Hard limit must be well below remaining");
    assert!(alloc.optimum_time_ms <= alloc.max_time_ms);
    // 10+0.1 clock: 600,000 ms, 100 ms inc
    let alloc2 = TimeManager::allocate(Some(600_000), Some(100), Some(30)).unwrap();
    assert!(alloc2.max_time_ms < 600_000);

    // Emergency time: 40ms remaining (less than 50ms buffer)
    let alloc_emerg = TimeManager::allocate(Some(40), Some(0), None).unwrap();
    assert!(alloc_emerg.max_time_ms <= 40);
    assert!(alloc_emerg.optimum_time_ms <= 10);
}

#[test]
fn test_search_never_exceeds_hard_limit() {
    let pos = Position::startpos();
    let tt = Arc::new(Mutex::new(TranspositionTable::new(DEFAULT_TT_SIZE_MB)));

    // Test with small movetime
    for limit_ms in [50, 100, 200] {
        let stop = Arc::new(AtomicBool::new(false));
        let limits = SearchLimits {
            movetime: Some(limit_ms),
            ..Default::default()
        };
        let start = Instant::now();
        let (mv, _) = search::search(&pos, limits, stop, Arc::clone(&tt));
        let elapsed = start.elapsed().as_millis() as u64;

        assert!(mv.is_some(), "Search must return a move");
        // Allow at most 25ms scheduler jitter
        assert!(
            elapsed <= limit_ms + 25,
            "Search elapsed {elapsed}ms exceeded limit {limit_ms}ms"
        );
    }
}

#[test]
fn test_one_plus_zero_simulation_clock() {
    // Simulate a series of 10 moves under 1+0 clock (starting at 60s)
    let mut pos = Position::startpos();
    let mut time_left = 60_000u64;
    let tt = Arc::new(Mutex::new(TranspositionTable::new(DEFAULT_TT_SIZE_MB)));

    for _move_num in 1..=10 {
        let stop = Arc::new(AtomicBool::new(false));
        let limits = SearchLimits {
            wtime: Some(time_left),
            winc: Some(0),
            ..Default::default()
        };
        let start = Instant::now();
        let (best, _) = search::search(&pos, limits, stop, Arc::clone(&tt));
        let elapsed = start.elapsed().as_millis() as u64;

        assert!(best.is_some());
        let mv = best.unwrap();
        assert!(pos.board().is_legal(mv));
        pos.play(mv);

        assert!(elapsed < time_left, "Must never flag / forfeit on time!");
        time_left = time_left.saturating_sub(elapsed);
        assert!(time_left > 0, "Time left must stay positive");
    }
}

