use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use string_pipeline::process;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("process_simple", |b| {
        b.iter(|| {
            process(
                black_box("/home/user/.cargo/bin"),
                // output: "bin"
                black_box("{split:/:-1}"),
            )
            .unwrap()
        })
    });

    c.bench_function("process_complex", |b| {
        b.iter(|| {
            process(
                black_box(" 18,   4.92, Unknown"),
                // output: "NUM: 18 - NUM: 4.92"
                black_box("{split:,:0..2|trim|prepend:num\\: |join: - |upper}"),
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
