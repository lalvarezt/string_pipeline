use clap::{Arg, Command};
use comfy_table::{
    Attribute as TableAttribute, Cell, Color as TableColor, ContentArrangement, Table,
    presets::UTF8_FULL,
};
use crossterm::{
    cursor, execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use serde::{Serialize, Serializer};
use std::io::{self, Write};
use std::time::{Duration, Instant};
use string_pipeline::Template;
use unicode_width::UnicodeWidthStr;

const TOOL_VERSION: &str = "1.0.0";

// Helper to serialize Duration as nanoseconds
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u128(duration.as_nanos())
}

/// Represents the results of a throughput benchmark for a specific input size
#[derive(Debug, Clone, Serialize)]
struct BenchmarkResult {
    input_size: usize,
    #[serde(serialize_with = "serialize_duration")]
    parse_time: Duration,
    #[serde(serialize_with = "serialize_duration")]
    total_format_time: Duration,
    #[serde(serialize_with = "serialize_duration")]
    avg_time_per_path: Duration,
    throughput_paths_per_sec: f64,
    parse_percentage: f64,
    latency_stats: LatencyStatistics,
}

/// Statistical analysis of latency distribution
#[derive(Debug, Clone, Serialize)]
struct LatencyStatistics {
    #[serde(serialize_with = "serialize_duration")]
    min: Duration,
    #[serde(serialize_with = "serialize_duration")]
    p50: Duration,
    #[serde(serialize_with = "serialize_duration")]
    p95: Duration,
    #[serde(serialize_with = "serialize_duration")]
    p99: Duration,
    #[serde(serialize_with = "serialize_duration")]
    max: Duration,
    stddev: f64,
    sample_count: usize,
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
            latency_stats,
        }
    }

    fn calculate_statistics(times: &[Duration]) -> LatencyStatistics {
        let sample_count = times.len();

        if times.is_empty() {
            return LatencyStatistics {
                min: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
                stddev: 0.0,
                sample_count: 0,
            };
        }

        let mut sorted_times: Vec<Duration> = times.to_vec();
        sorted_times.sort();

        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];

        // Nearest-rank percentile calculation: ceil(p * n) - 1
        let n = sorted_times.len() as f64;
        let p50_idx = ((n * 0.50).ceil() as usize).saturating_sub(1);
        let p95_idx = ((n * 0.95).ceil() as usize).saturating_sub(1);
        let p99_idx = ((n * 0.99).ceil() as usize).saturating_sub(1);

        let p50 = sorted_times[p50_idx];
        let p95 = sorted_times[p95_idx];
        let p99 = sorted_times[p99_idx];

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
            sample_count,
        }
    }

    fn scaling_factor(&self, baseline: &BenchmarkResult) -> f64 {
        let expected = self.input_size as f64 / baseline.input_size as f64;
        let actual =
            self.total_format_time.as_secs_f64() / baseline.total_format_time.as_secs_f64();
        actual / expected
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
            ("Regex extract filename", "{regex_extract:[^/]+$}"),
            (
                "Uppercase all components",
                "{split:/:..|map:{upper}|join:/}",
            ),
            ("Remove hidden dirs", "{split:/:..|filter_not:^\\.|join:/}"),
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

/// Runs a benchmark for a single template with varying input sizes
fn benchmark_template(
    _template_name: &str,
    template_str: &str,
    sizes: &[usize],
    iterations: usize,
) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let generator = PathGenerator::new();
    let mut results = Vec::new();

    // Parse template N times and average
    let mut total_parse_time = Duration::ZERO;
    for _ in 0..iterations {
        let parse_start = Instant::now();
        let _ = Template::parse(template_str)?;
        total_parse_time += parse_start.elapsed();
    }
    let avg_parse_time = total_parse_time / iterations as u32;

    // Parse once for actual use
    let template = Template::parse(template_str)?;

    for &size in sizes {
        // Generate N paths for this size
        let paths = generator.generate_paths(size);

        // Warmup: format all paths once (skip if iterations == 1)
        if iterations > 1 {
            for path in &paths {
                let _ = template.format(path)?;
            }
        }

        // Measure: time complete iterations, calculate avg per-path for each iteration
        let mut iteration_total_times = Vec::new();
        let mut iteration_avg_times = Vec::new();

        for _ in 0..iterations {
            let iteration_start = Instant::now();
            for path in &paths {
                let _ = template.format(path)?;
            }
            let iteration_time = iteration_start.elapsed();
            iteration_total_times.push(iteration_time);

            // Calculate average time per path for this iteration (for statistics)
            let avg_per_path = iteration_time / size as u32;
            iteration_avg_times.push(avg_per_path);
        }

        // Calculate average total time across all iterations
        let total_duration: Duration = iteration_total_times.iter().sum();
        let avg_format_time = total_duration / iterations as u32;

        let result =
            BenchmarkResult::new(size, avg_parse_time, avg_format_time, iteration_avg_times);

        results.push(result);
    }

    Ok(results)
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

// Styled output helpers
fn print_header(text: &str) {
    let mut stdout = io::stdout();
    let text_width = text.width();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("╔"),
        Print("═".repeat(78)),
        Print("╗\n║ "),
        Print(text),
        Print(" ".repeat(77 - text_width)),
        Print("║\n╚"),
        Print("═".repeat(78)),
        Print("╝\n"),
        ResetColor
    );
}

