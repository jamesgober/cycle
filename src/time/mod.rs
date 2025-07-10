//! Time utilities and sleep functions

use std::time::{Duration, Instant};

/// Sleep for a specified duration
pub async fn sleep(duration: Duration) {
    // TODO: Implement actual async sleep
    std::thread::sleep(duration);
}

/// Create an interval timer
pub fn interval(period: Duration) -> Interval {
    Interval::new(period)
}

/// Interval timer
pub struct Interval {
    period: Duration,
}

impl Interval {
    fn new(period: Duration) -> Self {
        Self { period }
    }
    
    /// Wait for the next tick
    pub async fn tick(&mut self) -> Instant {
        // TODO: Implement actual interval timing
        sleep(self.period).await;
        Instant::now()
    }
}
