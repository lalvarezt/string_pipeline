use clap::{Arg, Command};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use string_pipeline::Template;

#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    iterations: usize,
    average_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl BenchmarkResult {
    fn new(name: String, times: Vec<Duration>) -> Self {
        let iterations = times.len();
        let filtered_times = remove_outliers(times.clone());

        let average_time = if filtered_times.is_empty() {
            Duration::from_nanos(0)
        } else {
            let sum: Duration = filtered_times.iter().sum();
            sum / filtered_times.len() as u32
        };

        // Calculate min and max from filtered data (after outlier removal)
        let min_time = filtered_times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::from_nanos(0));
        let max_time = filtered_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::from_nanos(0));

        BenchmarkResult {
            name,
            iterations,
            average_time,
            min_time,
            max_time,
        }
    }
}

fn remove_outliers(mut times: Vec<Duration>) -> Vec<Duration> {
    if times.len() < 4 {
        return times;
    }

    times.sort();
    let len = times.len();

    // Remove top and bottom 5% as outliers
    let outlier_count = (len as f64 * 0.05).ceil() as usize;
    let start_idx = outlier_count;
    let end_idx = len - outlier_count;

    let filtered: Vec<Duration> = times[start_idx..end_idx].to_vec();

    filtered
}

struct BenchmarkSuite {
    iterations: usize,
    warmup_iterations: usize,
    test_data: String,
    quiet: bool,
}

impl BenchmarkSuite {
    fn new(iterations: usize, quiet: bool) -> Self {
        // Create some realistic test data
        let test_data = "apple,banana,cherry,date,elderberry,fig,grape,honeydew,ice_fruit,jackfruit,kiwi,lemon,mango,nectarine,orange,papaya,quince,raspberry,strawberry,tomato,ugli_fruit,vanilla,watermelon,xigua,yellow_apple,zucchini".to_string();

        // Use 10% of iterations for warmup
        let warmup_iterations = iterations / 10;

        BenchmarkSuite {
            iterations,
            warmup_iterations,
            test_data,
            quiet,
        }
    }

    fn run_all_benchmarks(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        if !self.quiet {
            println!("Running comprehensive pipeline benchmarks...");
            println!("Warmup iterations: {}", self.warmup_iterations);
            println!("Measurement iterations: {}", self.iterations);
            println!("Test data size: {} characters", self.test_data.len());
            println!();
        }

        // Single operation benchmarks
        results.extend(self.run_single_operation_benchmarks());

        // Multiple simple operations benchmarks
        results.extend(self.run_multiple_simple_benchmarks());

        // Multiple operations with map benchmarks
        results.extend(self.run_multiple_map_benchmarks());

        // Complex nested operations benchmarks
        results.extend(self.run_complex_benchmarks());

        results
    }

    fn run_single_operation_benchmarks(&self) -> Vec<BenchmarkResult> {
        if !self.quiet {
            println!("🔸 Running single operation benchmarks...");
        }

        let benchmarks = vec![
            ("Single: split", "{split:,:..|join:,}"),
            ("Single: upper", "{upper}"),
            ("Single: lower", "{lower}"),
            ("Single: trim", "{trim}"),
            ("Single: reverse", "{reverse}"),
            ("Single: sort", "{split:,:..|sort|join:,}"),
            ("Single: unique", "{split:,:..|unique|join:,}"),
            ("Single: replace", "{replace:s/a/A/g}"),
            ("Single: filter", "{split:,:..|filter:^[a-m]|join:,}"),
        ];

        benchmarks
            .into_iter()
            .map(|(name, template_str)| {
                if !self.quiet {
                    print!("  {} ... ", name);
                }
                let result = self.benchmark_template(name, template_str);
                if !self.quiet {
                    println!("✓ avg: {:?}", result.average_time);
                }
                result
            })
            .collect()
    }

