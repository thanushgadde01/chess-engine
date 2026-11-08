use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct TimeManager;

impl TimeManager {
    /// Calculate allocated search time in milliseconds given clock settings.
    pub fn allocate(
        time_left_ms: Option<u64>,
        inc_ms: Option<u64>,
        movestogo: Option<u64>,
    ) -> Option<Duration> {
        let time = time_left_ms?;
        let inc = inc_ms.unwrap_or(0);
        let mtg = movestogo.unwrap_or(25).clamp(1, 50);

        // Standard formula: remaining / mtg + 3/4 inc
        let mut target = (time / mtg) + (inc * 3 / 4);

        // Safety margin: never exceed remaining - 50ms (or 80% of remaining if small)
        let hard_limit = time.saturating_sub(50).max(10);
        target = target.min(hard_limit).max(10);

        Some(Duration::from_millis(target))
    }
}