fn print_section_header(text: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(text),
        ResetColor,
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("─".repeat(80)),
        ResetColor
    );
}

fn print_error(msg: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Red),
        Print("✗ "),
        ResetColor,
        Print(msg),
        Print("\n")
    );
}

fn print_progress_bar(current: usize, total: usize, template_name: &str) {
    let mut stdout = io::stdout();
    let progress = (current as f64 / total as f64) * 100.0;
    let filled = ((progress / 100.0) * 40.0) as usize;
    let _ = queue!(
        stdout,
        cursor::MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Cyan),
        Print("["),
        SetForegroundColor(Color::Green),
        Print("█".repeat(filled)),
        SetForegroundColor(Color::DarkGrey),
        Print("░".repeat(40 - filled)),
        SetForegroundColor(Color::Cyan),
        Print("]"),
        ResetColor,
        Print(format!(" {:.0}% ({}/{}) - ", progress, current, total)),
        SetAttribute(Attribute::Dim),
        Print(template_name),
        ResetColor
    );
    stdout.flush().ok();
}

fn print_template_results(template_name: &str, results: &[BenchmarkResult]) {
    print_section_header(&format!("Template: {}", template_name));

    // Create results table with comfy-table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Input Size")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Parse Time")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Total Time")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Avg/Path")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Throughput")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Parse %")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Scaling")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
        ]);

    for (idx, result) in results.iter().enumerate() {
        let scaling = if idx == 0 {
            "baseline".to_string()
        } else {
            format!("{:.2}x", result.scaling_factor(&results[0]))
        };

        table.add_row(vec![
            Cell::new(format_size(result.input_size)),
            Cell::new(format_duration(result.parse_time)),
            Cell::new(format_duration(result.total_format_time)),
            Cell::new(format_duration(result.avg_time_per_path)),
            Cell::new(format_throughput(result.throughput_paths_per_sec)),
            Cell::new(format!("{:.2}%", result.parse_percentage)),
            Cell::new(scaling),
        ]);
    }

    println!("\n{}", table);

    // Scaling analysis
    if results.len() >= 2 {
        let first = &results[0];
        let last = &results[results.len() - 1];

        let size_ratio = last.input_size as f64 / first.input_size as f64;
        let time_ratio =
            last.total_format_time.as_secs_f64() / first.total_format_time.as_secs_f64();
        let scaling_quality = time_ratio / size_ratio;

        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Magenta),
            Print("📊 Scaling Analysis:\n"),
            ResetColor
        );
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

    // Latency statistics for largest size
    if !results.is_empty() {
        let largest_result = results.last().unwrap();

        // Latency statistics
        let stats = &largest_result.latency_stats;
        println!(
            "\n📈 Latency Statistics (at {} inputs):",
            format_size(largest_result.input_size)
        );
        println!(
            "   Min: {}  p50: {}  p95: {}  p99: {}  Stddev: {}",
            format_duration(stats.min),
            format_duration(stats.p50),
            format_duration(stats.p95),
            format_duration(stats.p99),
            format_duration(Duration::from_nanos(stats.stddev as u64))
        );

        // Performance consistency analysis
        let p50_ns = stats.p50.as_nanos() as f64;
        let p99_ns = stats.p99.as_nanos() as f64;

        if p50_ns > 0.0 {
            let p99_p50_ratio = p99_ns / p50_ns;
            let stddev_percent = (stats.stddev / p50_ns) * 100.0;

            println!("   Analysis:");

            // Consistency (p99/p50 ratio)
            print!("   - Consistency: {:.2}x", p99_p50_ratio);
            if p99_p50_ratio < 2.0 {
                println!(" (excellent - very predictable)");
            } else if p99_p50_ratio < 3.0 {
                println!(" (good - mostly consistent)");
            } else if p99_p50_ratio < 5.0 {
                println!(" (fair - some variance)");
            } else {
                println!(" (poor - high variance)");
            }

            // Variance (stddev %)
            print!("   - Variance: {:.1}%", stddev_percent);
            if stddev_percent < 20.0 {
                println!(" (low - stable)");
            } else if stddev_percent < 40.0 {
                println!(" (moderate)");
            } else {
                println!(" (high - jittery)");
            }
        }

        println!();
    }
}

