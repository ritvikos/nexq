use crate::block::{Block, Committed, Consumed};
use core::{cell::UnsafeCell, mem::MaybeUninit};

pub(crate) struct Slot<T>(UnsafeCell<MaybeUninit<T>>);

impl<T> Slot<T> {
    pub(crate) fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    #[inline]
    pub(crate) unsafe fn read(&self) -> T {
        (*self.0.get()).assume_init_read()
    }

    // SAFETY: The caller must ensure the slot is empty with exclusive access.
    #[inline]
    pub(crate) unsafe fn write(&self, value: T) {
        (*self.0.get()).write(value);
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Send> Sync for Slot<T> {}

pub(crate) struct SlotWriter<'a, T> {
    block: &'a Block<T>,
    offset: usize,
}

impl<'a, T> SlotWriter<'a, T> {
    pub(crate) fn new(block: &'a Block<T>, offset: usize) -> Self {
        Self { block, offset }
    }

    pub(crate) fn commit(self, value: T) -> SlotWriter<'a, T> {
        unsafe { self.block.write_unchecked(self.offset, value) };
        self.block.cursor::<Committed>().advance_unit();
        let writer = SlotWriter::new(self.block, self.offset);
        core::mem::forget(self);
        writer
    }
}

impl<'a, T> Drop for SlotWriter<'a, T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        panic!("`SlotWriter` dropped w/o `SlotWriter::commit`");
    }
}

pub(crate) struct SlotReader<'a, T> {
    block: &'a Block<T>,
    offset: usize,
}

impl<'a, T> SlotReader<'a, T> {
    pub(crate) fn new(block: &'a Block<T>, offset: usize) -> Self {
        Self { block, offset }
    }

    pub(crate) fn consume(self) -> T {
        let value = unsafe { self.block.read_unchecked(self.offset) };
        self.block.cursor::<Consumed>().advance_unit();
        core::mem::forget(self);
        value
    }
}

impl<'a, T> Drop for SlotReader<'a, T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        panic!("`SlotReader` dropped w/o `SlotReader::consume`");
    }
}
