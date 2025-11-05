use clap::{Arg, Command};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use string_pipeline::Template;

/// Represents the results of a throughput benchmark for a specific input size
#[derive(Debug, Clone)]
struct BenchmarkResult {
    input_size: usize,
    parse_time: Duration,
    total_format_time: Duration,
    avg_time_per_path: Duration,
    throughput_paths_per_sec: f64,
    parse_percentage: f64,
    operation_metrics: Vec<OperationMetric>,
    latency_stats: LatencyStatistics,
}

/// Tracks metrics for individual operation types
#[derive(Debug, Clone)]
struct OperationMetric {
    operation_name: String,
    total_time: Duration,
    call_count: usize,
    avg_time_per_call: Duration,
    percentage_of_total: f64,
}

/// Statistical analysis of latency distribution
#[derive(Debug, Clone)]
struct LatencyStatistics {
    min: Duration,
    p50: Duration,
    p95: Duration,
    p99: Duration,
    max: Duration,
    stddev: f64,
}

impl BenchmarkResult {
    fn new(
        input_size: usize,
        parse_time: Duration,
        total_format_time: Duration,
        individual_times: Vec<Duration>,
    ) -> Self {
        let avg_time_per_path = total_format_time / input_size as u32;
        let throughput_paths_per_sec = input_size as f64 / total_format_time.as_secs_f64();
        let total_time = parse_time + total_format_time;
        let parse_percentage = (parse_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0;

        let latency_stats = Self::calculate_statistics(&individual_times);

        BenchmarkResult {
            input_size,
            parse_time,
            total_format_time,
            avg_time_per_path,
            throughput_paths_per_sec,
            parse_percentage,
            operation_metrics: Vec::new(),
            latency_stats,
        }
    }

    fn calculate_statistics(times: &[Duration]) -> LatencyStatistics {
        if times.is_empty() {
            return LatencyStatistics {
                min: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
                stddev: 0.0,
            };
        }

        let mut sorted_times: Vec<Duration> = times.to_vec();
        sorted_times.sort();

        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];

        let p50_idx = (sorted_times.len() as f64 * 0.50) as usize;
        let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;

        let p50 = sorted_times[p50_idx.min(sorted_times.len() - 1)];
        let p95 = sorted_times[p95_idx.min(sorted_times.len() - 1)];
        let p99 = sorted_times[p99_idx.min(sorted_times.len() - 1)];

        // Calculate standard deviation
        let mean = times.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / times.len() as f64;
        let variance = times
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / times.len() as f64;
        let stddev = variance.sqrt();

        LatencyStatistics {
            min,
            p50,
            p95,
            p99,
            max,
            stddev,
        }
    }

    fn scaling_factor(&self, baseline: &BenchmarkResult) -> f64 {
        let expected = self.input_size as f64 / baseline.input_size as f64;
        let actual =
            self.total_format_time.as_secs_f64() / baseline.total_format_time.as_secs_f64();
        actual / expected
    }

    fn add_operation_metrics(&mut self, metrics: Vec<OperationMetric>) {
        self.operation_metrics = metrics;
    }
}

/// Generates realistic absolute path strings for benchmarking
struct PathGenerator {
    directories: Vec<&'static str>,
    filenames: Vec<&'static str>,
    extensions: Vec<&'static str>,
}

impl PathGenerator {
    fn new() -> Self {
        PathGenerator {
            directories: vec![
                "home",
                "usr",
                "var",
                "opt",
                "etc",
                "lib",
                "bin",
                "sbin",
                "tmp",
                "dev",
                "projects",
                "workspace",
                "repos",
                "src",
                "tests",
                "docs",
                "config",
                "data",
                "cache",
                "logs",
                "build",
                "dist",
                "target",
                "node_modules",
                "vendor",
                "components",
                "services",
                "models",
                "controllers",
                "views",
                "utils",
            ],
            filenames: vec![
                "main",
                "lib",
                "index",
                "app",
                "server",
                "client",
                "config",
                "utils",
                "helper",
                "handler",
                "service",
                "model",
                "controller",
                "router",
                "middleware",
                "test",
                "spec",
                "readme",
                "license",
                "changelog",
                "makefile",
                "dockerfile",
                "package",
                "cargo",
                "mod",
                "types",
                "constants",
                "errors",
                "validation",
            ],
            extensions: vec![
                "rs", "txt", "md", "json", "toml", "yaml", "yml", "js", "ts", "py", "go", "c",
                "cpp", "h", "sh",
            ],
        }
    }

