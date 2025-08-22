use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimeFrame {
    pub now: Instant,
    pub last: Instant,
    pub delta: Duration,
}

impl TimeFrame {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            now,
            last: now,
            delta: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.last = self.now;
        self.delta = now.duration_since(self.now);
        self.now = now;
    }

    pub fn update_with_delta(&mut self, delta: Duration) {
        self.last = self.now;
        self.delta = delta;
        self.now = self.last + delta;
    }
}

impl Default for TimeFrame {
    fn default() -> Self {
        Self::new()
    }
}