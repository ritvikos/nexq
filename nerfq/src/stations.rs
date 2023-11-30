extern crate dashmap;

use crate::{message::Message, queue::Queue, retention::RetentionPolicy};
use dashmap::DashMap;
use std::{
    error::Error,
    sync::atomic::{AtomicU64, Ordering},
};

/// Name of the Station
type StationName = String;

#[derive(Debug, Default)]
pub struct Stations {
    // Replace dashMap with a concurrent trie for efficient name-based filtering? (similar to NATS subjects, e.g., subject.sub.*).
    /// Station name and metadata
    pub stations: DashMap<StationName, Station>,

    /// Total stations
    pub count: AtomicU64,

    /// Max number of stations
    pub max_count: Option<u64>,
}

impl Clone for Stations {
    fn clone(&self) -> Self {
        Self {
            stations: self.stations.clone(),
            count: AtomicU64::new(self.count.load(Ordering::SeqCst)),
            max_count: self.max_count,
        }
    }
}

impl Stations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_count(mut self, max_count: Option<u64>) -> Self {
        self.max_count = max_count;
        self
    }

    pub fn insert(&self, station: Station) -> Result<(), Box<dyn Error>> {
        let name = station.name.clone();

        self.bounded()?;

        if self.stations.insert(name, station).is_some() {
            return Err("Station already exists".into());
        }

        self.count.fetch_add(1, Ordering::AcqRel);

        Ok(())
    }

    pub fn remove(&self, name: String) -> Option<(String, Station)> {
        self.stations.remove(&name)
    }

    // Returns `true` if max_count < count.
    fn within_bounds(&self) -> bool {
        match self.max_count {
            Some(max_count) => self.count.load(Ordering::Relaxed) < max_count,
            None => true,
        }
    }

    // Validates message bounds for insertion.
    fn bounded(&self) -> Result<(), Box<dyn Error>> {
        match self.within_bounds() {
            true => Ok(()),
            false => Err("Station is full!".into()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Station {
    /// Station name
    pub name: String,

    /// Station constraints
    pub constraint: StationConstraint,

    /// Queue containing messages
    pub queue: Queue,

    /// Number of messages
    pub count: AtomicU64,

    /// Retention Policy
    pub retention_policy: RetentionPolicy,
}

impl Clone for Station {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            constraint: self.constraint.clone(),
            queue: self.queue.clone(),
            count: AtomicU64::new(self.count.load(Ordering::SeqCst)),
            retention_policy: self.retention_policy.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StationConstraint {
    /// Message limits
    pub message_limit: Option<u64>,
}

impl Station {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn build(self) -> Self {
        Self {
            name: self.name,
            constraint: self.constraint,
            queue: self.queue,
            count: self.count,
            retention_policy: self.retention_policy,
        }
    }

    /// Enqueues message into the queue
    pub fn enqueue(&mut self, message: Message) -> Result<(), Box<Message>> {
        self.queue.push(message)?;
        self.count.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    /// Dequeues message from the queue
    pub fn dequeue(&mut self) -> Result<Message, Box<dyn Error>> {
        match self.queue.pop() {
            Some(message) => {
                self.count.fetch_sub(1, Ordering::AcqRel);
                Ok(message)
            }
            None => Err("Cannot Dequeue".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU16;
    use time::OffsetDateTime;

    #[test]
    fn test_partition_enqueue_message() {
        let mut station = Station::new();

        let message = Message::new()
            .with_id("msg_001".into())
            .with_ttl(None)
            .with_payload("payload_001".into())
            .with_attempts(AtomicU16::default())
            .with_timestamp(OffsetDateTime::now_utc())
            .build();

        station.enqueue(message).unwrap();
    }
}
