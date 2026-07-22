//! Backoff strategies to handle contention w/ support
//! for spinning and snoozing to avoid busy-waiting.

use crossbeam_utils::Backoff as CrossbeamBackoff;

pub trait Backoff {
    fn spin(&mut self) -> bool;
    fn snooze(&mut self) -> bool;
    fn reset(&mut self);
}

pub struct ExponentialBackoff(CrossbeamBackoff);

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self(CrossbeamBackoff::new())
    }
}

impl Backoff for ExponentialBackoff {
    fn spin(&mut self) -> bool {
        self.0.spin();
        true
    }

    fn snooze(&mut self) -> bool {
        self.0.snooze();
        !self.0.is_completed()
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}