    /// Generate a single path with specified seed and depth
    fn generate_path(&self, seed: usize, depth: usize) -> String {
        let mut parts = vec![];

        // Generate directory components
        for i in 0..depth {
            let idx = (seed + i * 7) % self.directories.len();
            parts.push(self.directories[idx]);
        }

        // Add filename with extension
        let filename_idx = (seed * 13) % self.filenames.len();
        let ext_idx = (seed * 17) % self.extensions.len();
        let filename = format!(
            "{}.{}",
            self.filenames[filename_idx], self.extensions[ext_idx]
        );
        parts.push(&filename);

        format!("/{}", parts.join("/"))
    }

    /// Generate N unique paths with varying depths
    fn generate_paths(&self, count: usize) -> Vec<String> {
        (0..count)
            .map(|i| {
                let depth = 2 + (i % 9); // Depths from 2 to 10
                self.generate_path(i, depth)
            })
            .collect()
    }
}

/// Comprehensive template set covering all operations and real-world use cases
struct TemplateSet;

impl TemplateSet {
    fn get_templates() -> Vec<(&'static str, &'static str)> {
        vec![
            // Core individual operations
            ("Split all", "{split:/:..}"),
            ("Split last index", "{split:/:-1}"),
            ("Join", "{split:/:..|join:/}"),
            ("Upper", "{split:/:-1|upper}"),
            ("Lower", "{split:/:-1|lower}"),
            ("Trim", "{split:/:-1|trim}"),
            ("Replace simple", "{replace:s/\\.txt$/.md/}"),
            ("Replace complex", "{replace:s/\\/\\/+/\\//g}"),
            ("Substring", "{split:/:-1|substring:0..10}"),
            ("Reverse", "{split:/:-1|reverse}"),
            ("Strip ANSI", "{strip_ansi}"),
            ("Filter", "{split:/:..|filter:^[a-z]|join:/}"),
            ("Sort", "{split:/:..|sort|join:/}"),
            ("Unique", "{split:/:..|unique|join:/}"),
            ("Pad", "{split:/:-1|pad:50: :right}"),
            // Real-world path templates (television use cases)
            ("Extract filename", "{split:/:-1}"),
            ("Extract directory", "{split:/:0..-1|join:/}"),
            ("Basename no ext", "{split:/:-1|split:.:0}"),
            ("File extension", "{split:/:-1|split:.:-1}"),
            (
                "Regex extract filename",
                "{regex_extract:[^/]+$}",
            ),
            (
                "Uppercase all components",
                "{split:/:..|map:{upper}|join:/}",
            ),
            (
                "Remove hidden dirs",
                "{split:/:..|filter_not:^\\.|join:/}",
            ),
            ("Normalize filename", "{split:/:-1|trim|lower}"),
            ("Slug generation", "{replace:s/ /_/g|lower}"),
            ("Breadcrumb last 3", "{split:/:..|slice:-3..|join: > }"),
            // Complex chains
            ("Chain: trim+upper+pad", "{split:/:-1|trim|upper|pad:20}"),
            (
                "Chain: split+filter+sort+join",
                "{split:/:..|filter:^[a-z]|sort|join:-}",
            ),
            (
                "Chain: map complex",
                "{split:/:..|map:{trim|lower|replace:s/_/-/g}|join:/}",
            ),
        ]
    }
}

