//! Tie the producer/consumer cursors to the block-ring.
//! [`advance_producer`] and [`advance_consumer`] enforce paper's invariant:
//! a producer never advances into a block that consumer hasn't fully drained.

use crate::{
    block::{Allocated, Block, Committed, Consumed, Reserved},
    consumer::PopOutcome,
    cursor::{PackedCursor, RawCursor},
    errors::{DequeueError, EnqueueError, QueueError},
    producer::PushOutcome,
};
use core::{fmt::Debug, marker::PhantomData};

enum AdvanceResult {
    Success,
    NoSlot,
    Unavailable,
}

// state transition:
// producer: allocated -> committed
// consumer: reserved -> consumed
pub struct Queue<T, const TOTAL_BLOCKS: usize, const SLOTS_PER_BLOCK: usize> {
    producer: RawCursor,
    consumer: RawCursor,
    blocks: Box<[Block<T>]>,
    _marker: PhantomData<T>,
}

impl<T, const TOTAL_BLOCKS: usize, const SLOTS_PER_BLOCK: usize>
    Queue<T, TOTAL_BLOCKS, SLOTS_PER_BLOCK>
{
    pub fn new() -> Self {
        let _: () = assert!(TOTAL_BLOCKS > 0, "`TOTAL_BLOCKS` must be greater than 0");
        let _: () = assert!(
            SLOTS_PER_BLOCK > 0,
            "`SLOTS_PER_BLOCK` must be greater than 0"
        );

        let blocks: Box<[Block<T>; TOTAL_BLOCKS]> = Box::new(core::array::from_fn(|idx| {
            let block = Block::new(SLOTS_PER_BLOCK);

            if idx == 0 {
                block.cursor::<Allocated>().store(0);
                block.cursor::<Committed>().store(0);
                block.cursor::<Reserved>().store(0);
                block.cursor::<Consumed>().store(0);
            }

            block
        }));

        Self {
            producer: RawCursor::new(0, 0),
            consumer: RawCursor::new(0, 0),
            blocks,
            _marker: PhantomData,
        }
    }
}

impl<T, const TOTAL_BLOCKS: usize, const SLOTS_PER_BLOCK: usize> Debug
    for Queue<T, TOTAL_BLOCKS, SLOTS_PER_BLOCK>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue")
            .field("producer", &self.producer)
            .field("consumer", &self.consumer)
            .finish()
    }
}

impl<T, const TOTAL_BLOCKS: usize, const SLOTS_PER_BLOCK: usize>
    Queue<T, TOTAL_BLOCKS, SLOTS_PER_BLOCK>
{
    pub fn enqueue(&self, value: T) -> Result<(), QueueError> {
        let mut value = value;
        loop {
            let head = self.producer.load();
            let block = unsafe { self.blocks.get_unchecked(head.offset()) };
            let producer = block.as_producer(SLOTS_PER_BLOCK);

            match producer.push(value) {
                PushOutcome::Written => return Ok(()),
                PushOutcome::Full {
                    rejected,
                    version: _,
                } => match self.advance_producer(head) {
                    AdvanceResult::Unavailable => {
                        return Err(QueueError::Enqueue(EnqueueError::Unavailable))
                    }
                    AdvanceResult::NoSlot => return Err(QueueError::Enqueue(EnqueueError::NoSlot)),
                    AdvanceResult::Success => {
                        value = rejected;
                        continue;
                    }
                },
            }
        }
    }

    pub fn dequeue(&self) -> Result<T, QueueError> {
        loop {
            let head = self.consumer.load();
            let block = unsafe { self.blocks.get_unchecked(head.offset()) };
            let consumer = block.as_consumer(SLOTS_PER_BLOCK);

            match consumer.pop() {
                PopOutcome::Read(value) => return Ok(value),
                PopOutcome::NoSlot | PopOutcome::Unavailable => {
                    return Err(QueueError::Dequeue(DequeueError::Empty));
                }
                PopOutcome::Done(version) => {
                    if self.advance_consumer(head, version) {
                        continue;
                    }
                    return Err(QueueError::Dequeue(DequeueError::Empty));
                }
            }
        }
    }

    fn advance_producer(&self, head: PackedCursor) -> AdvanceResult {
        let next_idx = (head.offset() + 1) % TOTAL_BLOCKS;

        // SAFETY: 'next' is always within bounds due to wrap-around w/ modulo.
        let nblock = unsafe { self.blocks.get_unchecked(next_idx) };
        let consumed = nblock.cursor::<Consumed>().load();

        // INVARIANT: producer must never advance into a block that still has unconsumed data
        if consumed.version() < head.version()
            || (consumed.version() == head.version() && consumed.offset() != SLOTS_PER_BLOCK)
        {
            let reserved = nblock.cursor::<Reserved>().load();
            if reserved.offset() == consumed.offset() {
                return AdvanceResult::NoSlot;
            }
            return AdvanceResult::Unavailable;
        }

        nblock.cursor::<Committed>().fetch_max(head.next_version());
        nblock.cursor::<Allocated>().fetch_max(head.next_version());

        let next_offset = head.offset() + 1;
        let new_head = if next_offset == TOTAL_BLOCKS {
            PackedCursor::new(head.version() + 1, 0)
        } else {
            head.with_offset(next_offset)
        };

        self.producer.fetch_max(new_head);

        AdvanceResult::Success
    }

    fn advance_consumer(&self, head: PackedCursor, _vsn: usize) -> bool {
        let next = (head.offset() + 1) % TOTAL_BLOCKS;
        let nblock = unsafe { self.blocks.get_unchecked(next) };

        let committed = nblock.cursor::<Committed>().load();
        if committed.version() != head.version() + 1 {
            return false;
        }

        let reset = PackedCursor::new(head.version() + 1, 0);
        nblock.cursor::<Consumed>().fetch_max(reset);
        nblock.cursor::<Reserved>().fetch_max(reset);

        // increment version and wrap-around offset, if needed
        let new_head = if head.offset() + 1 == TOTAL_BLOCKS {
            PackedCursor::new(head.version() + 1, 0)
        } else {
            head.with_offset(head.offset() + 1)
        };

        self.consumer.fetch_max(new_head);
        true
    }
}
