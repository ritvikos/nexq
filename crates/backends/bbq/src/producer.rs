use crate::{
    block::{Allocated, Block},
    cursor::{CursorMut, PackedCursor},
    slot::SlotWriter,
};

pub(crate) enum PushOutcome<T> {
    Written,
    Full { rejected: T, version: usize },
}

enum PushState<'a, T> {
    Allocated(SlotWriter<'a, T>),
    Full(usize),
}

pub(crate) struct Producer<'a, T> {
    block: &'a Block<T>,
    slots_per_block: usize,
    allocated: CursorMut<'a>,
}

impl<'a, T> Producer<'a, T> {
    pub(crate) fn new(block: &'a Block<T>, slots_per_block: usize) -> Self {
        Self {
            block,
            slots_per_block,
            allocated: CursorMut::from(block.cursor::<Allocated>()),
        }
    }

    pub(crate) fn push(&self, value: T) -> PushOutcome<T> {
        match self.alloc() {
            PushState::Allocated(writer) => {
                let committed = writer.commit(value);
                core::mem::forget(committed);
                PushOutcome::Written
            }
            PushState::Full(version) => PushOutcome::Full {
                rejected: value,
                version,
            },
        }
    }

    fn alloc(&self) -> PushState<'a, T> {
        let current = self.allocated.load();

        // Prevent FAA overflow
        if current.offset() >= self.slots_per_block {
            return PushState::Full(current.version());
        }

        let raw = self.allocated.advance_unit();
        let current = PackedCursor::from(raw);

        if current.offset() >= self.slots_per_block {
            return PushState::Full(current.version());
        }

        PushState::Allocated(SlotWriter::new(self.block, current.offset()))
    }
}