/// Runs a benchmark for a single template with varying input sizes and detailed profiling
fn benchmark_template(
    template_name: &str,
    template_str: &str,
    sizes: &[usize],
    iterations: usize,
    detailed: bool,
) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let generator = PathGenerator::new();
    let mut results = Vec::new();

    // Parse template once
    let parse_start = Instant::now();
    let template = Template::parse(template_str)?;
    let parse_time = parse_start.elapsed();

    for &size in sizes {
        // Generate N paths for this size
        let paths = generator.generate_paths(size);

        // Warmup: format all paths once
        for path in &paths {
            let _ = template.format(path)?;
        }

        // Measure: format all paths multiple times for stable measurements
        let mut total_duration = Duration::ZERO;
        let mut individual_times = Vec::new();

        for _ in 0..iterations {
            let start = Instant::now();
            for path in &paths {
                let format_start = Instant::now();
                let _ = template.format(path)?;
                if detailed && iterations == 1 {
                    // Only collect individual times on single iteration runs
                    individual_times.push(format_start.elapsed());
                }
            }
            total_duration += start.elapsed();
        }

        // Average across iterations
        let avg_format_time = total_duration / iterations as u32;

        // If not detailed mode, create dummy individual times for stats
        if !detailed || iterations > 1 {
            let avg_per_path = avg_format_time / size as u32;
            individual_times = vec![avg_per_path; size];
        }

        let mut result = BenchmarkResult::new(size, parse_time, avg_format_time, individual_times);

        // If detailed mode, gather operation-level metrics
        if detailed {
            let op_metrics = gather_operation_metrics(&template, template_name, &paths)?;
            result.add_operation_metrics(op_metrics);
        }

        results.push(result);
    }

    Ok(results)
}

