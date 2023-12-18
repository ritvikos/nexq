use nanorand::{Rng, WyRand};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub enum Strategy {
    RoundRobin(AtomicUsize),
    HashBased(Hasher),
    Random,
}

#[derive(Debug, Clone)]
pub enum Hasher {
    Murmur3,
    Md5,
    XXhash,
}

impl Clone for Strategy {
    fn clone(&self) -> Self {
        match self {
            Self::RoundRobin(idx) => {
                Self::RoundRobin(AtomicUsize::new(idx.load(Ordering::Acquire))).clone()
            }
            Self::HashBased(hasher) => Self::HashBased(hasher.clone()),
            Self::Random => self.clone(),
        }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::RoundRobin(AtomicUsize::default())
    }
}

impl Strategy {
    pub fn rotate(&self, len: usize) -> usize {
        match self {
            Self::RoundRobin(idx) => idx.fetch_add(1, Ordering::AcqRel) % len,
            Self::HashBased(hasher) => todo!(),
            Self::Random => WyRand::new().generate_range(0..=len),
        }
    }
}
