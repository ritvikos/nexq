/// Message key.
#[derive(Clone, Debug)]
pub enum Key {
    Hash(String),
}

impl Default for Key {
    fn default() -> Self {
        Self::Hash(String::new())
    }
}

impl Key {
    /// Create new instance.
    pub fn new() -> Self {
        Self::default()
    }
}

pub trait ToKey {
    fn to_key(self) -> Key;
}

impl ToKey for String {
    fn to_key(self) -> Key {
        Key::Hash(self)
    }
}
