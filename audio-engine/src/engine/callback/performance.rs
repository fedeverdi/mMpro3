use std::time::Instant;

/// Performance statistics tracker
/// Tracks min/max/avg processing time over a window
pub struct PerformanceStats {
    pub buffer_count: usize,
    pub total_process_time_us: u128,
    pub min_process_time_us: u128,
    pub max_process_time_us: u128,
    last_log_time: Instant,
}

impl PerformanceStats {
    pub fn new() -> Self {
        Self {
            buffer_count: 0,
            total_process_time_us: 0,
            min_process_time_us: u128::MAX,
            max_process_time_us: 0,
            last_log_time: Instant::now(),
        }
    }

    /// Record a processing time measurement
    #[inline]
    pub fn record(&mut self, process_time_us: u128) {
        self.buffer_count += 1;
        self.total_process_time_us += process_time_us;
        self.min_process_time_us = self.min_process_time_us.min(process_time_us);
        self.max_process_time_us = self.max_process_time_us.max(process_time_us);
    }

    /// Check if it's time to send stats (every 2-3 seconds)
    pub fn should_log(&self) -> bool {
        self.last_log_time.elapsed().as_secs() >= 2
    }

    /// Reset statistics after sending
    pub fn reset(&mut self) {
        self.buffer_count = 0;
        self.total_process_time_us = 0;
        self.min_process_time_us = u128::MAX;
        self.max_process_time_us = 0;
        self.last_log_time = Instant::now();
    }
}
