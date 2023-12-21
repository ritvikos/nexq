extern crate ulid;

use crate::{
    error::{Error, Kind, PartitionError},
    message::{Key, Message, ToHash},
    storage::Store,
};
use std::{
    borrow::BorrowMut,
    cmp::Ordering,
    sync::atomic::{AtomicUsize, Ordering as AtomicOrdering},
};
use ulid::Ulid;

const INCREMENT_UNIT: usize = 1;

#[derive(Debug, Default)]
pub struct PartitionManager {
    /// Partition Manager
    pub partitions: Vec<Partition>,

    /// Max number of partitions
    pub max_count: Option<usize>,

    // Track state.
    state: PartitionState,
}

impl Clone for PartitionManager {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            max_count: self.max_count,
            state: self.state.clone(),
        }
    }
}

impl PartitionManager {
    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set pre-defined partitions.
    pub fn with_partitions(mut self, partitions: Vec<Partition>) -> Self {
        self.partitions = partitions;
        self
    }

    /// Set max count limit.
    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count = Some(max_count);
        self
    }

    /// Get total number of partitions.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.partitions.len()
    }

    /// Returns `true` if there're no partitions,
    /// `false` otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty()
    }

    /// Insert a partition with custom configuration.
    pub fn insert(&mut self, partition: Partition) -> Result<(), Error> {
        let f = |partition: Partition, partitions: &mut Self| partitions.insert_inner(partition);

        self.try_insert(f, partition)
    }

    /// Insert a partition at specified index with custom configuration. \
    /// Ensure that partitions aren't empty.
    pub fn insert_at(&mut self, partition: Partition, idx: usize) -> Result<(), Error> {
        let f = |partition: Partition, partitions: &mut Self| {
            partitions.insert_at_inner(partition, idx)
        };

        self.try_insert(f, partition)
    }

    /// Insert a message based on the availability key.
    pub fn insert_message(&mut self, message: Message) -> Result<(), Error> {
        match &message.key {
            Some(key) => match key {
                Key::Hash(key) => {
                    dbg!(self.hash_key(key));
                    Ok(())
                }
            },
            None => {
                let idx = self.round_robin();
                self.insert_message_inner(idx)
            }
        }
    }

    /// Defines the insertion criterion.
    fn try_insert<F>(&mut self, mut f: F, partition: Partition) -> Result<(), Error>
    where
        F: FnMut(Partition, &mut Self),
    {
        if let Some(ref max_count) = self.max_count {
            match self.count().cmp(max_count) {
                Ordering::Less => {
                    f(partition, self);
                    Ok(())
                }
                Ordering::Equal | Ordering::Greater => {
                    Err(Error::new(Kind::Partition(PartitionError::MaxCount)))
                }
            }
        } else {
            f(partition, self);
            Ok(())
        }
    }

    /// Insert a partition.
    fn insert_inner(&mut self, partition: Partition) {
        self.partitions.push(partition);
    }

    /// Insert a partition at specified index.
    fn insert_at_inner(&mut self, partition: Partition, idx: usize) {
        self.partitions.insert(idx, partition)
    }

    /// Round robin strategy.
    fn round_robin(&mut self) -> usize {
        self.state
            .round_robin
            .fetch_add(INCREMENT_UNIT, AtomicOrdering::AcqRel)
            % self.count()
    }

    /// Hash based strategy.
    fn hash_key(&self, key: &str) -> usize {
        key.to_hash() % self.count()
    }

    // Insert a message.
    fn insert_message_inner(&mut self, idx: usize) -> Result<(), Error> {
        // let partition = self.get_partition_mut(idx);
        // partition.store.
        println!("insert message at idx: {}", idx);
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    /// Partition id.
    pub id: Ulid,

    /// Replicas
    pub replicas: usize,

    /// Underlying storage.
    pub store: Store,
}

impl Partition {
    /// Creates new instance.
    pub fn new() -> Self {
        Self {
            id: Ulid::new(),
            ..Default::default()
        }
    }

    /// Sets the storage for partition.
    pub fn with_store(mut self, store: Store) -> Self {
        self.store = store;
        self
    }
}

#[derive(Debug)]
struct PartitionState {
    pub round_robin: AtomicUsize,
}

impl Default for PartitionState {
    fn default() -> Self {
        Self {
            round_robin: AtomicUsize::new(usize::MAX),
        }
    }
}

impl Clone for PartitionState {
    fn clone(&self) -> Self {
        Self {
            round_robin: AtomicUsize::new(self.round_robin.load(AtomicOrdering::Acquire)),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use super::{Partition, PartitionManager};
    use crate::{
        message::{Key, Message},
        partition::PartitionState,
    };
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_partitions_insertion_failed() {
        // Create 3 partition.
        let one = Partition::new();
        let two = Partition::new();
        let three = Partition::new();

        // Create partitions with maximum count condition.
        // Means, it cannot hold more than 2 partition.
        let mut partitions = PartitionManager::new().with_max_count(2);

        assert!(partitions.insert(one).is_ok());
        assert!(partitions.insert(two).is_ok());
        assert!(
            partitions.insert(three).is_err(),
            "If partition count exceeds the maximum limit, it results in error."
        )
    }

    #[test]
    fn test_partitions_with_round_robin_strategy() {
        // Create partition manager.
        let mut manager = PartitionManager {
            state: PartitionState {
                round_robin: AtomicUsize::new(0),
            },
            ..Default::default()
        };

        // Insert 4 partition.
        (1..=4).for_each(|_| {
            let partition = Partition::new();
            manager.insert(partition).unwrap();
        });

        // Check the overflow scenario.
        assert_eq!(3, manager.round_robin()); // chooses 4th partition (3rd index)

        // Iterate the newly inserted partitions.
        [0, 1, 2, 3].iter().for_each(|result| {
            assert_eq!(result, &manager.round_robin());
        });

        // Insert 5 more partition.
        (1..=5).for_each(|_| {
            let partition = Partition::new();
            manager.insert(partition).unwrap();
        });

        // Newly inserted partitions are taken into consideration.
        [4, 5, 6, 7, 8].iter().for_each(|result| {
            assert_eq!(result, &manager.round_robin());
        });

        // Final iterations.
        (1..=5).for_each(|_| {
            (0..=8).for_each(|result| assert_eq!(result, manager.round_robin()));
        })
    }

    #[test]
    fn test_partitions_insert_message_with_key_hash_strategy() {
        // Create partition manager.
        let mut manager = PartitionManager::new();

        // Insert 4 partition.
        (1..=4).for_each(|_| {
            let partition = Partition::new();
            manager.insert(partition).unwrap();
        });

        // Create a new Message.
        let message = Message::new()
            .with_id("msg_001".into())
            .with_ttl(None)
            .with_payload("payload_001".into())
            .with_attempts(AtomicUsize::default())
            .with_timestamp(OffsetDateTime::now_utc())
            .with_key(Some(Key::Hash("test key".to_string())))
            .build();

        // Insert the message in the partition.
        manager.insert_message(message).unwrap();
    }
}
