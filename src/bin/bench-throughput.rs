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

const TOOL_VERSION: &str = "2.0.0";

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
}

impl BenchmarkResult {
    fn new(
        input_size: usize,
        parse_time: Duration,
        total_format_time: Duration,
    ) -> Self {
        let avg_time_per_path = total_format_time / input_size as u32;
        let throughput_paths_per_sec = input_size as f64 / total_format_time.as_secs_f64();

        BenchmarkResult {
            input_size,
            parse_time,
            total_format_time,
            avg_time_per_path,
            throughput_paths_per_sec,
        }
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

/// Comprehensive template set with proper coverage for all operation types.
/// Organizes templates into three categories:
/// - String operations (direct, no split needed)
/// - Split operations
/// - List operations (require split first, use map:{upper} for secondary ops)
struct TemplateSet;

impl TemplateSet {
    fn get_templates() -> Vec<(&'static str, &'static str)> {
        vec![
            // ===== String Operations (direct, no split needed) =====
            ("Upper", "{upper}"),
            ("Lower", "{lower}"),
            ("Trim", "{trim}"),
            ("Trim left", "{trim:left}"),
            ("Substring range", "{substring:0..10}"),
            ("Substring negative", "{substring:-5..}"),
            ("Reverse string", "{reverse}"),
            ("Append", "{append:!!!}"),
            ("Prepend", "{prepend:>>>}"),
            ("Surround", "{surround:**}"),
            ("Pad right", "{pad:50: :right}"),
            ("Pad left", "{pad:50:0:left}"),
            ("Replace simple", "{replace:s/\\.txt$/.md/}"),
            ("Replace global", "{replace:s/\\/\\/+/\\//g}"),
            ("Regex extract", "{regex_extract:[^/]+$}"),
            ("Strip ANSI", "{strip_ansi}"),

            // ===== Split Operations =====
            ("Split all", "{split:/:..}"),
            ("Split last", "{split:/:-1}"),
            ("Split range", "{split:/:0..-1}"),

            // ===== List Operations (require split first) =====
            ("Join", "{split:/:..|join:/}"),
            ("Filter", "{split:,:..|filter:^[a-z]}"),
            ("Filter not", "{split:,:..|filter_not:^\\.}"),
            ("Sort", "{split:,:..|sort}"),
            ("Sort desc", "{split:,:..|sort:desc}"),
            ("Unique", "{split:,:..|unique}"),
            ("Slice range", "{split:,:..|slice:1..3}"),
            ("Slice negative", "{split:,:..|slice:-3..}"),
            ("Map upper", "{split:,:..|map:{upper}}"),
            ("Map complex", "{split:,:..|map:{trim|lower}}"),

            // ===== Complex Chains =====
            ("Chain string ops", "{trim|upper|pad:20}"),
            ("Chain list ops", "{split:/:..|filter:^[a-z]|sort|join:-}"),
            ("Map + join", "{split:/:..|map:{upper}|join:/}"),
            ("Nested split", "{split:/:-1|split:.:0}"),
        ]
    }
}

/// Runs a benchmark for a single template with a single input size
fn benchmark_template(
    template_str: &str,
    size: usize,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let generator = PathGenerator::new();

    // Time template parsing
    let parse_start = Instant::now();
    let template = Template::parse(template_str)?;
    let parse_time = parse_start.elapsed();

    // Generate paths
    let paths = generator.generate_paths(size);

    // Time formatting
    let format_start = Instant::now();
    for path in &paths {
        let _ = template.format(path)?;
    }
    let total_format_time = format_start.elapsed();

    Ok(BenchmarkResult::new(size, parse_time, total_format_time))
}

/// Execute a template without timing (for hyperfine integration)
fn execute_template(
    template_str: &str,
    size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse template
    let template = Template::parse(template_str)?;

    // Generate paths
    let generator = PathGenerator::new();
    let paths = generator.generate_paths(size);

    // Format all paths
    for path in &paths {
        let _ = template.format(path)?;
    }

    Ok(())
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

fn print_template_result(template_name: &str, result: &BenchmarkResult) {
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
        ]);

    table.add_row(vec![
        Cell::new(format_size(result.input_size)),
        Cell::new(format_duration(result.parse_time)),
        Cell::new(format_duration(result.total_format_time)),
        Cell::new(format_duration(result.avg_time_per_path)),
        Cell::new(format_throughput(result.throughput_paths_per_sec)),
    ]);

    println!("\n{}\n", table);
}

fn print_summary(all_results: &[(&str, BenchmarkResult)]) {
    let size = all_results[0].1.input_size;
    let header_text = format!("📊 SUMMARY - Performance at {}", format_size(size));
    print_header(&header_text);

    // Collect results for sorting
    let mut summary_data: Vec<(&str, Duration, f64)> = all_results
        .iter()
        .map(|(name, result)| (*name, result.avg_time_per_path, result.throughput_paths_per_sec))
        .collect();

    // Sort by throughput (highest first)
    summary_data.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

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
            Cell::new("Throughput")
                .add_attribute(TableAttribute::Bold)
                .fg(TableColor::Yellow),
        ]);

    for (idx, (template_name, avg_time, throughput)) in summary_data.iter().enumerate() {
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
    result: &'a BenchmarkResult,
}

