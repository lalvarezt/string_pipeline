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

/// Runs a benchmark for a single template with varying input sizes and detailed profiling
fn benchmark_template(
    _template_name: &str,
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

        let result = BenchmarkResult::new(size, parse_time, avg_format_time, individual_times);

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
        Print("═".repeat(108)),
        Print("╗\n║ "),
        Print(text),
        Print(" ".repeat(107 - text_width)),
        Print("║\n╚"),
        Print("═".repeat(108)),
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
        Print("─".repeat(110)),
        ResetColor
    );
}

fn print_success(msg: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print("✓ "),
        ResetColor,
        Print(msg),
        Print("\n")
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

fn print_template_results(template_name: &str, results: &[BenchmarkResult], detailed: bool) {
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
    if detailed && !results.is_empty() {
        let largest_result = results.last().unwrap();

        // Latency statistics
        let stats = &largest_result.latency_stats;
        println!(
            "\n📈 Latency Statistics (at {} inputs):",
            format_size(largest_result.input_size)
        );
        println!(
            "   Min: {}  p50: {}  p95: {}  p99: {}  Max: {}  Stddev: {:.2}ns",
            format_duration(stats.min),
            format_duration(stats.p50),
            format_duration(stats.p95),
            format_duration(stats.p99),
            format_duration(stats.max),
            stats.stddev
        );
        println!();
    }
}

fn print_summary(all_results: &[(&str, Vec<BenchmarkResult>)]) {
    print_header("📊 SUMMARY - Performance at Largest Input Size");

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

    // Create summary table with comfy-table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Template")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Input Size")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Avg/Path")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
            Cell::new("Throughput")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
        ]);

    for (idx, (template_name, input_size, avg_time, throughput)) in summary_data.iter().enumerate()
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
            Cell::new(format_size(*input_size)).fg(color),
            Cell::new(format_duration(*avg_time)).fg(color),
            Cell::new(format_throughput(*throughput)).fg(color),
        ]);
    }

    println!("{}", table);
}

/// Output results in JSON format for tracking over time
#[derive(Serialize)]
struct BenchmarkOutput<'a> {
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
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::SetTrue)
                .help("Minimal output (only show benchmark progress lines)"),
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
    let quiet = matches.get_flag("quiet");

    if sizes.is_empty() {
        eprintln!("Error: At least one input size is required");
        std::process::exit(1);
    }

    if !quiet {
        print_header("String Pipeline Throughput Benchmark v0.13.0");
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            Print("Measuring batch processing performance with varying input sizes\n"),
            Print("Pattern: Parse once, format N paths individually\n\n"),
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
            Print(format!("{}\n", iterations)),
            SetForegroundColor(Color::Cyan),
            Print("Detailed profiling: "),
            ResetColor,
            Print(if detailed { "enabled\n" } else { "disabled\n" }),
            SetForegroundColor(Color::Cyan),
            Print("Output format: "),
            ResetColor,
            Print(format!("{}\n", format))
        );
    }

    let templates = TemplateSet::get_templates();
    let mut all_results = Vec::new();
    let total_templates = templates.len();

    for (idx, (template_name, template_str)) in templates.iter().enumerate() {
        if !quiet {
            print_progress_bar(idx + 1, total_templates, template_name);
        }

        match benchmark_template(template_name, template_str, &sizes, iterations, detailed) {
            Ok(results) => {
                if !quiet {
                    let mut stdout = io::stdout();
                    let _ = execute!(
                        stdout,
                        cursor::MoveToColumn(0),
                        Clear(ClearType::CurrentLine),
                        SetForegroundColor(Color::Green),
                        Print("✓ "),
                        ResetColor,
                        Print(format!("Completed: {}\n", template_name))
                    );
                    print_template_results(template_name, &results, detailed);
                } else {
                    print_success(&format!("Benchmarking '{}'", template_name));
                }
                all_results.push((*template_name, results));
            }
            Err(e) => {
                if !quiet {
                    let mut stdout = io::stdout();
                    let _ = execute!(
                        stdout,
                        cursor::MoveToColumn(0),
                        Clear(ClearType::CurrentLine)
                    );
                }
                print_error(&format!("Failed to benchmark '{}': {}", template_name, e));
            }
        }
    }

    if !quiet {
        print_summary(&all_results);
    }

    if format == "json"
        && let Err(e) = output_json(&all_results, output_path.map(|s| s.as_str()))
    {
        eprintln!("Error writing JSON output: {}", e);
        std::process::exit(1);
    }

    if !quiet {
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            SetForegroundColor(Color::Green),
            SetAttribute(Attribute::Bold),
            Print("✓ Benchmark complete!\n"),
            ResetColor
        );
    }
}
