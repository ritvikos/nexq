/// Handle config for partitions.
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Max number of partitions
    pub max_count: Option<usize>,

    /// Rate Limit
    pub rate_limit: bool,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of partitions.
    pub fn with_max_count(mut self, max_count: Option<usize>) -> Self {
        self.max_count = max_count;
        self
    }

    /// Set rate limiting.
    pub fn rate_limit(mut self, rate_limit: bool) -> Self {
        self.rate_limit = rate_limit;
        self
    }
}

// TODO: Create a dedicated rate limiter.
