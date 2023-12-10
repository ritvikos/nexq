extern crate ulid;

use crate::{
    error::{Error, Kind, PartitionError},
    storage::Storage,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use ulid::Ulid;

#[derive(Debug, Default)]
pub struct Partitions {
    /// Partitions
    pub partitions: Vec<Partition>,

    /// Max number of partitions
    pub max_count: Option<usize>,

    /// Current index
    idx: AtomicUsize,
}

impl Clone for Partitions {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            max_count: self.max_count,
            idx: AtomicUsize::new(self.idx.load(Ordering::Acquire)),
        }
    }
}

impl Partitions {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets pre-defined partitions.
    pub fn with_partitions(mut self, partitions: Vec<Partition>) -> Self {
        self.partitions = partitions;
        self
    }

    /// Sets max count limit.
    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count = Some(max_count);
        self
    }

    /// Implements round robin partitioning.
    /// Cycle through the partitions.
    pub fn rotate(&mut self) -> usize {
        if self.partitions.is_empty() {
            return 0;
        }

        let next_idx = self.idx.fetch_add(1, Ordering::Acquire) + 1;
        (next_idx) % self.count()
    }

    /// Get total number of partitions.
    pub fn count(&self) -> usize {
        self.partitions.len()
    }

    /// Inserts a partition with custom configuration,
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

#[cfg(test)]
mod tests {
    use super::{Partition, Partitions};

    #[test]
    fn test_partitions_insert_pass() {
        let partition = Partition::new();
        let mut partitions = Partitions::new();
        assert!(
            partitions.insert(Some(partition)).is_ok(),
            "Inserting partition without any limits."
        )
    }

    #[test]
    fn test_partitions_insert_fail() {
        let partition_one = Partition::new();
        let partition_two = Partition::new();
        let partition_three = Partition::new();

        let mut partitions = Partitions::new().with_max_count(2);

        assert!(partitions.insert(Some(partition_one)).is_ok());
        assert!(partitions.insert(Some(partition_two)).is_ok());
        assert!(
            partitions.insert(Some(partition_three)).is_err(),
            "If partition count exceeds the maximum limit, it results in error."
        )
    }

    #[test]
    fn test_partitions_round_robin_routing() {
        let mut partitions = Partitions::new();

        (1..5).for_each(|_| {
            let partition = Partition::new();
            partitions.insert(Some(partition)).unwrap();
        });

        assert_eq!(1, partitions.rotate());
        assert_eq!(2, partitions.rotate());
        assert_eq!(3, partitions.rotate());
        assert_eq!(0, partitions.rotate());
        assert_eq!(1, partitions.rotate());
        assert_eq!(2, partitions.rotate());
        assert_eq!(3, partitions.rotate());
        assert_eq!(0, partitions.rotate());
    }
}
