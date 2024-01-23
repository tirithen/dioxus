#![allow(unused)]
use generational_box::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create<S: Storage<u32>>() -> GenerationalBox<u32, S> {
    GenerationalBox::new(0)
}

fn set_read<S: Storage<u32>>(signal: GenerationalBox<u32, S>) -> u32 {
    signal.set(1);
    *signal.read()
}

fn bench_storage<S>(c: &mut Criterion, names: (&str, &str)) {
    // Bench reading a signal
    let signal = create::<UnsyncStorage>();
    c.bench_function(names.1, |b| b.iter(|| set_read(black_box(signal))));

    // Bench creating a signal
    c.bench_function(names.0, |b| b.iter(|| black_box(create::<UnsyncStorage>())));
}

fn bench_fib(c: &mut Criterion) {
    bench_storage::<SyncStorage>(c, ("create_unsync", "set_read_unsync"));
    bench_storage::<UnsyncStorage>(c, ("create_sync", "set_read_sync"));
}

criterion_group!(benches, bench_fib);
criterion_main!(benches);
