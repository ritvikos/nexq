extern crate time;

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use time::{Duration, OffsetDateTime};

#[derive(Debug)]
pub struct Message {
    /// Unique id
    pub id: String,

    /// Payload
    pub payload: String,

    /// Attempts
    pub attempts: AtomicUsize,

    /// Attached metadata
    pub metadata: HashMap<String, Vec<String>>,

    /// Time to live
    pub ttl: Option<Duration>,

    /// Current date and time (in utc)
    pub timestamp: OffsetDateTime,

    /// Acknowledgement status
    pub ack: AtomicBool,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            payload: self.payload.clone(),
            attempts: AtomicUsize::new(self.attempts.load(Ordering::Acquire)),
            metadata: self.metadata.clone(),
            ttl: self.ttl,
            timestamp: self.timestamp,
            ack: AtomicBool::new(self.ack.load(Ordering::Acquire)),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: String::default(),
            attempts: AtomicUsize::default(),
            metadata: HashMap::default(),
            payload: String::default(),
            timestamp: OffsetDateTime::now_utc(),
            ttl: None,
            ack: AtomicBool::default(),
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

    pub fn with_attempts(mut self, attempts: AtomicUsize) -> Self {
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
            ack: self.ack,
        }
    }
}
