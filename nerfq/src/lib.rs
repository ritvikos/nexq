extern crate dashmap;

pub mod message;
pub mod topic;

use dashmap::DashMap;
use topic::Topic;

type TopicName = String;

pub struct Nerf {
    pub topic: DashMap<TopicName, Topic>,
}
