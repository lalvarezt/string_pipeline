use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use string_pipeline::Template;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("process_simple", |b| {
        b.iter(|| {
            Template::parse(black_box("{split:/:-1}"))
                .unwrap()
                .format(black_box("/home/user/.cargo/bin"))
                .unwrap()
        })
    });

    c.bench_function("process_complex", |b| {
        b.iter(|| {
            Template::parse(black_box(
                "{split:,:0..2|trim|prepend:num\\: |join: - |upper}",
            ))
            .unwrap()
            .format(black_box("18,   4.92, Unknown"))
            .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
