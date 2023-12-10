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
            Kind::Station(reason) => write!(f, "{}", reason),
        }
    }
}

/// Defines the error type.
#[derive(Debug, PartialEq)]
pub enum Kind {
    /// Occurs when unable to create stations.
    Station(StationError),

    /// Occurs when unable to create partitions.
    Partition(PartitionError),
}

#[derive(Debug, PartialEq)]
pub enum StationError {
    /// Occurs when the maximum stations limit is reached,
    /// preventing further creation.
    MaxCount,

    /// Occurs when the station already exists with same name,
    /// preventing further creation.
    AlreadyExists,
}

impl StdError for StationError {}

impl fmt::Display for StationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StationError::MaxCount => {
                write!(f, "Max station count limit reached.")
            }

            StationError::AlreadyExists => {
                write!(f, "Station already exists.")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
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
    use super::{Error, Kind, PartitionError, StationError};

    // -----
    // Station errors
    // -----

    fn station_already_exists() -> Result<(), Error> {
        Err(Error::new(Kind::Station(StationError::AlreadyExists)))
    }

    fn station_max_count() -> Result<(), Error> {
        Err(Error::new(Kind::Station(StationError::MaxCount)))
    }

    #[test]
    fn test_station_errors() {
        if let Err(err) = station_already_exists() {
            assert_eq!(Kind::Station(StationError::AlreadyExists), err.kind);
        }

        if let Err(err) = station_max_count() {
            assert_eq!(Kind::Station(StationError::MaxCount), err.kind);
        }
    }

    // -----
    // Partition errors
    // -----

    fn partition_max_count() -> Result<(), Error> {
        Err(Error::new(Kind::Partition(PartitionError::MaxCount)))
    }

    #[test]
    fn test_partition_errors() {
        if let Err(err) = partition_max_count() {
            assert_eq!(Kind::Partition(PartitionError::MaxCount), err.kind)
        }
    }
}
