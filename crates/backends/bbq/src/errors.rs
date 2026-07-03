use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("failed to enqueue: {0}")]
    Enqueue(#[from] EnqueueError),

    #[error("failed to dequeue: {0}")]
    Dequeue(#[from] DequeueError),
}

#[derive(Error, Debug)]
pub enum EnqueueError {
    #[error("no slot available")]
    Unavailable,

    #[error("no remaining slots")]
    NoSlot,
}

#[derive(Error, Debug)]
pub enum DequeueError {
    #[error("queue is empty")]
    Empty,
}