fn print_statistics_explanation(sample_count: usize) {
    print_header("📖 LATENCY STATISTICS METHODOLOGY");

    println!(
        "   Latency statistics calculated from {} iteration samples",
        sample_count
    );
    println!("   Each sample = average time per path for one complete iteration");
    println!();
    println!("   Statistical Methods:");
    println!("   - Percentiles: Nearest-rank method on sorted iteration averages");
    println!("     • p50 = value at index ceil(n × 0.50) - 1");
    println!("     • p95 = value at index ceil(n × 0.95) - 1");
    println!("     • p99 = value at index ceil(n × 0.99) - 1");
    println!();
    println!("   - Consistency: p99/p50 ratio (lower = more predictable)");
    println!("   - Variance: (stddev/p50) × 100% (lower = more stable)");
    println!("   - Stddev: √(Σ(x - mean)² / n) over iteration samples");
    println!();
}

fn print_summary(all_results: &[(&str, Vec<BenchmarkResult>)]) {
    // Get the largest input size for the header
    let largest_size = all_results
        .iter()
        .filter_map(|(_, results)| results.last().map(|r| r.input_size))
        .max()
        .unwrap_or(0);

    let header_text = format!(
        "📊 SUMMARY - Performance at Largest Input Size ({})",
        format_size(largest_size)
    );
    print_header(&header_text);

    // Collect results with latency stats for sorting
    let mut summary_data: Vec<(&str, Duration, Duration, Duration, f64, f64)> = all_results
        .iter()
        .filter_map(|(name, results)| {
            results.last().map(|last| {
                (
                    *name,
                    last.avg_time_per_path,
                    last.latency_stats.p95,
                    last.latency_stats.p99,
                    last.latency_stats.stddev,
                    last.throughput_paths_per_sec,
                )
            })
        })
        .collect();

    // Sort by throughput (highest first)
    summary_data.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap());

    // Create summary table with comfy-table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Template")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Avg/Path")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("p95")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("p99")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Stddev")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Throughput")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
        ]);

    for (idx, (template_name, avg_time, p95, p99, stddev, throughput)) in
        summary_data.iter().enumerate()
    {
        // Highlight fastest (green) and slowest (yellow)
        let color = if idx == 0 {
            TableColor::Green
        } else if idx == summary_data.len() - 1 {
            TableColor::Yellow
        } else {
            TableColor::Reset
        };

        table.add_row(vec![
            Cell::new(template_name).fg(color),
            Cell::new(format_duration(*avg_time)).fg(color),
            Cell::new(format_duration(*p95)).fg(color),
            Cell::new(format_duration(*p99)).fg(color),
            Cell::new(format_duration(Duration::from_nanos(*stddev as u64))).fg(color),
            Cell::new(format_throughput(*throughput)).fg(color),
        ]);
    }

    println!("{}", table);
}

/// Output results in JSON format for tracking over time
#[derive(Serialize)]
struct BenchmarkOutput<'a> {
    version: String,
    timestamp: u64,
    benchmarks: Vec<TemplateBenchmark<'a>>,
}

#[derive(Serialize)]
struct TemplateBenchmark<'a> {
    template_name: &'a str,
    results: &'a [BenchmarkResult],
}

