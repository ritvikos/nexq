use crate::queue::Queue;

/// Storage
#[derive(Clone, Debug)]
pub enum Store {
    /// In-Memory
    Memory(Queue),

    /// Persistent
    Persistent(),
}

impl Default for Store {
    fn default() -> Self {
        Self::Memory(Queue::default())
    }
}
