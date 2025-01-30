use core::{
    alloc::Layout,
    cell::UnsafeCell,
    fmt::Debug,
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crossbeam_utils::CachePadded;
use thiserror::Error;

pub struct QueueInner<T: Clone + Debug + Default> {
    producer: RawHead,
    consumer: RawHead,
    blocks: Box<[Block<T>]>,

    total_blocks: usize,
    total_entries: usize,
    shift: u32,

    _marker: PhantomData<T>,
}

impl<T: Clone + Debug + Default> QueueInner<T> {
    pub fn new(default: T, total_blocks: usize, total_entries: usize) -> Self {
        assert!(total_blocks > 0, "`TOTAL_BLOCKS` must be greater than 0");
        assert!(total_entries > 0, "`TOTAL_ENTRIES` must be greater than 0");

        let layout =
            Layout::array::<Block<T>>(total_blocks).expect("Failed to calculate memory layout");

        let mut blocks = unsafe {
            let ptr = std::alloc::alloc(layout) as *mut Block<T>;

            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            Box::from_raw(core::slice::from_raw_parts_mut(ptr, total_blocks))
        };

        Self::init(&mut blocks, default, total_blocks, total_entries);

        let shift = max_u32(
            usize::BITS - total_entries.leading_zeros() + 1,
            usize::BITS - total_blocks.leading_zeros(),
        );

        Self {
            producer: RawHead::new(),
            consumer: RawHead::new(),
            blocks,
            total_blocks,
            total_entries,
            shift,
            _marker: PhantomData,
        }
    }

    fn init(blocks: &mut [Block<T>], default: T, total_blocks: usize, entries: usize) {
        (0..total_blocks).for_each(|idx| {
            let block = Block::new(default.clone(), entries);

            if idx == 0 {
                block.cursor.allocated.set(0);
                block.cursor.committed.set(0);
                block.cursor.reserved.set(0);
                block.cursor.consumed.set(0);
            }

            unsafe { ptr::write(&mut blocks[idx], block) }
        });
    }
}

impl<T: Clone + Debug + Default> Debug for QueueInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueInner")
            .field("producer", &self.producer)
            .field("consumer", &self.consumer)
            .field("blocks", &self.blocks.iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<T: Clone + Debug + Default> QueueInner<T> {
    pub fn enqueue(&self, value: T, shift: u32) -> Result<(), QueueError> {
        loop {
            let head = self.producer.load_unpack(shift);
            let block = unsafe { self.blocks.get_unchecked(head.index) };

            match block.push(self.total_entries, self.shift) {
                State::Allocated(index) => {
                    block.commit(index, value);
                    return Ok(());
                }
                State::Full(_) => match self.advance_producer(head) {
                    State::Unavailable => todo!(),
                    State::NoSlot => todo!(),
                    State::Success => continue,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }

    fn dequeue(&self) -> Result<T, QueueError> {
        loop {
            let head = self.consumer.load_unpack(self.shift);
            let block = unsafe { self.blocks.get_unchecked(head.index) };

            match block.pop(self.total_entries, self.shift) {
                State::Reserved(cursor) => {
                    let output = block.consume(cursor.offset);
                    return Ok(output);

                    // if output != T::default() {
                    //     return Ok(output);
                    // }

                    // continue;
                }
                State::NoSlot => todo!(),
                State::Unavailable => todo!(),
                State::Full(version) => {
                    // if self.advance_consumer(head) {
                    //     continue;
                    // }
                    // return
                }
                _ => unreachable!(),
            }
        }
    }

    fn advance_producer(&self, mut head: Head) -> State {
        let next = (head.index + 1) % self.total_blocks;

        let nblock = unsafe { self.blocks.get_unchecked(next) };
        let consumed = nblock.load(CursorState::Consumed, self.shift);

        if consumed.version < head.version
            || (consumed.version == head.version && consumed.offset != self.total_entries)
        {
            let reserved = nblock.load(CursorState::Reserved, self.shift);
            if reserved.offset == consumed.offset {
                return State::NoSlot;
            }
            return State::Unavailable;
        }

        let committed = Cursor::new(head.version + 1, 0).pack(self.shift);
        nblock.cursor.committed.max(committed);

        let allocated = Cursor::new(head.version + 1, 0).pack(self.shift);
        nblock.cursor.allocated.max(allocated);

        head.index = (head.index + 1) % self.total_blocks;
        if head.index == 0 {
            head.version += 1
        }

        self.producer.max(head.pack(self.shift));

        return State::Success;
    }

    // TODO: Last operation
    // fn advance_consumer(&self, head: Head) -> bool {
    //     let nblock = unsafe { self.blocks.get_unchecked((head.index + 1) % TOTAL_BLOCKS) };
    //     let committed = nblock.cursor.

    //     todo!()
    // }
}

// TODO: Cleanup
// - Maybe rename trait to [`CursorOps`]
// - Add behavior of struct [`Cursor`] into it
// - Return its 'trait' instead of 'struct' in functions
trait Advanceable {
    fn new(value: usize) -> Self;
    fn advance_one(&self) -> usize;

    // The [`set`] method is for initialization until a better workaround is used.
    fn set(&self, value: usize);
}

#[derive(Clone, Debug)]
struct Block<T: Clone + Debug + Default> {
    cursor: Cursors<RawCursor>,
    slots: Box<[Slot<T>]>,
}

impl<T: Clone + Debug + Default> Block<T> {
    // TODO: Use [`T::default()`] instead of `default` arg.
    fn new(default: T, entries: usize) -> Self {
        let slots = (0..entries)
            .map(|_| Slot::new_with(default.clone()))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            cursor: Cursors::new(entries),
            slots,
        }
    }

    fn push(&self, total_entries: usize, shift: u32) -> State {
        let current = self.cursor.allocated.load_unpack(shift);

        if current.offset >= total_entries {
            return State::Full(current.version);
        }

        let raw = self.advance(CursorState::Allocated);
        let current = Cursor::from(raw, shift);

        if current.offset >= total_entries {
            return State::Full(current.version);
        }

        State::Allocated(current.offset)
    }

    fn pop(&self, entries: usize, shift: u32) -> State {
        // TODO: Refactor
        // Instead of:
        // - `self.cursor.reserved.load(Ordering::Acquire)`
        //
        // do this (introduce abstraction):
        // - `self.load(cursor_state).operation()` or
        // - `self.load(cursor_state)` || `self.load_unpack(cursor_state)`

        loop {
            let reserved_raw = self.cursor.reserved.load(Ordering::Acquire);
            let reserved = self.cursor.reserved.unpack(reserved_raw, shift);

            if reserved.offset < entries {
                let committed = self.cursor.committed.load_unpack(shift);

                if reserved.offset == committed.offset {
                    return State::NoSlot;
                }

                if committed.offset != entries {
                    let allocated = self.cursor.allocated.load_unpack(shift);
                    if allocated.offset != committed.offset {
                        return State::Unavailable;
                    }
                }

                if self.cursor.reserved.max(reserved_raw + 1) == reserved_raw {
                    return State::Reserved(reserved);
                }

                continue;
            }

            return State::Full(reserved.version);
        }
    }

    #[inline]
    fn commit(&self, offset: usize, value: T) {
        unsafe { self.write_unchecked(offset, value) };
        self.advance(CursorState::Committed);
    }

    #[inline]
    fn consume(&self, offset: usize) -> T {
        let value = unsafe { self.read_unchecked(offset) };
        self.advance(CursorState::Consumed);
        value
    }

    #[inline]
    unsafe fn read_unchecked(&self, offset: usize) -> T {
        let slot = unsafe { self.slots.get_unchecked(offset) };
        unsafe { slot.read() }
    }

    #[inline]
    unsafe fn write_unchecked(&self, index: usize, value: T) {
        let slot = unsafe { self.slots.get_unchecked(index) };
        unsafe { slot.write(value) };
    }

    #[inline]
    fn advance(&self, state: CursorState) -> usize {
        self.raw_cursor(state).advance_one()
    }

    #[inline]
    fn load(&self, state: CursorState, shift: u32) -> Cursor {
        self.raw_cursor(state).load_unpack(shift)
    }

    // TODO: Maybe return [`Advanceable`]?
    #[inline]
    fn raw_cursor(&self, state: CursorState) -> &RawCursor {
        match state {
            CursorState::Allocated => &self.cursor.allocated,
            CursorState::Committed => &self.cursor.committed,
            CursorState::Reserved => &self.cursor.reserved,
            CursorState::Consumed => &self.cursor.consumed,
        }
    }
}

struct Slot<T: Clone + Debug + Default>(UnsafeCell<T>);

impl<T: Clone + Debug + Default> Clone for Slot<T> {
    fn clone(&self) -> Self {
        // SAFETY: We are only reading the value, so this is safe as long as
        // no mutable references are created elsewhere.
        let value = unsafe { &*self.0.get() }.clone();
        Slot(UnsafeCell::new(value))
    }
}

impl<T: Clone + Debug + Default> Slot<T> {
    // fn new() -> Self {
    //     Self(UnsafeCell::new(T))
    // }

    fn new_with(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    #[inline]
    unsafe fn read(&self) -> T {
        self.0.get().replace(T::default())
    }

    // SAFETY: The caller must ensure the slot is empty with exclusive access.
    #[inline]
    unsafe fn write(&self, value: T) {
        self.0.get().write(value);
    }

    pub const fn as_ptr(&self) -> *const T {
        self as *const _ as *const T
    }
}

impl<T: Clone + Debug + Default> Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = unsafe { (*self.0.get()).clone() };
        f.debug_struct("Slot").field("value", &value).finish()
    }
}

unsafe impl<T: Clone + Debug + Default + Send> Send for Slot<T> {}
unsafe impl<T: Clone + Debug + Default + Send> Sync for Slot<T> {}

const fn max_u32(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

/// 2-bits for index and version:
/// `index`: [`Block`] on which the queue is currently working on.
/// `version`: Prevent ABA.
///
/// Structure:
/// +----------------+-------------------+
/// |    Version     |      Index        |
/// +----------------+-------------------+
///
/// Rest of the bits are unused.
#[derive(Debug)]
struct RawHead(CachePadded<AtomicUsize>);

impl RawHead {
    #[inline]
    fn new() -> Self {
        Self::from(0)
    }

    /// Loads the raw data and unpacks into [`Head`].
    fn load_unpack(&self, shift: u32) -> Head {
        let raw = self.load(Ordering::Acquire);
        Head::from(raw, shift)
    }

    #[inline]
    fn load(&self, ordering: Ordering) -> usize {
        self.0.load(ordering)
    }

    #[inline]
    fn store(&self, value: usize, ordering: Ordering) {
        self.0.store(value, ordering);
    }

    fn max(&self, head: usize) -> usize {
        let mut ret = 0;
        while let Err(_) = self
            .0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                ret = old;
                Some(core::cmp::max(head, old))
            })
        {}
        ret
    }
}

impl From<usize> for RawHead {
    fn from(value: usize) -> Self {
        Self(CachePadded::new(AtomicUsize::new(value)))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Head {
    version: usize,
    index: usize,
}

impl Head {
    fn pack(&self, shift: u32) -> usize {
        self.version << shift | self.index
    }

    fn from(value: usize, shift: u32) -> Self {
        Self {
            version: value >> shift,
            index: (value & !(usize::MAX << shift)),
        }
    }
}

#[derive(Clone, Debug)]
struct Cursors<T: Clone + Advanceable> {
    allocated: T,
    committed: T,
    reserved: T,
    consumed: T,
}

impl<T: Clone + Advanceable> Cursors<T> {
    fn new(value: usize) -> Self {
        Self {
            allocated: T::new(value),
            committed: T::new(value),
            reserved: T::new(value),
            consumed: T::new(value),
        }
    }

    // #[inline]
    // fn load(&self, state: CursorState) -> Cursor {
    //     let t = self.inner(state);
    //     todo!()
    // }

    // #[inline]
    // fn inner(&self, state: CursorState) -> &T {
    //     match state {
    //         CursorState::Allocated => &self.allocated,
    //         CursorState::Committed => &self.committed,
    //         CursorState::Reserved => &self.reserved,
    //         CursorState::Consumed => &self.consumed,
    //     }
    // }
}

/// 2-bits for offset and version:
/// `offset`: Position within block. \
/// `version`: Prevent ABA.
///
/// Structure:
/// +----------------+-------------------+
/// |    Version     |      Offset       |
/// +----------------+-------------------+
///
/// Rest of the bits are unused.
///
/// Initially, idx and off in the first block are zero and for
/// remaining blocks off is set to BLOCK_SIZE. The initial value
/// of vsn will be introduced in Sec. 4.2.2.
#[derive(Debug)]
struct RawCursor(CachePadded<AtomicUsize>);

impl Clone for RawCursor {
    fn clone(&self) -> Self {
        let value = self.0.load(Ordering::Relaxed);
        Self::new(value)
    }
}

impl RawCursor {
    // TODO: Refactor API with trait and state.
    /// Loads the raw data and unpacks into [`Head`].
    fn load_unpack(&self, shift: u32) -> Cursor {
        let raw = self.load(Ordering::Acquire);
        self.unpack(raw, shift)
    }

    #[inline]
    fn load(&self, ordering: Ordering) -> usize {
        self.0.load(ordering)
    }

    #[inline]
    fn unpack(&self, raw: usize, shift: u32) -> Cursor {
        Cursor::from(raw, shift)
    }

    #[inline]
    fn advance(&self, count: usize) -> usize {
        self.0.fetch_add(count, Ordering::AcqRel)
    }

    fn max(&self, head: usize) -> usize {
        let mut ret = 0;
        while let Err(_) = self
            .0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                ret = old;
                Some(core::cmp::max(head, old))
            })
        {}
        ret
    }
}

impl Advanceable for RawCursor {
    #[inline]
    fn new(value: usize) -> Self {
        Self::from(value)
    }

    fn advance_one(&self) -> usize {
        self.advance(1)
    }

    fn set(&self, value: usize) {
        self.0.store(value, Ordering::Release);
    }
}

impl From<usize> for RawCursor {
    fn from(value: usize) -> Self {
        Self(CachePadded::new(AtomicUsize::new(value)))
    }
}

struct Cursor {
    version: usize,
    offset: usize,
}

impl Cursor {
    #[inline]
    fn new(version: usize, offset: usize) -> Self {
        Self { version, offset }
    }

    #[inline]
    fn pack(&self, shift: u32) -> usize {
        self.version << shift | self.offset
    }

    #[inline]
    fn from(value: usize, shift: u32) -> Self {
        Self {
            version: value >> shift,
            offset: (value & !(usize::MAX << shift)),
        }
    }
}

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Enqueue error: {0}")]
    Enqueue(#[from] EnqueueError),

    #[error("Dequeue error: {0}")]
    Dequeue(#[from] DequeueError),
}

#[derive(Error, Debug)]
pub enum EnqueueError {
    #[error("Failed to enqueue: the block is full.")]
    Full(usize),

    #[error("Failed to enqueue: no slot available.")]
    Unavailable,

    #[error("Fail to enqueue: No remaining slots.")]
    NoSlot,
}

#[derive(Error, Debug)]
pub enum DequeueError {
    #[error("Failed to dequeue: the queue is empty.")]
    Empty,
}

#[derive(Error, Debug)]
pub enum BlockError {
    #[error("Block is full.")]
    Full,
}

// / 4 valid states:
// / - Allocated: The producer reserved a slot to write.
// / - Committed: The producer has written data to the slot.
// / - Reserved: Slot reserved by consumer to read.
// / - Consumed: A consumer processed the data and slot is free for reuse.
//
// State Transition:
// Producer: Allocated -> Committed
// Consumer: Committed -> Reserved -> Consumed
#[derive(Clone, Copy, Debug)]
enum CursorState {
    Allocated,
    Committed,
    Reserved,
    Consumed,
}

enum State {
    Allocated(usize),
    Full(usize),
    Unavailable,
    Success,
    NoSlot,
    Reserved(Cursor),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{max_u32, Head, QueueInner, RawHead};

    const TOTAL_BLOCKS: usize = 100_000;
    const ENTRIES: usize = 10_000;

    const SHIFT: u32 = max_u32(
        usize::BITS - ENTRIES.leading_zeros() + 1,
        usize::BITS - TOTAL_BLOCKS.leading_zeros(),
    );

    #[test]
    fn test_queue_head() {
        let raw = RawHead::from(TOTAL_BLOCKS);
        let head = raw.load_unpack(SHIFT);

        assert_eq!(
            head,
            Head {
                version: 0,
                index: 100_000,
            }
        );
    }

    #[test]
    fn test_queue_enqueue() {
        const TOTAL_BLOCKS: usize = 10;
        const ENTRIES: usize = 10;

        let queue = QueueInner::<usize>::new(0, TOTAL_BLOCKS, ENTRIES);

        (0..21).for_each(|idx| queue.enqueue(idx, SHIFT).unwrap());

        println!("queue: {queue:#?}");
    }

    #[test]
    fn test_queue_enqueue_concurrent() {
        const TOTAL_BLOCKS: usize = 10;
        const ENTRIES: usize = 100;
        const THREADS: usize = 8;
        const VALUES_PER_THREAD: usize = 13;

        let mut handles = vec![];

        let queue = Arc::new(QueueInner::<String>::new(
            String::from(""),
            TOTAL_BLOCKS,
            ENTRIES,
        ));
        println!("first block cursor: {:?}", queue.blocks[0].cursor);
        println!("subsequent block cursor: {:?}", queue.blocks[1].cursor);

        (1..=THREADS).for_each(|_| {
            let queue = Arc::clone(&queue);

            let handle = std::thread::spawn(move || {
                (1..=VALUES_PER_THREAD).for_each(|i| {
                    let value = String::from(format!("value {i}"));
                    queue.enqueue(value, SHIFT).unwrap();
                });
            });

            handles.push(handle);
        });

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
