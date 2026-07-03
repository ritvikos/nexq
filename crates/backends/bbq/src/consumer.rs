//! Reserve a slot, read, mark consumed - three-step dance.

use crate::{
    block::{Allocated, Block, Committed, Reserved},
    cursor::{CursorMut, CursorRef},
    slot::SlotReader,
};

pub(crate) enum PopOutcome<T> {
    Read(T),
    Done(usize),
    Unavailable,
    NoSlot,
}

enum PopState<'a, T> {
    Reserved(SlotReader<'a, T>),
    Done(usize),
    Unavailable,
    NoSlot,
}

pub(crate) struct Consumer<'a, T> {
    block: &'a Block<T>,
    slots_per_block: usize,
    allocated: CursorRef<'a>,
    committed: CursorRef<'a>,
    reserved: CursorMut<'a>,
}

impl<'a, T> Consumer<'a, T> {
    pub(crate) fn new(block: &'a Block<T>, slots_per_block: usize) -> Self {
        Self {
            block,
            slots_per_block,
            allocated: CursorRef::from(block.cursor::<Allocated>()),
            committed: CursorRef::from(block.cursor::<Committed>()),
            reserved: CursorMut::from(block.cursor::<Reserved>()),
        }
    }

    pub(crate) fn pop(&self) -> PopOutcome<T> {
        match self.reserve() {
            PopState::Reserved(reader) => PopOutcome::Read(reader.consume()),
            PopState::Done(version) => PopOutcome::Done(version),
            PopState::NoSlot => PopOutcome::NoSlot,
            PopState::Unavailable => PopOutcome::Unavailable,
        }
    }

    fn reserve(&self) -> PopState<'a, T> {
        loop {
            let reserved = self.reserved.load();
            println!("reserved offset: {}", reserved.offset());
            println!("slots_per_block: {}", self.slots_per_block);

            if reserved.offset() >= self.slots_per_block {
                return PopState::Done(reserved.version());
            }

            let committed = self.committed.load();

            // No space to reserve
            // The consumer never pass the producer
            if reserved.offset() == committed.offset() {
                return PopState::NoSlot;
            }

            // Prevent out of order commits
            if committed.offset() != self.slots_per_block {
                let allocated = self.allocated.load();

                // Wait until allocated entries are committed
                if allocated.offset() != committed.offset() {
                    return PopState::Unavailable;
                }
            }

            // Try reserve next slot
            if self.reserved.fetch_max(reserved.advance_offset()) == reserved {
                return PopState::Reserved(SlotReader::new(self.block, reserved.offset()));
            }
        }
    }
}
