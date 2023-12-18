extern crate ulid;

use crate::{
    error::{Error, Kind, PartitionError},
    storage::Storage,
    strategy::Strategy,
};
use std::{cmp::Ordering, ops::Range};
use ulid::Ulid;

// TODO:
// Create a high-level interface to ensure
// there's atleast single pre-configured partition.

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

    /// Returns `true` if there're no partitions,
    /// `false` otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty()
    }

    /// Get index based on the strategy.
    pub fn rotate(&self) -> usize {
        self.strategy.rotate(self.count())
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

    /// Insert a partition.
    fn insert_inner(&mut self, partition: Partition) {
        self.partitions.push(partition);
    }

    /// Insert a partition at specified index.
    fn insert_at_inner(&mut self, partition: Partition, idx: usize) {
        self.partitions.insert(idx, partition)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    /// Partition id.
    pub id: Ulid,

    /// Partition Range
    pub range: Option<Range<usize>>,

    /// Underlying storage.
    pub storage: Storage,
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
    pub fn with_storage(mut self, storage: Storage) -> Self {
        self.storage = storage;
        self
    }

    /// Sets the ranged partition.
    pub fn with_range(mut self, range: Range<usize>) -> Self {
        self.range = Some(range);
        self
    }
}

#[cfg(test)]
mod tests {
    use ulid::Ulid;

    use super::{Partition, Partitions};
    use crate::strategy::Strategy;
    use std::{ops::Range, sync::atomic::AtomicUsize};

    #[test]
    fn test_partitions_insertion_failed() {
        // Create 3 partition.
        let one = Partition::new();
        let two = Partition::new();
        let three = Partition::new();

        // Create partitions with maximum count condition.
        // Means, it cannot hold more than 2 partition.
        let mut partitions = Partitions::new().with_max_count(2);

        assert!(partitions.insert(one).is_ok());
        assert!(partitions.insert(two).is_ok());
        assert!(
            partitions.insert(three).is_err(),
            "If partition count exceeds the maximum limit, it results in error."
        )
    }

    #[test]
    fn test_partitions_with_round_robin_strategy() {
        // Define round robin strategy.
        let round_robin = Strategy::RoundRobin(AtomicUsize::new(usize::MAX));

        // Create partitions.
        let mut partitions = Partitions::new().with_strategy(round_robin);

        // Insert 4 partition.
        (1..=4).for_each(|_| {
            let partition = Partition::new();
            partitions.insert(partition).unwrap();
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
            partitions.insert(partition).unwrap();
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

    #[test]
    fn test_design_pattern() {
        #[derive(Clone, Debug, Default)]
        pub struct Partition {
            /// Partition id.
            pub id: Ulid,

            /// Partition Range
            pub range: Option<Range<usize>>,
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
            pub fn with_range(mut self, range: Range<usize>) -> impl RangedPartition {
                self.range = Some(range);
                self
            }
        }

        pub trait RangedPartition {
            fn route(&self);
            fn get_ranges() -> usize;
        }

        impl RangedPartition for Partition {
            fn route(&self) {}

            fn get_ranges() -> usize {
                0
            }
        }

        let partition_one = Partition::new().with_range(0..12);
    }
}
