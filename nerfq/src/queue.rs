extern crate crossbeam_queue;

use crate::message::Message;
use crossbeam_queue::{ArrayQueue, SegQueue};
use std::cell::UnsafeCell;

#[derive(Debug)]
pub enum Queue {
    /// Bounded Queue
    Bounded(UnsafeCell<Box<ArrayQueue<Message>>>),

    /// Unbounded Queue
    Unbounded(UnsafeCell<SegQueue<Message>>),
}

impl Clone for Queue {
    fn clone(&self) -> Self {
        match self {
            Self::Bounded(_) => Self::Bounded(UnsafeCell::new(Box::new(ArrayQueue::new(1000)))),
            Self::Unbounded(_) => Self::Unbounded(UnsafeCell::new(SegQueue::new())),
        }
    }
}

impl Default for Queue {
    fn default() -> Self {
        Queue::Unbounded(UnsafeCell::new(SegQueue::default()))
    }
}

impl Queue {
    /// Creates a new queue instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to push an element into the queue
    pub fn push(&mut self, message: Message) -> Result<(), Box<Message>> {
        match self {
            Queue::Bounded(queue) => unsafe { (*queue.get()).push(message) },
            Queue::Unbounded(queue) => {
                unsafe {
                    (*queue.get()).push(message);
                }
                Ok(())
            }
        }?;

        Ok(())
    }

    /// Attempts to pop an element from the queue.
    pub fn pop(&mut self) -> Option<Message> {
        match self {
            Queue::Bounded(queue) => unsafe { (*queue.get()).pop() },
            Queue::Unbounded(queue) => unsafe { (*queue.get()).pop() },
        }?;
        None
    }

    /// Returns the number of elements in the queue.
    pub fn len(&mut self) -> usize {
        match self {
            Queue::Bounded(queue) => unsafe { (*queue.get()).len() },
            Queue::Unbounded(queue) => unsafe { (*queue.get()).len() },
        }
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&mut self) -> bool {
        match self {
            Queue::Bounded(queue) => unsafe { (*queue.get()).is_empty() },
            Queue::Unbounded(queue) => unsafe { (*queue.get()).is_empty() },
        }
    }
}
