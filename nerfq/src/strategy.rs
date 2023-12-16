use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub enum Strategy {
    RoundRobin { idx: AtomicUsize },
    HashBased { hasher: Hasher, key: String },
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
            Self::RoundRobin { idx } => Self::RoundRobin {
                idx: AtomicUsize::new(idx.load(Ordering::Acquire)),
            },

            Self::HashBased { hasher, key } => Self::HashBased {
                hasher: hasher.clone(),
                key: key.clone(),
            },
        }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::RoundRobin {
            idx: AtomicUsize::default(),
        }
    }
}

impl Strategy {
    pub fn rotate(&self, len: &usize) -> usize {
        match self {
            Self::RoundRobin { idx } => idx.fetch_add(1, Ordering::AcqRel) % len,
            Self::HashBased { hasher, key } => todo!(),
        }
    }
}
