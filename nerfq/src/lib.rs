extern crate dashmap;

pub mod error;
pub mod message;
pub mod partition;
pub mod queue;
pub mod retention;
pub mod station;
pub mod storage;

use station::Stations;

#[derive(Clone, Debug)]
pub struct NerfBroker {
    /// Station name and metadata
    pub stations: Stations,
}

#[cfg(test)]
mod tests {
    use crate::{
        queue::Queue,
        retention::RetentionPolicy,
        station::{Station, Stations},
        NerfBroker,
    };
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    // const THREAD_COUNT: usize = 16;
    const ITERATIONS_PER_THREAD: usize = 10000;

    #[test]
    fn test_broker_custom() {
        let retention_policy_one = RetentionPolicy {
            ..Default::default()
        };

        let queue_one = Queue::new();

        let station_one = Station::new()
            .with_name("station_one".into())
            .with_retention_policy(retention_policy_one)
            .with_queue(queue_one);

        let retention_policy_two = RetentionPolicy {
            ..Default::default()
        };

        let queue_two = Queue::new();

        let station_two = Station::new()
            .with_name("station_two".into())
            .with_retention_policy(retention_policy_two)
            .with_queue(queue_two);

        let stations = Stations::new();
        stations.insert(station_one).unwrap();
        stations.insert(station_two).unwrap();

        let broker = NerfBroker { stations };
        println!("{broker:#?}");
    }

    #[test]
    fn test_broker_insert_multiple_stations_standalone() {
        let broker = NerfBroker {
            stations: Stations::new(),
        };

        // single threaded
        (0..ITERATIONS_PER_THREAD).for_each(|i| {
            let station = Station::new().with_name(format!("station{}", i));
            broker.stations.insert(station).unwrap();
        });

        let expected = ITERATIONS_PER_THREAD;
        assert_eq!(expected, broker.stations.count());
    }

    #[test]
    fn test_broker_insert_multiple_stations_concurrently_atomic() {
        let mut handles = Vec::new();
        let thread_count = 10;
        let iterations_per_thread = 5;

        let broker = NerfBroker {
            stations: Stations::new(),
        };

        (0..thread_count).for_each(|_| {
            let broke = broker.clone();
            let handle = thread::spawn(move || {
                for j in 0..iterations_per_thread {
                    let name = format!("station{}", j);
                    let station = Station::new().with_name(name.clone());
                    broke.stations.insert(station).unwrap();
                    broke.stations.remove(name).unwrap();
                }
            });
            handles.push(handle);
        });

        for handle in handles {
            handle.join().unwrap();
        }

        let expected = 0;
        assert_eq!(expected, broker.stations.count());
    }

    #[test]
    fn test_broker_insert_multiple_stations_concurrently_arc() {
        let mut handles = vec![];

        let broker = NerfBroker {
            stations: Stations::new(),
        };

        let broker_arc = Arc::new(Mutex::new(broker));

        (0..10000).for_each(|i| {
            let broker = Arc::clone(&broker_arc);

            let handle = thread::spawn(move || {
                // broker.stations.insert(station).unwrap();
                let broke = broker.lock().unwrap();
                let station = Station::new().with_name(format!("station{}", i));
                broke.stations.insert(station).unwrap();
            });
            handles.push(handle);
        });

        for handle in handles {
            handle.join().unwrap();
        }

        let expected = 10000;
        assert_eq!(expected, broker_arc.lock().unwrap().stations.count());
    }
}
