use std::collections::VecDeque;
use instant::*;



pub struct FrameRateCounter {
    capacity:   usize,
    history:    VecDeque<Instant>,
}

impl FrameRateCounter {
    pub fn new(capacity: usize) -> Self {
        let mut history = VecDeque::new();
        history.reserve_exact(capacity);
        history.push_back(Instant::now());
        Self { capacity, history }
    }

    pub fn frame(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now - self.history[0];
        let n = self.history.len();
        if n >= self.capacity { self.history.pop_front(); }
        self.history.push_back(now);
        dt / (n as u32)
    }
}
