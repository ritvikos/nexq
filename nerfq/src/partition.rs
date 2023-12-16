extern crate ulid;

use crate::{
    error::{Error, Kind, PartitionError},
    storage::Storage,
    strategy::Strategy,
};
use ulid::Ulid;

#[derive(Debug, Default)]
pub struct Partitions {
    /// Partitions
    pub partitions: Vec<Partition>,

    /// Max number of partitions
    pub max_count: Option<usize>,

    /// Partitioning Strategy
    strategy: Strategy,
}

impl Clone for Partitions {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            max_count: self.max_count,
            strategy: self.strategy.clone(),
        }
    }
}

impl Partitions {
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

    /// Set partitioning strategy.
    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Get total number of partitions.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.partitions.len()
    }

    /// Get index based on the strategy.
    pub fn rotate(&self) -> usize {
        self.strategy.rotate(&self.count())
    }

    /// Insert a partition with custom configuration,
    /// defaulting to standard if unspecified.
    pub fn insert(&mut self, partition: Option<Partition>) -> Result<(), Error> {
        match partition {
            Some(partition) => self.try_insert(partition),
            None => self.try_insert(Partition::default()),
        }
    }

    /// Insert partition.
    fn try_insert(&mut self, partition: Partition) -> Result<(), Error> {
        if let Some(max_count) = self.max_count {
            if self.partitions.len() < max_count {
                self.partitions.push(partition);
                Ok(())
            } else {
                Err(Error::new(Kind::Partition(PartitionError::MaxCount)))
            }
        } else {
            self.partitions.push(partition);
            Ok(())
        }
    }

    // TODO:
    // Use another data structure.
    // Support efficient deletion.
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    /// Partition id.
    pub id: Ulid,

    /// Underlying storage.
    pub storage: Storage,
}

impl Partition {
    /// Creates new instance
    pub fn new() -> Self {
        Self {
            id: Ulid::new(),
            ..Default::default()
        }
    }

    /// Sets the storage for partition
    pub fn with_storage(mut self, storage: Storage) -> Self {
        self.storage = storage;
        self
    }
}

/// Partitioning strategy
#[derive(Debug, Default)]
pub enum PartitionStrategy {
    /// Allocating partitions in a circular order.
    #[default]
    RoundRobin,
}

#[cfg(test)]
mod tests {
    use super::{Partition, Partitions};
    use crate::strategy::Strategy;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_partitions_insertion_passed() {
        // Total partition to insert.
        let count = 100;

        // Create partitions.
        let mut partitions = Partitions::new();

        // Insert 100 items in the partitions.
        (1..=count).for_each(|_| {
            assert!(
                partitions.insert(Some(Partition::new())).is_ok(),
                "Inserting partition without any limits."
            )
        });

        assert_eq!(count, partitions.count())
    }

    #[test]
    fn test_partitions_insertion_failed() {
        // Create 3 partition.
        let one = Partition::new();
        let two = Partition::new();
        let three = Partition::new();

        // Create partitions with maximum count condition.
        // Means, it cannot hold more than 2 partition.
        let mut partitions = Partitions::new().with_max_count(2);

        assert!(partitions.insert(Some(one)).is_ok());
        assert!(partitions.insert(Some(two)).is_ok());
        assert!(
            partitions.insert(Some(three)).is_err(),
            "If partition count exceeds the maximum limit, it results in error."
        )
    }

    #[test]
    fn test_partitions_with_round_robin_strategy() {
        // Define round robin strategy.
        let round_robin = Strategy::RoundRobin {
            idx: AtomicUsize::new(usize::MAX),
        };

        // Create partitions.
        let mut partitions = Partitions::new().with_strategy(round_robin);

        // Insert 4 partition.
        (1..=4).for_each(|_| {
            let partition = Partition::new();
            partitions.insert(Some(partition)).unwrap();
        });

        // Check the overflow scenario.
        assert_eq!(3, partitions.rotate()); // chooses 4th partition (3rd index)

        // Iterate the newly inserted partitions.
        [0, 1, 2, 3].iter().for_each(|result| {
            assert_eq!(result, &partitions.rotate());
        });

        // Insert 5 more partition.
        (1..=5).for_each(|_| {
            let partition = Partition::new();
            partitions.insert(Some(partition)).unwrap();
        });

        // Newly inserted partitions are taken into consideration.
        [4, 5, 6, 7, 8].iter().for_each(|result| {
            assert_eq!(result, &partitions.rotate());
        });

        // Final iterations.
        (1..=5).for_each(|_| {
            (0..=8).for_each(|result| assert_eq!(result, partitions.rotate()));
        })
    }
}
