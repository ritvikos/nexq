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
    pub(crate) fn new<const SLOTS: usize>() -> Self {
        let mut slots = Vec::with_capacity(SLOTS);
        for _ in 0..SLOTS {
            slots.push(Slot::new());
        }

        Self {
            slots: slots.into_boxed_slice(),
            allocated: RawCursor::new(0, SLOTS),
            committed: RawCursor::new(0, SLOTS),
            reserved: RawCursor::new(0, SLOTS),
            consumed: RawCursor::new(0, SLOTS),
        }
    }

    pub(crate) fn as_producer(&self) -> Producer<'_, T> {
        Producer::new(self)
    }

    pub(crate) fn as_consumer(&self) -> Consumer<'_, T> {
        Consumer::new(self)
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
