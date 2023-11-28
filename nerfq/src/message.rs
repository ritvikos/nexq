extern crate time;

use std::sync::atomic::AtomicU16;
use time::OffsetDateTime;

pub struct Message {
    pub id: String,
    pub payload: String,
    pub timestamp: OffsetDateTime,
    pub attempts: AtomicU16,
}
