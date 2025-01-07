extern crate time;

use time::OffsetDateTime;

/// Handle metadata for partition manager.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// Time at which last partition was inserted
    pub last_inserted_at: OffsetDateTime,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            last_inserted_at: OffsetDateTime::now_utc(),
        }
    }
}
