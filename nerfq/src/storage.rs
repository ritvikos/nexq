use crate::queue::Queue;

/// Storage
#[derive(Clone, Debug)]
pub enum Storage {
    /// In-Memory
    Memory(Queue),

    /// Persistent
    Persistent(),
}

impl Default for Storage {
    fn default() -> Self {
        Self::Memory(Queue::default())
    }
}