    fn run_multiple_simple_benchmarks(&self) -> Vec<BenchmarkResult> {
        if !self.quiet {
            println!("\n🔸 Running multiple simple operations benchmarks...");
        }

        let benchmarks = vec![
            ("Multi: split + join", "{split:,:..|join: }"),
            ("Multi: split + sort + join", "{split:,:..|sort|join:;}"),
            ("Multi: split + unique + join", "{split:,:..|unique|join:,}"),
            (
                "Multi: split + reverse + join",
                "{split:,:..|reverse|join:-}",
            ),
            (
                "Multi: split + filter + join",
                "{split:,:..|filter:^[a-m]|join:,}",
            ),
            (
                "Multi: split + slice + join",
                "{split:,:..|slice:0..5|join:&}",
            ),
            (
                "Multi: upper + trim + replace",
                "{upper|trim|replace:s/,/ /g}",
            ),
            (
                "Multi: split + sort + unique + join",
                "{split:,:..|sort|unique|join:+}",
            ),
        ];

        benchmarks
            .into_iter()
            .map(|(name, template_str)| {
                if !self.quiet {
                    print!("  {} ... ", name);
                }
                let result = self.benchmark_template(name, template_str);
                if !self.quiet {
                    println!("✓ avg: {:?}", result.average_time);
                }
                result
            })
            .collect()
    }

    fn run_multiple_map_benchmarks(&self) -> Vec<BenchmarkResult> {
        if !self.quiet {
            println!("\n🔸 Running multiple operations with map benchmarks...");
        }

        let benchmarks = vec![
            (
                "Map: split + map(upper) + join",
                "{split:,:..|map:{upper}|join:,}",
            ),
            (
                "Map: split + map(trim+upper) + join",
                "{split:,:..|map:{trim|upper}|join: }",
            ),
            (
                "Map: split + map(prepend) + join",
                "{split:,:..|map:{prepend:item}|join:,}",
            ),
            (
                "Map: split + map(append) + join",
                "{split:,:..|map:{append:-fruit}|join:;}",
            ),
            (
                "Map: split + map(reverse) + join",
                "{split:,:..|map:{reverse}|join:,}",
            ),
            (
                "Map: split + map(substring) + join",
                "{split:,:..|map:{substring:0..3}|join: }",
            ),
            (
                "Map: split + map(pad) + join",
                "{split:,:..|map:{pad:10:_}|join:,}",
            ),
            (
                "Map: split + map(replace) + join",
                "{split:,:..|map:{replace:s/e/E/g}|join:,}",
            ),
        ];

        benchmarks
            .into_iter()
            .map(|(name, template_str)| {
                if !self.quiet {
                    print!("  {} ... ", name);
                }
                let result = self.benchmark_template(name, template_str);
                if !self.quiet {
                    println!("✓ avg: {:?}", result.average_time);
                }
                result
            })
            .collect()
    }

    fn run_complex_benchmarks(&self) -> Vec<BenchmarkResult> {
        if !self.quiet {
            println!("\n🔸 Running complex nested operations benchmarks...");
        }

        let benchmarks = vec![
            (
                "Complex: split + map(split+join) + join",
                "{split:,:..|map:{split:_:..|join:-}|join: }",
            ),
            (
                "Complex: split + map(upper+substring) + join",
                "{split:,:..|map:{upper|substring:0..5}|join:,}",
            ),
            (
                "Complex: split + filter + map(reverse) + join",
                "{split:,:..|filter:^[a-m]|map:{reverse}|join:&}",
            ),
            (
                "Complex: split + map(upper+replace) + join",
                "{split:,:..|map:{upper|replace:s/A/a/g}|join:;}",
            ),
            (
                "Complex: split + unique + map(upper) + join",
                "{split:,:..|unique|map:{upper}|join:,}",
            ),
            (
                "Complex: split + map(replace+upper)",
                "{split:,:..|map:{replace:s/a/A/g|upper}|join:,}",
            ),
            (
                "Complex: map with substring and pad",
                "{split:,:..|map:{substring:0..3|pad:5:_}|join:+}",
            ),
            (
                "Complex: multi-level processing",
                "{split:,:..|filter:^[a-z]|map:{upper}|sort|join: }",
            ),
        ];

        benchmarks
            .into_iter()
            .map(|(name, template_str)| {
                if !self.quiet {
                    print!("  {} ... ", name);
                }
                let result = self.benchmark_template(name, template_str);
                if !self.quiet {
                    println!("✓ avg: {:?}", result.average_time);
                }
                result
            })
            .collect()
    }

