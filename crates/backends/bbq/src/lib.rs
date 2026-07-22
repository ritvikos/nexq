//! Lock-free MPMC block-based bounded queue implementation, based on the
//! [BBQ paper][bbq-paper]: "A Block-based Bounded Queue for Exchanging
//! Data and Profiling."
//!
//! [bbq-paper]: https://www.usenix.org/conference/atc22/presentation/wang-jiawei

mod backoff;
mod block;
mod consumer;
mod cursor;
pub mod errors;
mod producer;
mod queue;
mod slot;
#[cfg(test)]
mod tests;

pub use backoff::{Backoff, ExponentialBackoff};
pub use queue::Queue;
