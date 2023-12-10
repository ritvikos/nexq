extern crate dashmap;

use crate::{
    error::{Error, Kind, StationError},
    message::Message,
    partition::Partitions,
    queue::Queue,
    retention::RetentionPolicy,
};
use dashmap::DashMap;

/// Name of the station
type StationName = String;

#[derive(Debug, Default)]
pub struct Stations {
    // Replace dashmap with a concurrent trie for efficient name-based filtering? (similar to NATS subjects, e.g., subject.sub.*).
    /// Station name and metadata
    pub stations: DashMap<StationName, Station>,

    /// Max number of stations
    pub max_count: Option<usize>,
}

impl Clone for Stations {
    fn clone(&self) -> Self {
        Self {
            stations: self.stations.clone(),
            max_count: None,
        }
    }
}

impl Stations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count = Some(max_count);
        self
    }

    pub fn count(&self) -> usize {
        self.stations.len()
    }

    pub fn insert(&self, station: Station) -> Result<(), Error> {
        let name = station.name.clone();

        self.bounded()?;

        if self.stations.insert(name, station).is_some() {
            return Err(Error::new(Kind::Station(StationError::AlreadyExists)));
        }

        Ok(())
    }

    pub fn remove(&self, name: String) -> Option<(String, Station)> {
        self.stations.remove(&name)
    }

    // Returns `true` if max_count < count.
    fn within_bounds(&self) -> bool {
        match &self.max_count {
            Some(max_count) => self.count() < *max_count,
            None => true,
        }
    }

    // Validates message bounds for insertion.
    fn bounded(&self) -> Result<(), Error> {
        match self.within_bounds() {
            true => Ok(()),
            false => Err(Error::new(Kind::Station(StationError::MaxCount))),
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

    /// Retention policy
    pub retention_policy: RetentionPolicy,
}

impl Clone for Station {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            queue: self.queue.clone(),
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
            retention_policy: self.retention_policy,
            partitions: self.partitions,
        }
    }

    /// Enqueues message into the queue
    pub fn enqueue(&mut self, message: Message) -> Result<(), Box<Message>> {
        self.queue.push(message)?;
        Ok(())
    }

    /// Dequeues message from the queue
    pub fn dequeue(&mut self) -> Result<Message, Box<dyn std::error::Error>> {
        match self.queue.pop() {
            Some(message) => Ok(message),
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