fn output_json(
    all_results: &[(&str, BenchmarkResult)],
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let benchmarks: Vec<TemplateBenchmark> = all_results
        .iter()
        .map(|(name, result)| TemplateBenchmark {
            template_name: name,
            result,
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

fn list_templates() {
    println!("Available predefined templates:\n");
    for (name, template) in TemplateSet::get_templates() {
        println!("  {:<30} {}", name, template);
    }
}

fn execute_all_templates_mode(size: usize, output_path: Option<&str>, verbose: bool) {
    print_header(&format!(
        "String Pipeline Throughput Benchmark {}",
        TOOL_VERSION
    ));

    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        Print("Measuring template processing performance\n\n"),
        SetForegroundColor(Color::Cyan),
        Print("Input size: "),
        ResetColor,
        Print(format!("{}\n", format_size(size)))
    );

    let templates = TemplateSet::get_templates();
    let mut all_results = Vec::new();
    let total_templates = templates.len();

    for (idx, (template_name, template_str)) in templates.iter().enumerate() {
        print_progress_bar(idx + 1, total_templates, template_name);

        match benchmark_template(template_str, size) {
            Ok(result) => {
                let mut stdout = io::stdout();
                let _ = execute!(
                    stdout,
                    cursor::MoveToColumn(0),
                    Clear(ClearType::CurrentLine)
                );
                if verbose {
                    print_template_result(template_name, &result);
                }
                all_results.push((*template_name, result));
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

    print_summary(&all_results);

    if let Some(path) = output_path
        && let Err(e) = output_json(&all_results, Some(path))
    {
        eprintln!("Error writing JSON output: {}", e);
        std::process::exit(1);
    }

    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold),
        Print("\n✓ Benchmark complete!\n"),
        ResetColor
    );
}

fn execute_specific_template_mode(template_str: &str, size: usize) {
    match execute_template(template_str, size) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let matches = Command::new("String Pipeline Throughput Benchmark")
        .version(TOOL_VERSION)
        .about("Benchmarks template processing performance")
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .value_name("TEMPLATE")
                .help("Template to benchmark: 'all' for predefined set, or template string")
                .default_value("all"),
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("COUNT")
                .help("Number of paths to process")
                .default_value("10000"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("JSON output file (only for --template all)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed per-template results (only for --template all)"),
        )
        .arg(
            Arg::new("list")
                .long("list-templates")
                .action(clap::ArgAction::SetTrue)
                .help("List available predefined templates and exit"),
        )
        .get_matches();

    // Parse arguments
    let template_arg = matches.get_one::<String>("template").unwrap();
    let size: usize = matches
        .get_one::<String>("size")
        .unwrap()
        .parse()
        .expect("Invalid size value");
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.to_string())
        .or_else(|| get_default_output_path().ok());
    let verbose = matches.get_flag("verbose");
    let list = matches.get_flag("list");

    // List templates
    if list {
        list_templates();
        return;
    }

    // Mode 1: All templates
    if template_arg == "all" {
        execute_all_templates_mode(size, output_path.as_deref(), verbose);
    } else {
        // Mode 2: Specific template
        execute_specific_template_mode(template_arg, size);
    }
}
