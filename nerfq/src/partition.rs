extern crate ulid;

use crate::{
    error::{Error, Kind, PartitionError},
    storage::Storage,
};
use ulid::Ulid;

#[derive(Debug, Default)]
pub struct Partitions {
    /// Partitions
    pub partitions: Vec<Partition>,

    /// Max number of partitions
    pub max_count: Option<usize>,
}

impl Clone for Partitions {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            max_count: self.max_count,
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
        match self.max_count {
            Some(max_count) => {
                if self.partitions.len().lt(&max_count) {
                    self.partitions.push(partition);
                    Ok(())
                } else {
                    Err(Error::new(Kind::Partition(PartitionError::MaxCount)))
                }
            }
            None => {
                self.partitions.push(partition);
                Ok(())
            }
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
            "Inserting partition without any limits"
        )
    }

    #[test]
    fn test_partitions_insert_fail() {
        let partition_one = Partition::new();
        let partition_two = Partition::new();
        let partition_three = Partition::new();

        let mut partitions = Partitions::new().with_max_count(2);

        partitions.insert(Some(partition_one)).unwrap();
        partitions.insert(Some(partition_two)).unwrap();

        assert!(
            partitions.insert(Some(partition_three)).is_err(),
            "If partition count exceeds the maximum limit, it results in error."
        )
    }
}