fn output_json(
    all_results: &[(&str, Vec<BenchmarkResult>)],
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let benchmarks: Vec<TemplateBenchmark> = all_results
        .iter()
        .map(|(name, results)| TemplateBenchmark {
            template_name: name,
            results,
        })
        .collect();

    let output = BenchmarkOutput {
        version: TOOL_VERSION.to_string(),
        timestamp,
        benchmarks,
    };

    let json_string = serde_json::to_string_pretty(&output)?;

    if let Some(path) = output_path {
        std::fs::write(path, json_string)?;
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Green),
            Print("✓ JSON output written to: "),
            ResetColor,
            Print(format!("{}\n", path))
        );
    } else {
        println!("\n{}", json_string);
    }

    Ok(())
}

fn get_default_output_path() -> Result<String, Box<dyn std::error::Error>> {
    let data_home = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME environment variable not set");
        format!("{}/.local/share", home)
    });

    let benchmark_dir = format!("{}/string-pipeline/benchmarks", data_home);
    std::fs::create_dir_all(&benchmark_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    Ok(format!("{}/bench-{}.json", benchmark_dir, timestamp))
}

fn main() {
    let matches = Command::new("String Pipeline Throughput Benchmark")
        .version(TOOL_VERSION)
        .about("Benchmarks batch processing throughput with varying input sizes")
        .arg(
            Arg::new("sizes")
                .short('s')
                .long("sizes")
                .value_name("COUNTS")
                .help("Comma-separated input sizes (number of paths to process)")
                .default_value("10000"),
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .value_name("COUNT")
                .help("Number of measurement iterations per size for stability")
                .default_value("1"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Override default JSON output location (default: $XDG_DATA_HOME/string-pipeline/benchmarks/bench-<timestamp>.json)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed output for each template (default shows only summary)"),
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

    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.to_string())
        .or_else(|| get_default_output_path().ok());
    let verbose = matches.get_flag("verbose");

    if sizes.is_empty() {
        eprintln!("Error: At least one input size is required");
        std::process::exit(1);
    }

    // Always show header
    print_header(&format!(
        "String Pipeline Throughput Benchmark {}",
        TOOL_VERSION
    ));
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        Print("Measuring batch processing performance with varying input sizes\n"),
        Print("Pattern: Parse and format N paths with M iterations for stability\n\n"),
        SetForegroundColor(Color::Cyan),
        Print("Input sizes: "),
        ResetColor,
        Print(format!(
            "{:?}\n",
            sizes.iter().map(|s| format_size(*s)).collect::<Vec<_>>()
        )),
        SetForegroundColor(Color::Cyan),
        Print("Measurement iterations: "),
        ResetColor,
        Print(format!("{}\n", iterations))
    );

    let templates = TemplateSet::get_templates();
    let mut all_results = Vec::new();
    let total_templates = templates.len();

    for (idx, (template_name, template_str)) in templates.iter().enumerate() {
        // Always show progress bar
        print_progress_bar(idx + 1, total_templates, template_name);

        match benchmark_template(template_name, template_str, &sizes, iterations) {
            Ok(results) => {
                let mut stdout = io::stdout();
                let _ = execute!(
                    stdout,
                    cursor::MoveToColumn(0),
                    Clear(ClearType::CurrentLine)
                );
                if verbose {
                    print_template_results(template_name, &results);
                }
                all_results.push((*template_name, results));
            }
            Err(e) => {
                let mut stdout = io::stdout();
                let _ = execute!(
                    stdout,
                    cursor::MoveToColumn(0),
                    Clear(ClearType::CurrentLine)
                );
                print_error(&format!("Failed to benchmark '{}': {}", template_name, e));
            }
        }
    }

    // Get iteration count from first template for statistics explanation
    let sample_count = if !all_results.is_empty() && !all_results[0].1.is_empty() {
        all_results[0].1[0].latency_stats.sample_count
    } else {
        iterations
    };

    // In verbose mode, show statistics explanation before summary
    if verbose {
        print_statistics_explanation(sample_count);
    }

    // Always show summary
    print_summary(&all_results);

    // Always output JSON
    if let Some(path) = output_path.as_ref()
        && let Err(e) = output_json(&all_results, Some(path.as_str()))
    {
        eprintln!("Error writing JSON output: {}", e);
        std::process::exit(1);
    }

    // Always show completion message
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold),
        Print("✓ Benchmark complete!\n"),
        ResetColor
    );
}
