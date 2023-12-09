use crate::stations::Store;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct Partitions {
    /// Partitions
    pub partitions: Vec<Partition>,

    /// Number of partitions
    pub count: AtomicUsize,
}

impl Clone for Partitions {
    fn clone(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            count: AtomicUsize::new(self.count.load(Ordering::Acquire)),
        }
    }
}

impl Partitions {
    /// Creates new instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets number of partitions
    pub fn with_count(self, count: usize) -> Self {
        self.count.fetch_add(count, Ordering::AcqRel);
        self
    }

    /// Inserts partition with custom configuration if specified,
    /// defaults to a standard partition if not specified
    pub fn insert(&mut self, partition: Option<Partition>) {
        match partition {
            Some(partition) => self.insert_inner(partition),
            None => self.insert_inner(Partition::default()),
        }
    }

    /// Insert partition
    fn insert_inner(&mut self, partition: Partition) {
        self.partitions.push(partition);
    }
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    /// Partition id
    pub id: String,

    /// Underlying storage
    pub store: Store,
}

impl Partition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store(mut self, store: Store) -> Self {
        self.store = store;
        self
    }
}
