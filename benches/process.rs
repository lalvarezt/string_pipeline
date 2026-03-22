use criterion::{Criterion, criterion_group, criterion_main};
use once_cell::sync::Lazy;
use std::hint::black_box;
use std::time::Duration;
use string_pipeline::Template;

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

const SMALL_INPUT: &str = "apple,banana,cherry,date,elderberry,fig,grape,honeydew,kiwi,lemon";
const PADDED_SMALL_INPUT: &str = " apple , banana , cherry , date , elderberry ";
const USER_RECORD: &str = "john doe admin@example.com";
static LARGE_INPUT: Lazy<String> = Lazy::new(|| SMALL_INPUT.repeat(1_000)); // ~600 KB
static LARGE_MAP_INPUT: Lazy<String> = Lazy::new(|| PADDED_SMALL_INPUT.repeat(1_000));

// -----------------------------------------------------------------------------
// 1. Parsing Benchmarks – How fast can we compile templates?
// -----------------------------------------------------------------------------

fn bench_parsing(c: &mut Criterion) {
    let cases = [
        ("single_block_simple", "{upper}"),
        ("single_block_chain", "{split:,:..|map:{trim|upper}|join:,}"),
        (
            "multi_section",
            "User: {split: :0} Email: {split: :1} Fruits: {split:,:..|sort|join:\\|}",
        ),
        (
            "complex_nested",
            "{split:,:..|filter:^[a-m]|map:{trim|upper|substring:0..3}|sort|join:,}",
        ),
        ("tv_path_last_segment", "{split:/:-1}"),
        ("tv_tabbed_display", "{split:\\t:0} ({split:\\t:2})"),
        (
            "tv_editor_command",
            "${EDITOR:-vim} '+{strip_ansi|split:\\::1}' '{strip_ansi|split:\\::0}'",
        ),
        (
            "tv_pr_command",
            "gh pr view {strip_ansi|split:#:1|split:   :0} --web",
        ),
        ("tv_display_suffix", "{} - displayed"),
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
        ("single_block_upper_small", "{upper}", SMALL_INPUT),
        ("split_join_small", "{split:,:..|join: }", SMALL_INPUT),
        ("split_join_large", "{split:,:..|join: }", &LARGE_INPUT),
        (
            "multi_section_format",
            "Name: {split: :0} Surname: {split: :1}",
            USER_RECORD,
        ),
        (
            "repeated_split_sections",
            "First: {split:,:0} Second: {split:,:1} Third: {split:,:2}",
            SMALL_INPUT,
        ),
        (
            "filter_sort",
            "{split:,:..|filter:^[a-m]|sort|join:,}",
            SMALL_INPUT,
        ),
        ("map_upper", "{split:,:..|map:{upper}|join:,}", SMALL_INPUT),
        (
            "map_trim_upper_large",
            "{split:,:..|map:{trim|upper}|join:,}",
            &LARGE_MAP_INPUT,
        ),
        (
            "complex_nested",
            "{split:,:..|filter:^[a-m]|map:{reverse|upper}|sort|join:,}",
            SMALL_INPUT,
        ),
        ("tv_path_last_segment", "{split:/:-1}", "/a/b/c/d.txt"),
        (
            "tv_tabbed_display",
            "{split:\\t:0} ({split:\\t:2})",
            "api\tnginx:latest\tUp 2 hours",
        ),
        (
            "tv_editor_command",
            "${EDITOR:-vim} '+{strip_ansi|split:\\::1}' '{strip_ansi|split:\\::0}'",
            "src/main.rs:42:fn main()",
        ),
        (
            "tv_pr_command",
            "gh pr view {strip_ansi|split:#:1|split:   :0} --web",
            "feature/add-benchmarks #123   ready",
        ),
        ("tv_display_suffix", "{} - displayed", "entry_12345"),
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
// 3. Structured inputs – multi-section formatting with per-section inputs
// -----------------------------------------------------------------------------

fn bench_structured_inputs(c: &mut Criterion) {
    let tpl = Template::parse("Users: {upper} | Files: {lower}").unwrap();
    let user_inputs = ["john doe", "jane smith", "peter parker"];
    let file_inputs = ["FILE1.TXT", "FILE2.TXT", "FILE3.TXT"];
    let inputs: [&[&str]; 2] = [&user_inputs, &file_inputs];
    let separators = [" ", ","];

    let mut group = c.benchmark_group("structured_inputs");
    group.bench_function("format_with_inputs", |b| {
        b.iter(|| {
            tpl.format_with_inputs(black_box(&inputs), black_box(&separators))
                .unwrap()
        })
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
    targets = bench_parsing, bench_execution, bench_structured_inputs
}
criterion_main!(benches);
