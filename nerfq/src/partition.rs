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
    pub fn count(&self) -> usize {
        self.partitions.len()
    }

    /// Get next partition based on specified partitioning strategy.
    pub fn get_next_by(&mut self, strategy: PartitionStrategy) -> Partition {
        match strategy {
            PartitionStrategy::RoundRobin => self.round_robin_next(),
        }
    }

    /// Get next partition based on round robin partitioning.
    pub fn round_robin_next(&self) -> Partition {
        let idx = self.round_robin_rotate();

        // index is always within bounds.
        self.partitions[idx].clone()
    }

    /// Implement round robin partitioning.
    /// Cycle through the partitions.
    pub fn round_robin_rotate(&self) -> usize {
        if self.partitions.is_empty() {
            return 0;
        }

        self.idx.fetch_add(1, Ordering::AcqRel);
        let next_idx = self.idx.load(Ordering::Acquire);

        next_idx % self.count()
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

        assert_eq!(1, partitions.round_robin_rotate());
        assert_eq!(2, partitions.round_robin_rotate());
        assert_eq!(3, partitions.round_robin_rotate());
        assert_eq!(0, partitions.round_robin_rotate());
    }

    #[test]
    fn test_partitions_get_next() {
        let mut partitions = Partitions::new();

        let mut ids = Vec::new();

        (1..5).for_each(|_| {
            let partition = Partition::new();
            ids.push(partition.id);
            partitions.insert(Some(partition)).unwrap();
        });

        assert_eq!(ids[1], partitions.round_robin_next().id);
        assert_eq!(ids[2], partitions.round_robin_next().id);
        assert_eq!(ids[3], partitions.round_robin_next().id);
        assert_eq!(ids[0], partitions.round_robin_next().id);

        // More rigorous testing to ensure multiple iterations
        // doesn't overflow bounds with current round robin
        // implementation.

        // let ulids = [ids[1], ids[2], ids[3], ids[0]];
        // (1..101).for_each(|_| {
        //     ulids.iter().for_each(|i| {
        //         assert_eq!(i, &partitions.get_next().id);
        //     });
        // });
    }

    // #[test]
    // fn test_partitions_get_next_by_round_robin() {
    //     use crate::partition::PartitionStrategy;
    //     let mut partitions = Partitions::new();

    //     let mut ids = Vec::new();

    //     (1..5).for_each(|_| {
    //         let partition = Partition::new();
    //         ids.push(partition.id);
    //         partitions.insert(Some(partition)).unwrap();
    //     });

    //     assert_eq!(
    //         ids[1],
    //         partitions.get_next_by(PartitionStrategy::RoundRobin).id
    //     );
    //     assert_eq!(
    //         ids[2],
    //         partitions.get_next_by(PartitionStrategy::RoundRobin).id
    //     );
    //     assert_eq!(
    //         ids[3],
    //         partitions.get_next_by(PartitionStrategy::RoundRobin).id
    //     );
    //     assert_eq!(
    //         ids[0],
    //         partitions.get_next_by(PartitionStrategy::RoundRobin).id
    //     );
    // }
}
