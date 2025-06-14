use criterion::{Criterion, criterion_group, criterion_main};
use once_cell::sync::Lazy;
use std::hint::black_box;
use std::time::Duration;
use string_pipeline::Template;

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

const SMALL_INPUT: &str = "apple,banana,cherry,date,elderberry,fig,grape,honeydew,kiwi,lemon";
static LARGE_INPUT: Lazy<String> = Lazy::new(|| SMALL_INPUT.repeat(1_000)); // ~600 KB

// -----------------------------------------------------------------------------
// 1. Parsing Benchmarks – How fast can we compile templates?
// -----------------------------------------------------------------------------

fn bench_parsing(c: &mut Criterion) {
    let cases = [
        ("simple", "{upper}"),
        ("medium", "{split:,:..|join: }"),
        (
            "complex",
            "{split:,:..|filter:^[a-m]|map:{trim|upper|substring:0..3}|sort|join:,}",
        ),
        ("nested_map", "{split:,:..|map:{split:_:..|reverse}|join: }"),
    ];

    let mut group = c.benchmark_group("template_parsing");
    for (name, tpl) in cases {
        group.bench_function(name, |b| {
            b.iter(|| Template::parse(black_box(tpl)).unwrap())
        });
    }
    group.finish();
}

// -----------------------------------------------------------------------------
// 2. Execution Benchmarks – Runtime performance of compiled templates
// -----------------------------------------------------------------------------

fn bench_execution(c: &mut Criterion) {
    // (id, template, input)
    let cases = [
        ("split_join_small", "{split:,:..|join: }", SMALL_INPUT),
        ("split_join_large", "{split:,:..|join: }", &LARGE_INPUT),
        (
            "filter_sort",
            "{split:,:..|filter:^[a-m]|sort|join:,}",
            SMALL_INPUT,
        ),
        ("map_upper", "{split:,:..|map:{upper}|join:,}", SMALL_INPUT),
        (
            "complex_nested",
            "{split:,:..|filter:^[a-m]|map:{reverse|upper}|sort|join:,}",
            SMALL_INPUT,
        ),
    ];

    let mut group = c.benchmark_group("template_execution");
    for (name, tpl_str, input) in cases {
        // Compile once outside the measurement loop
        let tpl = Template::parse(tpl_str).unwrap();
        group.bench_function(name, |b| b.iter(|| tpl.format(black_box(input)).unwrap()));
    }
    group.finish();
}

// -----------------------------------------------------------------------------
// 3. Cache Effectiveness – First run vs subsequent runs
// -----------------------------------------------------------------------------

fn bench_caching(c: &mut Criterion) {
    let tpl_str = "{split:,:..|filter:a|join:,}";
    let tpl = Template::parse(tpl_str).unwrap();

    let mut group = c.benchmark_group("cache_effect");

    // Measure first-call cost (cold caches)
    group.bench_function("first_call", |b| {
        b.iter(|| tpl.format(black_box(SMALL_INPUT)).unwrap())
    });

    // Warm the caches once
    let _ = tpl.format(SMALL_INPUT).unwrap();

    // Measure subsequent calls (hot caches)
    group.bench_function("subsequent_calls", |b| {
        b.iter(|| tpl.format(black_box(SMALL_INPUT)).unwrap())
    });

    group.finish();
}

// -----------------------------------------------------------------------------
// Criterion configuration & entry point
// -----------------------------------------------------------------------------

criterion_group! {
    name = benches;
    config = Criterion::default()
        .configure_from_args()
        .sample_size(200)
        .measurement_time(Duration::from_secs(5));
    targets = bench_parsing, bench_execution, bench_caching
}
criterion_main!(benches);
