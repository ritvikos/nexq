/// Defines retention policy
#[derive(Clone, Debug, Default)]
pub struct RetentionPolicy {
    pub storage: StoragePolicy,
    pub interest: InterestPolicy,
    pub discard: DiscardPolicy,
}

/// Storage policy
#[derive(Clone, Debug)]
pub enum StoragePolicy {
    /// In-Memory
    Memory(StorageConstraint),

    /// Persistent
    Persistent(StorageConstraint),
}

impl Default for StoragePolicy {
    fn default() -> Self {
        Self::Memory(StorageConstraint::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct StorageConstraint {
    pub count_limit: Option<usize>,
    pub size_limit: Option<usize>,
}

/// Interest Policy
#[derive(Clone, Debug, Default)]
pub enum InterestPolicy {
    /// Behaves as Work Queue
    /// Each message can be consumed only once
    #[default]
    Work,
}

#[derive(Clone, Debug, Default)]
pub enum DiscardPolicy {
    /// Discards old messages in the station.
    /// if it exceeds predefined limits.
    #[default]
    Old,

    /// Restricts adding new messages in the station,
    /// if it exceeds predefined limits.
    New,

    // In case of unbounded queue,
    // there're no predefined limits,
    // unless OOM [in-memory] / storage [persistence].
    /// No predefined limits on adding messages in the station.
    None,
}

impl RetentionPolicy {
    /// Creates new instance
    pub fn new() -> Self {
        Self::default()
    }
}
