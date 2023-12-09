extern crate dashmap;

use crate::{message::Message, partition::Partitions, queue::Queue, retention::RetentionPolicy};
use dashmap::DashMap;
use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Name of the station
type StationName = String;

#[derive(Debug, Default)]
pub struct Stations {
    // Replace dashmap with a concurrent trie for efficient name-based filtering? (similar to NATS subjects, e.g., subject.sub.*).
    /// Station name and metadata
    pub stations: DashMap<StationName, Station>,

    /// Total stations
    pub count: AtomicUsize,

    /// Max number of stations
    pub max_count: Option<AtomicUsize>,
}

impl Clone for Stations {
    fn clone(&self) -> Self {
        Self {
            stations: self.stations.clone(),
            count: AtomicUsize::new(self.count.load(Ordering::Acquire)),
            max_count: None,
        }
    }
}

impl Stations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count = Some(AtomicUsize::new(max_count));
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
        match &self.max_count {
            Some(max_count) => {
                self.count.load(Ordering::Acquire) < max_count.load(Ordering::Acquire)
            }
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

    /// Queue containing messages
    #[deprecated(note = "Use partitions instead")]
    pub queue: Queue,

    /// Partitions
    pub partitions: Partitions,

    /// Number of messages
    pub count: AtomicUsize,

    /// Retention policy
    pub retention_policy: RetentionPolicy,
}

/// Store
#[derive(Clone, Debug)]
pub enum Store {
    /// In-Memory
    Memory(Queue),

    /// Persistent
    Persistent(),
}

impl Default for Store {
    fn default() -> Self {
        Self::Memory(Queue::default())
    }
}

impl Clone for Station {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            queue: self.queue.clone(),
            count: AtomicUsize::new(self.count.load(Ordering::Acquire)),
            retention_policy: self.retention_policy.clone(),
            partitions: self.partitions.clone(),
        }
    }
}

impl Station {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_retention_policy(mut self, retention_policy: RetentionPolicy) -> Self {
        self.retention_policy = retention_policy;
        self
    }

    pub fn with_queue(mut self, queue: Queue) -> Self {
        self.queue = queue;
        self
    }

    pub fn build(self) -> Self {
        Self {
            name: self.name,
            queue: self.queue,
            count: self.count,
            retention_policy: self.retention_policy,
            partitions: self.partitions,
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
    use std::sync::atomic::AtomicUsize;
    use time::OffsetDateTime;

    #[test]
    fn test_partition_enqueue_message() {
        let mut station = Station::new();

        let message = Message::new()
            .with_id("msg_001".into())
            .with_ttl(None)
            .with_payload("payload_001".into())
            .with_attempts(AtomicUsize::default())
            .with_timestamp(OffsetDateTime::now_utc())
            .build();

        station.enqueue(message).unwrap();
    }
}
