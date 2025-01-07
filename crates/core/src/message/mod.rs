extern crate time;

pub mod hash;
pub mod key;

use self::key::Key;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use time::{Duration, OffsetDateTime};

type Metadata = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub struct Message {
    /// Unique id
    pub id: String,

    /// Payload
    pub payload: String,

    /// Attempts
    pub attempts: AtomicUsize,

    /// Attached metadata
    pub metadata: Option<Metadata>,

    /// Time to live
    pub ttl: Option<Duration>,

    /// Current date and time (in utc)
    pub timestamp: OffsetDateTime,

    /// Acknowledgement status
    pub ack: AtomicBool,

    /// Partition Key
    pub key: Option<Key>,
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
            key: self.key.clone(),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: String::default(),
            attempts: AtomicUsize::default(),
            metadata: None,
            payload: String::default(),
            timestamp: OffsetDateTime::now_utc(),
            ttl: None,
            ack: AtomicBool::default(),
            key: None,
        }
    }
}

impl Message {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    #[must_use]
    pub fn with_attempts(mut self, attempts: impl Into<AtomicUsize>) -> Self {
        self.attempts = attempts.into();
        self
    }

    #[must_use]
    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = payload.into();
        self
    }

    #[must_use]
    pub fn with_timestamp(mut self, timestamp: OffsetDateTime) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_metadata(mut self, metadata: Option<Metadata>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_key(mut self, key: Option<Key>) -> Self {
        self.key = key;
        self
    }

    pub fn with_ttl(mut self, ttl: Option<Duration>) -> Self {
        self.ttl = ttl;
        self
    }

    #[must_use]
    pub fn build(self) -> Self {
        Self {
            id: self.id,
            attempts: self.attempts,
            metadata: self.metadata,
            payload: self.payload,
            timestamp: self.timestamp,
            ttl: self.ttl,
            ack: self.ack,
            key: self.key,
        }
    }

    /// Set acknowledgement status for the message.
    pub fn ack(&mut self, ack: impl Into<AtomicBool>) {
        self.ack = ack.into();
    }
}

#[cfg(test)]
mod tests {
    use crate::message::{hash::ToHash, key::Key, Message};
    use std::sync::atomic::Ordering;
    use time::OffsetDateTime;
    use ulid::Ulid;

    const HASH_OUTPUT: usize = 320927739;

    #[test]
    fn test_message_ack() {
        // Create id.
        let id = Ulid::new();

        // Create a new Message.
        let mut message = Message::new()
            .with_id(id)
            .with_ttl(None)
            .with_payload("payload_001")
            .with_attempts(0)
            .with_timestamp(OffsetDateTime::now_utc())
            .with_key(Some(Key::Hash("test key".to_string())))
            .build();

        // Acknowledge the message.
        message.ack(true);

        assert!(message.ack.load(Ordering::Relaxed))
    }

    #[test]
    fn test_message_key_hash() {
        let key = "test key".to_string();
        assert_eq!(HASH_OUTPUT, key.to_hash());
    }
}
