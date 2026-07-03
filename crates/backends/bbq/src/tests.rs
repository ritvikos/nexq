use crate::queue::Queue;

#[test]
fn test_enqueue_dequeue() {
    const TOTAL_BLOCKS: usize = 1;
    const SLOTS_PER_BLOCK: usize = 10;

    let queue = Queue::<String, TOTAL_BLOCKS, SLOTS_PER_BLOCK>::new();

    for i in 0..TOTAL_BLOCKS * SLOTS_PER_BLOCK {
        let value = String::from(format!("value {i}"));
        queue.enqueue(value).unwrap();
    }

    for i in 0..TOTAL_BLOCKS * SLOTS_PER_BLOCK {
        let value = queue.dequeue().unwrap();
        assert_eq!(value, String::from(format!("value {i}")));
    }

    println!("{queue:#?}");
}

#[test]
fn test_wraparound_bumps_version() {
    const TOTAL_BLOCKS: usize = 1;
    const SLOTS_PER_BLOCK: usize = 10;

    let queue = Queue::<String, TOTAL_BLOCKS, SLOTS_PER_BLOCK>::new();

    for i in 0..SLOTS_PER_BLOCK {
        queue.enqueue(format!("value {i}")).unwrap();
    }
    for i in 0..SLOTS_PER_BLOCK {
        let value = queue.dequeue().unwrap();
        assert_eq!(value, format!("value {i}"));
    }

    queue.enqueue(format!("value {SLOTS_PER_BLOCK}")).unwrap();

    println!("{queue:#?}");
}
