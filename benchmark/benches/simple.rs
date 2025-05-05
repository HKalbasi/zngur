use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build_vec_by_push 10_000 rust", |b| {
        b.iter(|| benchmark::build_vec_by_push_rust(black_box(10_000)))
    });
    c.bench_function("build_vec_by_push 10_000 cpp", |b| {
        b.iter(|| benchmark::build_vec_by_push_cpp(black_box(10_000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
