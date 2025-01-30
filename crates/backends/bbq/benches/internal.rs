use bbq::QueueInner;

use criterion::{black_box, criterion_group, criterion_main};
use criterion::{AxisScale, Criterion, PlotConfiguration};

const fn max_u32(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn add_function<F, M>(group: &mut criterion::BenchmarkGroup<M>, id: impl Into<String>, mut f: F)
where
    F: FnMut(Vec<String>) -> (),
    M: criterion::measurement::Measurement,
{
    let n = 10_000;
    let values: Vec<String> = (1..=n).map(|i| format!("string value {}", i)).collect();
    group.bench_function(id, |b| {
        b.iter(|| {
            f(black_box(values.clone()));
        })
    });
}

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("single-thread-multi-string");

    group.sample_size(60);
    group.measurement_time(std::time::Duration::from_secs(10));
    group.throughput(criterion::Throughput::Elements(10_000));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    const TOTAL_BLOCKS: usize = 100;
    const ENTRIES: usize = 10_000;

    const SHIFT: u32 = max_u32(
        usize::BITS - ENTRIES.leading_zeros() + 1,
        usize::BITS - TOTAL_BLOCKS.leading_zeros(),
    );

    let queue = QueueInner::<String>::new(String::from(""), TOTAL_BLOCKS, ENTRIES);
    add_function(&mut group, "benchmark-enqueue", |values| {
        for value in values {
            queue.enqueue(value, SHIFT).unwrap();
        }
        // for _ in 0..10_000 {
        //     queue.dequeue().unwrap();
        // }
    });

    // Vec Benchmark (for comparison)
    let mut v = Vec::<String>::with_capacity(10_000);
    add_function(&mut group, "vec-push", |values| {
        for value in values {
            v.push(value);
        }
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = criterion_benchmark
}
criterion_main!(benches);