    fn benchmark_template(&self, name: &str, template_str: &str) -> BenchmarkResult {
        let template = Template::parse(template_str)
            .unwrap_or_else(|e| panic!("Failed to parse template '{}': {}", template_str, e));

        // Warmup phase - run operations without timing to warm up caches and system state
        for _ in 0..self.warmup_iterations {
            let _ = template
                .format(&self.test_data)
                .unwrap_or_else(|e| panic!("Failed to execute template '{}': {}", template_str, e));
        }

        // Actual measurement phase
        let mut times = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = template
                .format(&self.test_data)
                .unwrap_or_else(|e| panic!("Failed to execute template '{}': {}", template_str, e));
            let duration = start.elapsed();
            times.push(duration);
        }

        BenchmarkResult::new(name.to_string(), times)
    }
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2}μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

fn print_text_report(results: &[BenchmarkResult], total_time: Duration, warmup_iterations: usize) {
    println!("\n{}", "=".repeat(80));
    println!("                          BENCHMARK RESULTS");
    println!("{}", "=".repeat(80));

    println!("\n📊 Summary:");
    println!("• Total benchmarks run: {}", results.len());
    println!("• Total execution time: {}", format_duration(total_time));
    println!(
        "• Measurement iterations per benchmark: {}",
        results.first().map(|r| r.iterations).unwrap_or(0)
    );
    println!(
        "• Warmup iterations per benchmark: {} (10% of measurements)",
        warmup_iterations
    );

    println!("\n📈 Detailed Results:");
    println!(
        "{:<50} {:>12} {:>12} {:>12}",
        "Benchmark", "Average", "Min", "Max"
    );
    println!("{}", "-".repeat(88));

    for result in results {
        println!(
            "{:<50} {:>12} {:>12} {:>12}",
            result.name,
            format_duration(result.average_time),
            format_duration(result.min_time),
            format_duration(result.max_time)
        );
    }

    // Category analysis
    let mut categories: HashMap<&str, Vec<&BenchmarkResult>> = HashMap::new();
    for result in results {
        let category = if result.name.starts_with("Single:") {
            "Single Operations"
        } else if result.name.starts_with("Multi:") {
            "Multiple Simple Operations"
        } else if result.name.starts_with("Map:") {
            "Map Operations"
        } else if result.name.starts_with("Complex:") {
            "Complex Operations"
        } else {
            "Other"
        };
        categories.entry(category).or_default().push(result);
    }

    println!("\n📋 Performance by Category:");
    println!("{}", "-".repeat(80));

    for (category, category_results) in categories {
        let avg_time: Duration = category_results
            .iter()
            .map(|r| r.average_time)
            .sum::<Duration>()
            / category_results.len() as u32;

        let fastest = category_results
            .iter()
            .min_by_key(|r| r.average_time)
            .unwrap();

        let slowest = category_results
            .iter()
            .max_by_key(|r| r.average_time)
            .unwrap();

        println!("🔹 {} ({} tests)", category, category_results.len());
        println!(
            "   Average: {} | Fastest: {} ({}) | Slowest: {} ({})",
            format_duration(avg_time),
            format_duration(fastest.average_time),
            fastest.name,
            format_duration(slowest.average_time),
            slowest.name
        );
        println!();
    }
}

fn main() {
    let matches = Command::new("String Pipeline Benchmark")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Comprehensive benchmarking tool for string pipeline operations")
        .arg(
            Arg::new("iterations")
                .short('n')
                .long("iterations")
                .value_name("COUNT")
                .help("Number of iterations to run for each benchmark")
                .default_value("1000"),
        )
        .get_matches();

    let iterations: usize = matches
        .get_one::<String>("iterations")
        .unwrap()
        .parse()
        .expect("Invalid number of iterations");

    if iterations < 10 {
        eprintln!("Warning: Running with less than 10 iterations may produce unreliable results");
    }

    let suite = BenchmarkSuite::new(iterations, false);
    let start_time = Instant::now();
    let results = suite.run_all_benchmarks();
    let total_time = start_time.elapsed();

    print_text_report(&results, total_time, suite.warmup_iterations);
}

