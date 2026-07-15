//! Packed (version, offset) cursor

use core::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_utils::CachePadded;

/// Packed representation of version + offset in a single `usize`.
///
/// Structure:
/// +----------------+-------------------+
/// |    Version     |      Offset       |
/// +----------------+-------------------+
/// |  upper 32 bits |  lower 32 bits    |
/// +----------------+-------------------+
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct PackedCursor(usize);

impl PackedCursor {
    // HACK: assume 64-bit - stopgap - works on my machine.
    // TODO: replace w/ paper's approach.
    const VERSION_BIT_LEN: usize = 32;
    const OFFSET_BIT_LEN: usize = usize::BITS as usize - Self::VERSION_BIT_LEN;
    const OFFSET_MASK: usize = (1 << Self::OFFSET_BIT_LEN) - 1;

    pub(crate) fn new(version: usize, offset: usize) -> Self {
        Self((version << Self::OFFSET_BIT_LEN) | (offset & Self::OFFSET_MASK))
    }

    pub(crate) fn raw(self) -> usize {
        self.0
    }

    pub(crate) fn version(self) -> usize {
        self.0 >> Self::OFFSET_BIT_LEN
    }

    pub(crate) fn offset(self) -> usize {
        self.0 & Self::OFFSET_MASK
    }

    /// Return a new cursor w/ same version but different offset.
    pub(crate) fn with_offset(self, offset: usize) -> Self {
        Self::new(self.version(), offset)
    }

    /// Increment the offset by one w/ same version.
    pub(crate) fn advance_offset(self) -> Self {
        self.with_offset(self.offset() + 1)
    }

    /// Bump the version by one and reset the offset to zero.
    pub(crate) fn next_version(self) -> Self {
        Self::new(self.version() + 1, 0)
    }
}

impl From<usize> for PackedCursor {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub(crate) struct RawCursor(CachePadded<AtomicUsize>);

impl core::fmt::Debug for RawCursor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let cursor = self.load();
        f.debug_struct("RawCursor")
            .field("raw", &cursor.0)
            .field("version", &cursor.version())
            .field("offset", &cursor.offset())
            .finish()
    }
}

impl RawCursor {
    pub(crate) fn new(version: usize, offset: usize) -> Self {
        Self(CachePadded::new(AtomicUsize::new(
            PackedCursor::new(version, offset).0,
        )))
    }

    #[inline]
    pub(crate) fn advance_unit(&self) -> usize {
        self.0.fetch_add(1, Ordering::Release)
    }

    #[inline]
    pub(crate) fn store(&self, value: usize) {
        self.0.store(value, Ordering::Release);
    }

    #[inline]
    pub(crate) fn load(&self) -> PackedCursor {
        PackedCursor(self.0.load(Ordering::Acquire))
    }

    // Atomically set to `max(current, val)`. Returns the previous value.
    pub(crate) fn fetch_max(&self, value: PackedCursor) -> PackedCursor {
        let mut current = self.load();
        loop {
            if current >= value {
                return current;
            }
            match self.0.compare_exchange_weak(
                current.raw(),
                value.raw(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(prev) => return PackedCursor(prev),
                Err(actual) => current = PackedCursor(actual),
            }
        }
    }
}

pub(crate) struct CursorRef<'a>(&'a RawCursor);
impl<'a> CursorRef<'a> {
    pub(crate) fn load(&self) -> PackedCursor {
        self.0.load()
    }
}

impl<'a> From<&'a RawCursor> for CursorRef<'a> {
    fn from(cursor: &'a RawCursor) -> Self {
        Self(cursor)
    }
}

pub(crate) struct CursorMut<'a>(&'a RawCursor);
impl<'a> CursorMut<'a> {
    pub(crate) fn load(&self) -> PackedCursor {
        self.0.load()
    }

    pub(crate) fn fetch_max(&self, value: PackedCursor) -> PackedCursor {
        self.0.fetch_max(value)
    }

    pub(crate) fn advance_unit(&self) -> usize {
        self.0.advance_unit()
    }
}

impl<'a> From<&'a RawCursor> for CursorMut<'a> {
    fn from(cursor: &'a RawCursor) -> Self {
        Self(cursor)
    }
}
