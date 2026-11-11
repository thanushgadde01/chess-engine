#[derive(Clone, Debug, Default)]
pub struct TimeManager;

#[derive(Clone, Copy, Debug)]
pub struct TimeAllocation {
    pub optimum_time_ms: u64, // Soft limit: don't start a new depth after this
    pub max_time_ms: u64,     // Hard limit: abort search immediately if reached
}

impl TimeManager {
    /// Calculate allocated search time given clock settings.
    /// Uses ~remaining/25 + inc/2, safety margin of 50ms, with a hard cap well below remaining time.
    pub fn allocate(
        time_left_ms: Option<u64>,
        inc_ms: Option<u64>,
        movestogo: Option<u64>,
    ) -> Option<TimeAllocation> {
        let time = time_left_ms?;
        let inc = inc_ms.unwrap_or(0);
        let mtg = movestogo.unwrap_or(25).clamp(1, 50);

        // Safety margin: subtract 50ms buffer for GUI communication/process latency
        let usable_time = time.saturating_sub(50);
        if usable_time <= 10 {
            // Extreme emergency time: make an instant move in 5ms
            return Some(TimeAllocation {
                optimum_time_ms: 5,
                max_time_ms: 10.min(time),
            });
        }

        // Standard formula: remaining / mtg + inc / 2
        let target = (usable_time / mtg) + (inc / 2);

        // Hard limit: strictly capped at at most 80% of usable time (or usable_time - 20ms)
        let hard_limit = (usable_time * 8 / 10).min(usable_time.saturating_sub(20)).max(10);
        let soft_limit = target.min(hard_limit / 2).max(5);

        Some(TimeAllocation {
            optimum_time_ms: soft_limit,
            max_time_ms: hard_limit,
        })
    }
}