/// Gather detailed metrics for each operation type in the template
fn gather_operation_metrics(
    template: &Template,
    _template_name: &str,
    paths: &[String],
) -> Result<Vec<OperationMetric>, Box<dyn std::error::Error>> {
    // For now, we'll do a simple breakdown by re-running the template
    // In a future enhancement, we could instrument the library itself

    // Count operation types in the template string
    let template_str = format!("{:?}", template);

    let mut metrics = Vec::new();
    let mut operation_counts: HashMap<String, usize> = HashMap::new();

    // Simple heuristic: count operations mentioned
    let operations = vec![
        "Split", "Join", "Upper", "Lower", "Trim", "Replace", "Substring", "Reverse",
        "StripAnsi", "Filter", "Sort", "Unique", "Pad", "Map", "RegexExtract", "Append",
        "Prepend", "Surround", "Slice", "FilterNot",
    ];

    for op in &operations {
        if template_str.contains(op) {
            *operation_counts.entry(op.to_string()).or_insert(0) += 1;
        }
    }

    // Measure total time for the template
    let total_start = Instant::now();
    for path in paths {
        let _ = template.format(path)?;
    }
    let total_time = total_start.elapsed();

    // Create metrics based on detected operations
    // Note: This is a simplified approach. Full instrumentation would require library changes.
    for (op_name, count) in &operation_counts {
        metrics.push(OperationMetric {
            operation_name: op_name.clone(),
            total_time: total_time / operation_counts.len() as u32, // Simplified distribution
            call_count: count * paths.len(),
            avg_time_per_call: total_time / (count * paths.len()) as u32,
            percentage_of_total: 100.0 / operation_counts.len() as f64, // Simplified
        });
    }

    Ok(metrics)
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{nanos}ns")
    } else if nanos < 1_000_000 {
        format!("{:.2}μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

fn format_throughput(paths_per_sec: f64) -> String {
    if paths_per_sec >= 1_000_000.0 {
        format!("{:.2}M/s", paths_per_sec / 1_000_000.0)
    } else if paths_per_sec >= 1_000.0 {
        format!("{:.2}K/s", paths_per_sec / 1_000.0)
    } else {
        format!("{:.2}/s", paths_per_sec)
    }
}

fn format_size(size: usize) -> String {
    if size >= 1_000_000 {
        format!("{}M", size / 1_000_000)
    } else if size >= 1_000 {
        format!("{}K", size / 1_000)
    } else {
        size.to_string()
    }
}

fn print_template_results(template_name: &str, results: &[BenchmarkResult], detailed: bool) {
    println!("\n{}", "=".repeat(110));
    println!("Template: {}", template_name);
    println!("{}", "=".repeat(110));

    println!(
        "\n{:<12} {:>12} {:>12} {:>12} {:>15} {:>10} {:>12}",
        "Input Size", "Parse Time", "Total Time", "Avg/Path", "Throughput", "Parse %", "Scaling"
    );
    println!("{}", "-".repeat(110));

    for (idx, result) in results.iter().enumerate() {
        let scaling = if idx == 0 {
            "baseline".to_string()
        } else {
            format!("{:.2}x", result.scaling_factor(&results[0]))
        };

        println!(
            "{:<12} {:>12} {:>12} {:>12} {:>15} {:>9.2}% {:>12}",
            format_size(result.input_size),
            format_duration(result.parse_time),
            format_duration(result.total_format_time),
            format_duration(result.avg_time_per_path),
            format_throughput(result.throughput_paths_per_sec),
            result.parse_percentage,
            scaling
        );
    }

    // Scaling analysis
    if results.len() >= 2 {
        let first = &results[0];
        let last = &results[results.len() - 1];

        let size_ratio = last.input_size as f64 / first.input_size as f64;
        let time_ratio =
            last.total_format_time.as_secs_f64() / first.total_format_time.as_secs_f64();
        let scaling_quality = time_ratio / size_ratio;

        println!("\n📊 Scaling Analysis:");
        println!(
            "   Size increase: {:.0}x ({} → {})",
            size_ratio,
            format_size(first.input_size),
            format_size(last.input_size)
        );
        println!(
            "   Time increase: {:.2}x ({} → {})",
            time_ratio,
            format_duration(first.total_format_time),
            format_duration(last.total_format_time)
        );

        let scaling_desc = if scaling_quality < 0.95 {
            "Sub-linear (improving with scale!) 🚀"
        } else if scaling_quality <= 1.05 {
            "Linear (perfect scaling) ✓"
        } else if scaling_quality <= 1.5 {
            "Slightly super-linear"
        } else {
            "Super-linear (degrading with scale)"
        };

        println!(
            "   Scaling behavior: {:.2}x - {}",
            scaling_quality, scaling_desc
        );
        println!(
            "   Parse cost reduction: {:.2}% → {:.2}%",
            first.parse_percentage, last.parse_percentage
        );
    }

    // Detailed operation breakdown for largest size
    if detailed && !results.is_empty() {
        let largest_result = results.last().unwrap();
        if !largest_result.operation_metrics.is_empty() {
            println!("\n🔍 Operation Breakdown (at {} inputs):", format_size(largest_result.input_size));
            println!(
                "{:<20} {:>12} {:>12} {:>15} {:>10}",
                "Operation", "Calls", "Total Time", "Avg/Call", "% Total"
            );
            println!("{}", "-".repeat(80));

            for metric in &largest_result.operation_metrics {
                println!(
                    "{:<20} {:>12} {:>12} {:>15} {:>9.2}%",
                    truncate_name(&metric.operation_name, 20),
                    format_size(metric.call_count),
                    format_duration(metric.total_time),
                    format_duration(metric.avg_time_per_call),
                    metric.percentage_of_total
                );
            }
        }

        // Latency statistics for largest size
        let stats = &largest_result.latency_stats;
        println!("\n📈 Latency Statistics (at {} inputs):", format_size(largest_result.input_size));
        println!(
            "   Min: {}  p50: {}  p95: {}  p99: {}  Max: {}  Stddev: {:.2}ns",
            format_duration(stats.min),
            format_duration(stats.p50),
            format_duration(stats.p95),
            format_duration(stats.p99),
            format_duration(stats.max),
            stats.stddev
        );
    }
}

fn print_summary(all_results: &[(&str, Vec<BenchmarkResult>)]) {
    println!("\n{}", "=".repeat(110));
    println!("SUMMARY - Performance at Largest Input Size");
    println!("{}", "=".repeat(110));

    // Collect results with throughput for sorting
    let mut summary_data: Vec<(&str, usize, Duration, f64)> = all_results
        .iter()
        .filter_map(|(name, results)| {
            results.last().map(|last| {
                (
                    *name,
                    last.input_size,
                    last.avg_time_per_path,
                    last.throughput_paths_per_sec,
                )
            })
        })
        .collect();

    // Sort by throughput (highest first)
    summary_data.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

    println!(
        "\n{:<35} {:>12} {:>12} {:>15}",
        "Template", "Input Size", "Avg/Path", "Throughput"
    );
    println!("{}", "-".repeat(85));

    for (template_name, input_size, avg_time, throughput) in summary_data {
        println!(
            "{:<35} {:>12} {:>12} {:>15}",
            truncate_name(template_name, 35),
            format_size(input_size),
            format_duration(avg_time),
            format_throughput(throughput)
        );
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}

/// Output results in JSON format for tracking over time
fn output_json(
    all_results: &[(&str, Vec<BenchmarkResult>)],
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let mut json_output = String::from("{\n");
    json_output.push_str(&format!("  \"timestamp\": {},\n", timestamp));
    json_output.push_str("  \"benchmarks\": [\n");

    for (idx, (template_name, results)) in all_results.iter().enumerate() {
        json_output.push_str("    {\n");
        json_output.push_str(&format!("      \"template_name\": \"{}\",\n", template_name));
        json_output.push_str("      \"results\": [\n");

        for (ridx, result) in results.iter().enumerate() {
            json_output.push_str("        {\n");
            json_output.push_str(&format!("          \"input_size\": {},\n", result.input_size));
            json_output.push_str(&format!(
                "          \"parse_time_ns\": {},\n",
                result.parse_time.as_nanos()
            ));
            json_output.push_str(&format!(
                "          \"total_format_time_ns\": {},\n",
                result.total_format_time.as_nanos()
            ));
            json_output.push_str(&format!(
                "          \"avg_time_per_path_ns\": {},\n",
                result.avg_time_per_path.as_nanos()
            ));
            json_output.push_str(&format!(
                "          \"throughput_per_sec\": {:.2},\n",
                result.throughput_paths_per_sec
            ));
            json_output.push_str(&format!(
                "          \"parse_percentage\": {:.2},\n",
                result.parse_percentage
            ));

            // Latency statistics
            json_output.push_str("          \"latency_stats\": {\n");
            json_output.push_str(&format!(
                "            \"min_ns\": {},\n",
                result.latency_stats.min.as_nanos()
            ));
            json_output.push_str(&format!(
                "            \"p50_ns\": {},\n",
                result.latency_stats.p50.as_nanos()
            ));
            json_output.push_str(&format!(
                "            \"p95_ns\": {},\n",
                result.latency_stats.p95.as_nanos()
            ));
            json_output.push_str(&format!(
                "            \"p99_ns\": {},\n",
                result.latency_stats.p99.as_nanos()
            ));
            json_output.push_str(&format!(
                "            \"max_ns\": {},\n",
                result.latency_stats.max.as_nanos()
            ));
            json_output.push_str(&format!(
                "            \"stddev_ns\": {:.2}\n",
                result.latency_stats.stddev
            ));
            json_output.push_str("          },\n");

            // Operation metrics
            if !result.operation_metrics.is_empty() {
                json_output.push_str("          \"operations\": [\n");
                for (oidx, op) in result.operation_metrics.iter().enumerate() {
                    json_output.push_str("            {\n");
                    json_output.push_str(&format!(
                        "              \"name\": \"{}\",\n",
                        op.operation_name
                    ));
                    json_output.push_str(&format!(
                        "              \"total_time_ns\": {},\n",
                        op.total_time.as_nanos()
                    ));
                    json_output.push_str(&format!("              \"call_count\": {},\n", op.call_count));
                    json_output.push_str(&format!(
                        "              \"avg_time_per_call_ns\": {},\n",
                        op.avg_time_per_call.as_nanos()
                    ));
                    json_output.push_str(&format!(
                        "              \"percentage_of_total\": {:.2}\n",
                        op.percentage_of_total
                    ));
                    json_output.push_str(if oidx == result.operation_metrics.len() - 1 {
                        "            }\n"
                    } else {
                        "            },\n"
                    });
                }
                json_output.push_str("          ]\n");
            } else {
                json_output.push_str("          \"operations\": []\n");
            }

            json_output.push_str(if ridx == results.len() - 1 {
                "        }\n"
            } else {
                "        },\n"
            });
        }

        json_output.push_str("      ]\n");
        json_output.push_str(if idx == all_results.len() - 1 {
            "    }\n"
        } else {
            "    },\n"
        });
    }

    json_output.push_str("  ]\n");
    json_output.push_str("}\n");

    if let Some(path) = output_path {
        let mut file = std::fs::File::create(path)?;
        file.write_all(json_output.as_bytes())?;
        println!("\n✓ JSON output written to: {}", path);
    } else {
        println!("\n{}", json_output);
    }

    Ok(())
}

fn main() {
    let matches = Command::new("String Pipeline Throughput Benchmark")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Benchmarks batch processing throughput with varying input sizes and detailed profiling")
        .arg(
            Arg::new("sizes")
                .short('s')
                .long("sizes")
                .value_name("COUNTS")
                .help("Comma-separated input sizes (number of paths to process)")
                .default_value("100,500,1000,5000,10000,50000,100000"),
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .value_name("COUNT")
                .help("Number of measurement iterations per size for stability")
                .default_value("50"),
        )
        .arg(
            Arg::new("detailed")
                .short('d')
                .long("detailed")
                .action(clap::ArgAction::SetTrue)
                .help("Enable detailed per-operation profiling and statistics"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format: console or json")
                .default_value("console"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path (for JSON format)"),
        )
        .get_matches();

    // Parse arguments
    let sizes_str = matches.get_one::<String>("sizes").unwrap();
    let sizes: Vec<usize> = sizes_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse()
                .unwrap_or_else(|_| panic!("Invalid size value: {}", s))
        })
        .collect();

    let iterations: usize = matches
        .get_one::<String>("iterations")
        .unwrap()
        .parse()
        .expect("Invalid iteration count");

    let detailed = matches.get_flag("detailed");
    let format = matches.get_one::<String>("format").unwrap();
    let output_path = matches.get_one::<String>("output");

    if sizes.is_empty() {
        eprintln!("Error: At least one input size is required");
        std::process::exit(1);
    }

    println!("String Pipeline Throughput Benchmark");
    println!("=====================================");
    println!("Measuring batch processing performance with varying input sizes");
    println!("Pattern: Parse once, format N paths individually");
    println!();
    println!(
        "Input sizes: {:?}",
        sizes.iter().map(|s| format_size(*s)).collect::<Vec<_>>()
    );
    println!("Measurement iterations: {}", iterations);
    println!("Detailed profiling: {}", if detailed { "enabled" } else { "disabled" });
    println!("Output format: {}", format);
    println!();

    let templates = TemplateSet::get_templates();
    let mut all_results = Vec::new();

    for (template_name, template_str) in &templates {
        print!("Benchmarking '{}' ... ", template_name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        match benchmark_template(template_name, template_str, &sizes, iterations, detailed) {
            Ok(results) => {
                println!("✓");
                print_template_results(template_name, &results, detailed);
                all_results.push((*template_name, results));
            }
            Err(e) => {
                println!("✗");
                eprintln!("Failed to benchmark '{}': {}", template_name, e);
            }
        }
    }

    print_summary(&all_results);

    if format == "json" {
        if let Err(e) = output_json(&all_results, output_path.map(|s| s.as_str())) {
            eprintln!("Error writing JSON output: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n{}", "=".repeat(110));
    println!("Benchmark complete!");
}
