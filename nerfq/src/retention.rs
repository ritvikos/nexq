/// Defines retention policy
#[derive(Clone, Debug, Default)]
pub struct RetentionPolicy {
    pub storage: StoragePolicy,
    pub interest: InterestPolicy,
    pub discard: DiscardPolicy,
}

/// Storage policy
#[derive(Clone, Debug, Default)]
pub enum StoragePolicy {
    /// In-Memory
    #[default]
    Memory,

    /// Persistent
    Persistent,
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
    /// Discards old messages in the Station
    /// if it exceeds predefined limits.
    #[default]
    Old,

    /// Restricts adding new messages in the Station
    /// if it exceeds predefined limits.
    New,
}

impl RetentionPolicy {
    /// Creates new instance
    pub fn new() -> Self {
        Self::default()
    }
}
