//! Task spawning benchmarks

use criterion::{criterion_group, criterion_main, Criterion};
use cycle::prelude::*;

fn bench_task_spawn(c: &mut Criterion) {
    c.bench_function("task_spawn", |b| {
        b.iter(|| {
            let _handle = cycle::spawn(async {
                // Minimal task
            });
        });
    });
}

criterion_group!(benches, bench_task_spawn);
criterion_main!(benches);
