extern crate ulid;

pub mod config;
pub mod manager;
pub mod metadata;
pub mod state;

use crate::storage::Store;
use ulid::Ulid;

#[derive(Clone, Debug, Default)]
pub struct Partition {
    /// Partition id
    pub id: Ulid,

    /// Number of replicas
    pub replicas: usize,

    /// Underlying storage
    pub store: Store,
}

impl Partition {
    /// Create new instance.
    pub fn new() -> Self {
        Self {
            id: Ulid::new(),
            ..Default::default()
        }
    }

    /// Set the storage for partition.
    pub fn with_store(mut self, store: Store) -> Self {
        self.store = store;
        self
    }
}
