extern crate dashmap;

pub mod message;
pub mod queue;
pub mod retention;
pub mod stations;

use stations::Stations;

#[derive(Clone)]
pub struct NerfBroker {
    /// Station name and metadata
    pub stations: Stations,
}
