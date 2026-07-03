//! Slot-ring + four-cursors: alloc → commit, reserve → consume.

use crate::{consumer::Consumer, cursor::RawCursor, producer::Producer, slot::Slot};

pub(crate) struct Block<T> {
    slots: Box<[Slot<T>]>,
    allocated: RawCursor,
    committed: RawCursor,
    reserved: RawCursor,
    consumed: RawCursor,
}

impl<T> Block<T> {
    pub(crate) fn new(slots_per_block: usize) -> Self {
        let mut slots = Vec::with_capacity(slots_per_block);
        for _ in 0..slots_per_block {
            slots.push(Slot::new());
        }

        Self {
            slots: slots.into_boxed_slice(),
            allocated: RawCursor::new(0, slots_per_block),
            committed: RawCursor::new(0, slots_per_block),
            reserved: RawCursor::new(0, slots_per_block),
            consumed: RawCursor::new(0, slots_per_block),
        }
    }

    pub(crate) fn as_producer(&self, slots_per_block: usize) -> Producer<'_, T> {
        Producer::new(self, slots_per_block)
    }

    pub(crate) fn as_consumer(&self, slots_per_block: usize) -> Consumer<'_, T> {
        Consumer::new(self, slots_per_block)
    }

    #[inline]
    pub(crate) unsafe fn read_unchecked(&self, offset: usize) -> T {
        let slot = unsafe { self.slots.get_unchecked(offset) };
        unsafe { slot.read() }
    }

    #[inline]
    pub(crate) unsafe fn write_unchecked(&self, index: usize, value: T) {
        let slot = unsafe { self.slots.get_unchecked(index) };
        unsafe { slot.write(value) };
    }

    pub(crate) fn cursor<F: CursorField>(&self) -> &RawCursor {
        F::select(self)
    }
}

pub(crate) trait CursorField {
    fn select<T>(block: &Block<T>) -> &RawCursor;
}

macro_rules! cursor_field {
    ($name:ident, $field:ident) => {
        pub(crate) struct $name;
        impl CursorField for $name {
            fn select<T>(block: &Block<T>) -> &RawCursor {
                &block.$field
            }
        }
    };
}

cursor_field!(Allocated, allocated);
cursor_field!(Committed, committed);
cursor_field!(Reserved, reserved);
cursor_field!(Consumed, consumed);
