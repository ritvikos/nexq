use std::sync::atomic::{AtomicUsize, Ordering};

/// Handle internal state for partition manager.
#[derive(Debug)]
pub struct State {
    /// Round robin index.
    pub round_robin: AtomicUsize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            round_robin: AtomicUsize::new(0),
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            round_robin: AtomicUsize::new(self.round_robin.load(Ordering::Acquire)),
        }
    }
}
