extern crate time;

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU16, Ordering},
};
use time::{Duration, OffsetDateTime};

#[derive(Debug)]
pub struct Message {
    /// Unique id
    pub id: String,

    /// Payload
    pub payload: String,

    /// Attempts
    pub attempts: AtomicU16,

    /// Attached metadata
    pub metadata: HashMap<String, Vec<String>>,

    /// Time to live
    pub ttl: Option<Duration>,

    /// Current date and time (in utc)
    pub timestamp: OffsetDateTime,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            payload: self.payload.clone(),
            attempts: AtomicU16::new(self.attempts.load(Ordering::SeqCst)),
            metadata: self.metadata.clone(),
            ttl: self.ttl,
            timestamp: self.timestamp,
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: String::default(),
            attempts: AtomicU16::default(),
            metadata: HashMap::default(),
            payload: String::default(),
            timestamp: OffsetDateTime::now_utc(),
            ttl: None,
        }
    }
}

impl Message {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_attempts(mut self, attempts: AtomicU16) -> Self {
        self.attempts = attempts;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Vec<String>>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_payload(mut self, payload: String) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_timestamp(mut self, timestamp: OffsetDateTime) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_ttl(mut self, ttl: Option<Duration>) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn build(self) -> Self {
        Self {
            id: self.id,
            attempts: self.attempts,
            metadata: self.metadata,
            payload: self.payload,
            timestamp: self.timestamp,
            ttl: self.ttl,
        }
    }
}
