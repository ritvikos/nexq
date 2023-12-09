use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct Error {
    /// Kind of error
    pub kind: Kind,
}

impl Error {
    /// Create a new instance.
    pub(crate) fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            Kind::Partition(reason) => write!(f, "{}", reason),
        }
    }
}

/// Defines the error type.
#[derive(Debug)]
pub enum Kind {
    /// Occurs when unable to create partitions.
    Partition(PartitionError),
}

#[derive(Debug)]
pub enum PartitionError {
    /// Occurs when the maximum partition limit is reached,
    /// preventing further creation.
    MaxCount,
}

impl StdError for PartitionError {}

impl fmt::Display for PartitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartitionError::MaxCount => {
                write!(f, "Max partition count limit reached.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Kind, PartitionError};

    fn create_partition() -> Result<(), Error> {
        Err(Error::new(Kind::Partition(PartitionError::MaxCount)))
    }

    #[test]
    fn test_custom_error() {
        if let Err(err) = create_partition() {
            eprintln!("Error creating partition: {}", err);
        }
    }
}
