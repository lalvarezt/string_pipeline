# 🏆 String Pipeline Benchmarking Tool

_NOTE: what follows has mostly been assembled using AI as an experiment and as a basis for further improvements._

A simple benchmarking tool that helps measure performance of string pipeline operations and provides timing information in both text and JSON formats.

## 📋 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [✨ Features Overview](#-features-overview)
- [📖 Usage Guide](#-usage-guide)
  - [Basic Usage](#basic-usage)
  - [Command Line Options](#command-line-options)
  - [Output Formats](#output-formats)
- [🧪 Benchmark Categories](#-benchmark-categories)
  - [Single Operations](#1--single-operations)
  - [Multiple Simple Operations](#2--multiple-simple-operations)
  - [Map Operations](#3-️-map-operations)
  - [Complex Operations](#4--complex-operations)
- [📊 Test Data & Methodology](#-test-data--methodology)
- [📈 Performance Analysis](#-performance-analysis)
  - [Basic Methods](#basic-methods)
  - [Timing Precision](#timing-precision)
  - [Metrics Explanation](#metrics-explanation)
- [💼 Automated Usage](#-automated-usage)
  - [Script Integration](#script-integration)
  - [Performance Comparison](#performance-comparison)
- [🔧 Development Guide](#-development-guide)
  - [Adding New Benchmarks](#adding-new-benchmarks)
  - [Performance Considerations](#performance-considerations)
  - [Best Practices](#best-practices)
- [📋 Example Results](#-example-results)
- [⚠️ Troubleshooting](#️-troubleshooting)

## 🚀 Quick Start

```bash
# Run with default settings (1000 iterations, text output)
cargo run --bin bench

# Run in release mode for better performance
cargo run --release --bin bench

# Quick test with fewer iterations
cargo run --bin bench -- --iterations 100
```

## ✨ Features Overview

- 🧪 **Test Coverage**: Tests single operations, multiple operations, map operations, and complex nested operations
- 📊 **Basic Statistics**: Runs configurable iterations (default 1000) and calculates averages with outlier removal
- 🏋️ **Warmup Phase**: Runs warmup iterations (10% of measurements) to help get consistent timing
- 🎯 **Outlier Removal**: Removes top and bottom 5% of measurements to reduce noise
- 📄 **Multiple Output Formats**: Supports both human-readable text and machine-readable JSON output
- 🏗️ **Performance Categories**: Groups results by operation type for easier analysis
- 📈 **Basic Metrics**: Provides average, minimum, maximum times from the filtered measurements
- ⚡ **Automation Support**: Works well in CI/CD and automated scripts
- 🔍 **Debug Integration**: Works with the existing debug system's timing capabilities

## 📖 Usage Guide

### Basic Usage

| Command | Description | Use Case |
|---------|-------------|----------|
| `cargo run --bin bench` | Default run (1000 iterations, text) | Development testing |
| `cargo run --release --bin bench` | Optimized build | Better performance measurements |
| `./target/release/bench.exe` | Direct binary execution | Scripts and automation |

```bash
# 🚀 Development workflow
cargo run --bin bench -- --iterations 100  # Quick test

# 🔄 More thorough testing
cargo build --release --bin bench
./target/release/bench --iterations 5000 --format json > results.json
```

### Command Line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--iterations` | `-n` | `1000` | Number of iterations per benchmark |
| `--format` | `-f` | `text` | Output format: `text` or `json` |
| `--help` | `-h` | - | Show help information |
| `--version` | `-V` | - | Show version information |

**Examples:**

```bash
# 📊 Better accuracy (more iterations)
cargo run --bin bench -- --iterations 2000

# 🤖 Machine processing (JSON output)
cargo run --bin bench -- --format json

# 🚀 Quick development test
cargo run --bin bench -- --iterations 50 --format text

# 🔍 Help and version info
cargo run --bin bench -- --help
cargo run --bin bench -- --version
```

### Output Formats

#### 📄 Text Output (Default)

Good for **reading results** and **development workflows**:

- ✅ **Progress indicators** during execution with real-time feedback
- ✅ **Formatted tables** with aligned columns and readable timing units
- ✅ **Performance summary** by category with fastest/slowest identification
- ✅ **Basic statistics** including total execution time and outlier counts
- ✅ **Color-coded** output (when terminal supports it)

```text
🔸 Running single operation benchmarks...
  Single: upper ... ✓ avg: 295ns
  Single: lower ... ✓ avg: 149ns

📊 Summary:
• Total benchmarks run: 33
• Total execution time: 392.17ms
```

#### 🤖 JSON Output

Good for **automation**, **scripts**, and **data processing**:

- ✅ **Machine-readable** structured data
- ✅ **Timestamps** and version information for tracking
- ✅ **Timing metrics** for each benchmark
- ✅ **Categorized results** for easier filtering
- ✅ **Works well** with tools like `jq`, `python`, etc.

```json
{
  "summary": {
    "total_benchmarks": 33,
    "total_execution_time_ns": 392170000,
    "iterations_per_benchmark": 1000
  },
  "categories": {
    "single_operations": [...],
    "map_operations": [...]
  },
  "timestamp": "2024-01-15T10:30:45Z",
  "version": "0.12.0"
}
```

## 🧪 Benchmark Categories

The benchmark suite is organized into **four distinct categories** that test different aspects of the pipeline system, from basic operations to complex nested transformations.

### 1. 🔧 Single Operations

Tests **individual pipeline operations** to establish baseline performance:

| Operation | Template | Purpose | Expected Performance |
|-----------|----------|---------|---------------------|
| `split` | `{split:,:..\|join:,}` | Text splitting capability | ~3-4μs |
| `upper` | `{upper}` | Case conversion | ~200-300ns |
| `lower` | `{lower}` | Case conversion | ~150-200ns |
| `trim` | `{trim}` | Whitespace removal | ~100-150ns |
| `reverse` | `{reverse}` | String/list reversal | ~600-700ns |
| `sort` | `{split:,:..\|sort\|join:,}` | Alphabetical sorting | ~3-4μs |
| `unique` | `{split:,:..\|unique\|join:,}` | Duplicate removal | ~5-6μs |
| `replace` | `{replace:s/a/A/g}` | Pattern replacement | ~2-3μs |
| `filter` | `{split:,:..\|filter:^[a-m]\|join:,}` | Pattern filtering | ~14-16μs |

> 💡 **Baseline Importance:** These measurements establish the **fundamental performance characteristics** of each operation and serve as building blocks for understanding more complex pipeline performance.

### 2. 🔗 Multiple Simple Operations

Tests **chains of basic operations** to measure composition overhead:

| Pipeline | Template | Purpose | Performance Range |
|----------|----------|---------|------------------|
| Split + Join | `{split:,:..\|join: }` | Basic transformation | ~3μs |
| Split + Sort + Join | `{split:,:..\|sort\|join:;}` | Sorting pipeline | ~3-4μs |
| Split + Unique + Join | `{split:,:..\|unique\|join:,}` | Deduplication | ~5-6μs |
| Split + Reverse + Join | `{split:,:..\|reverse\|join:-}` | Reversal pipeline | ~3μs |
| Split + Filter + Join | `{split:,:..\|filter:^[a-m]\|join:,}` | Filtering pipeline | ~16-17μs |
| Split + Slice + Join | `{split:,:..\|slice:0..5\|join:&}` | Range extraction | ~4μs |
| Upper + Trim + Replace | `{upper\|trim\|replace:s/,/ /g}` | String transformations | ~3-4μs |
| Split + Sort + Unique + Join | `{split:,:..\|sort\|unique\|join:+}` | Multi-step processing | ~5-6μs |

> 🎯 **Composition Analysis:** These tests reveal how **operation chaining affects performance** and whether there are significant overhead costs in pipeline composition.

### 3. 🗺️ Map Operations

Tests **operations applied to each list item** via the map function:

| Operation Type | Template | Purpose | Performance Range |
|----------------|----------|---------|------------------|
| Map(Upper) | `{split:,:..\|map:{upper}\|join:,}` | Case conversion mapping | ~8-9μs |
| Map(Trim+Upper) | `{split:,:..\|map:{trim\|upper}\|join: }` | Chained operations in map | ~9-10μs |
| Map(Prepend) | `{split:,:..\|map:{prepend:item}\|join:,}` | Text prefix addition | ~9-10μs |
| Map(Append) | `{split:,:..\|map:{append:-fruit}\|join:;}` | Text suffix addition | ~10-11μs |
| Map(Reverse) | `{split:,:..\|map:{reverse}\|join:,}` | String reversal per item | ~8-9μs |
| Map(Substring) | `{split:,:..\|map:{substring:0..3}\|join: }` | Text extraction per item | ~8-9μs |
| Map(Pad) | `{split:,:..\|map:{pad:10:_}\|join:,}` | Text padding per item | ~10-11μs |
| Map(Replace) | `{split:,:..\|map:{replace:s/e/E/g}\|join:,}` | Pattern replacement per item | ~49-60μs |

> 🔍 **Map Performance:** Map operations show **scaling behavior** based on list size and the complexity of the inner operation. Replace operations are notably slower due to regex processing.

### 4. 🚀 Complex Operations

Tests **sophisticated nested operations** and real-world transformation scenarios:

| Complexity Level | Template | Purpose | Performance Range |
|------------------|----------|---------|------------------|
| Nested Split+Join | `{split:,:..\|map:{split:_:..\|join:-}\|join: }` | Multi-level parsing | ~15-16μs |
| Combined Transform | `{split:,:..\|map:{upper\|substring:0..5}\|join:,}` | Chained transformations | ~10μs |
| Filter+Map Chain | `{split:,:..\|filter:^[a-m]\|map:{reverse}\|join:&}` | Conditional processing | ~16-17μs |
| Replace+Transform | `{split:,:..\|map:{upper\|replace:s/A/a/g}\|join:;}` | Pattern + transformation | ~50-60μs |
| Unique+Map | `{split:,:..\|unique\|map:{upper}\|join:,}` | Dedup + transformation | ~10-11μs |
| Multi-Replace | `{split:,:..\|map:{replace:s/a/A/g\|upper}\|join:,}` | Complex pattern work | ~51-60μs |
| Substring+Pad | `{split:,:..\|map:{substring:0..3\|pad:5:_}\|join:+}` | Text formatting pipeline | ~10-11μs |
| Multi-Level Filter | `{split:,:..\|filter:^[a-z]\|map:{upper}\|sort\|join: }` | Comprehensive processing | ~17-18μs |

> 🏆 **Real-World Scenarios:** Complex operations represent **typical production use cases** and help identify performance bottlenecks in sophisticated data transformation pipelines.

## 📊 Test Data & Methodology

### 🍎 Test Dataset

The benchmark uses a **carefully designed test dataset** that provides realistic performance characteristics:

| Property | Value | Purpose |
|----------|-------|---------|
| **Content** | Comma-separated fruit names | Real-world data structure |
| **Length** | 208 characters | Moderate size for consistent timing |
| **Items** | 26 distinct fruits | Good sample size |
| **Unicode** | ASCII + Unicode safe | Comprehensive character handling |
| **Separators** | Commas, underscores, pipes | Multiple parsing scenarios |

**Actual Test Data:**

```text
"apple,banana,cherry,date,elderberry,fig,grape,honeydew,ice_fruit,jackfruit,kiwi,lemon,mango,nectarine,orange,papaya,quince,raspberry,strawberry,tomato,ugli_fruit,vanilla,watermelon,xigua,yellow_apple,zucchini"
```

> 🎯 **Why This Dataset?** This data provides **realistic performance characteristics** without being too large to cause timing inconsistencies or too small to provide meaningful measurements.

## 📈 Performance Analysis

### Basic Methods

#### 🏋️ Warmup Phase

The benchmark includes a **warmup phase** to help get more consistent measurements by reducing cold-start effects:

| Step | Process | Rationale |
|------|---------|-----------|
| 1. **Warmup Calculation** | Calculate 10% of measurement iterations | Proportional to test size |
| 2. **Cache Warming** | Run operations without timing measurement | Prime CPU caches and memory |
| 3. **System Stabilization** | Allow CPU frequency scaling to settle | More consistent conditions |
| 4. **Memory Allocation** | Pre-allocate common data structures | Reduce allocation overhead |

```rust
// Warmup phase implementation
fn benchmark_template(&self, name: &str, template_str: &str) -> BenchmarkResult {
    let template = Template::parse(template_str)?;

    // Warmup phase - run operations without timing
    for _ in 0..self.warmup_iterations {
        let _ = template.format(&self.test_data)?;
    }

    // Actual measurement phase begins here...
}
```

> 🎯 **Warmup Benefits:** Helps reduce timing variations by reducing cold cache effects and system instability.

#### 🎯 Outlier Removal

The benchmark uses a **simple approach** to reduce measurement noise:

| Step | Process | Rationale |
|------|---------|-----------|
| 1. **Data Collection** | Collect all timing measurements | Raw performance data |
| 2. **Sorting** | Sort measurements by duration | Prepare for filtering |
| 3. **Filtering** | Remove top & bottom 5% | Remove timing outliers |
| 4. **Average Calculation** | Calculate mean of remaining 90% | More stable average |
| 5. **Reporting** | Report outliers removed count | Show what was filtered |

```rust
// Simplified outlier removal algorithm
fn remove_outliers(mut times: Vec<Duration>) -> (Vec<Duration>, usize) {
    times.sort();
    let len = times.len();
    let outlier_count = (len as f64 * 0.05).ceil() as usize;

    let start_idx = outlier_count;
    let end_idx = len - outlier_count;

    let filtered = times[start_idx..end_idx].to_vec();
    let outliers_removed = times.len() - filtered.len();

    (filtered, outliers_removed)
}
```

> 📊 **Simple Approach:** This basic filtering helps reduce noise in timing measurements, similar to what other benchmarking tools do.

### Timing Precision

#### ⚡ Timing Details

| Feature | Implementation | Benefit |
|---------|----------------|---------|
| **Resolution** | Nanosecond precision via `std::time::Instant` | Good for fast operations |
| **Overhead** | Small timing overhead (~10-20ns) | Minimal impact on results |
| **Platform** | Cross-platform timing support | Works across systems |
| **Formatting** | Automatic unit selection (ns/μs/ms/s) | Easy to read output |

#### 📏 Unit Formatting Algorithm

```rust
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
```

### Metrics Explanation

#### 📊 Core Metrics

| Metric | Description | Interpretation |
|--------|-------------|----------------|
| **Average** | Mean time after outlier removal | Main performance indicator |
| **Min** | Fastest measurement after outlier removal | Best-case timing |
| **Max** | Slowest measurement after outlier removal | Worst-case timing |
| **Iterations** | Number of measurement runs performed | How many times we measured |
| **Warmup** | Number of pre-measurement runs | System preparation cycles |

#### 🎯 Performance Ranges

| Performance Level | Time Range | Operations |
|------------------|------------|------------|
| **Ultra Fast** | < 1μs | `upper`, `lower`, `trim` |
| **Fast** | 1-10μs | `split`, `join`, `sort`, basic chains |
| **Moderate** | 10-50μs | `map` operations, complex chains |
| **Intensive** | > 50μs | `replace` operations, regex processing |

> 💡 **Iteration Guidelines:**
>
> - **Development**: 50-100 iterations for quick feedback
> - **Automation**: 500-1000 iterations for better reliability
> - **Thorough testing**: 2000-5000 iterations for more stable results

## 📋 Example Results

### 📊 Text Output Sample

```text
🔸 Running single operation benchmarks...
  Single: split ... ✓ avg: 3.53μs
  Single: upper ... ✓ avg: 295ns
  Single: lower ... ✓ avg: 149ns

🔸 Running multiple simple operations benchmarks...
  Multi: split + join ... ✓ avg: 3.12μs
  Multi: split + sort + join ... ✓ avg: 3.47μs

================================================================================
                          BENCHMARK RESULTS
================================================================================

📊 Summary:
• Total benchmarks run: 33
• Total execution time: 392.17ms
• Measurement iterations per benchmark: 1000
• Warmup iterations per benchmark: 100 (10% of measurements)

📈 Detailed Results:
Benchmark                                               Average          Min          Max
----------------------------------------------------------------------------------------
Single: upper                                             295ns        200ns       380ns
Single: lower                                             149ns        120ns       180ns
Map: split + map(replace) + join                        49.16μs      42.90μs      55.80μs

📋 Performance by Category:
🔹 Single Operations (9 tests)
   Average: 3.31μs | Fastest: 136ns (trim) | Slowest: 14.03μs (filter)

🔹 Map Operations (8 tests)
   Average: 14.22μs | Fastest: 8.35μs (map(upper)) | Slowest: 49.16μs (map(replace))
```

### 🤖 JSON Output Sample

```json
{
  "summary": {
    "total_benchmarks": 33,
    "total_execution_time_ns": 392170000,
    "total_execution_time_formatted": "392.17ms",
    "iterations_per_benchmark": 1000,
    "outlier_removal_method": "Top and bottom 5% removed",
    "warmup_iterations_per_benchmark": 100
  },
  "categories": {
    "single_operations": [
      {
        "name": "Single: upper",
        "iterations": 1000,
        "average_time_ns": 295000,
        "average_time_formatted": "295ns",
        "min_time_ns": 200000,
        "min_time_formatted": "200ns",
        "max_time_ns": 9100000,
        "max_time_formatted": "9.10μs",
        "outliers_removed": 100,
        "total_raw_measurements": 1000
      }
    ]
  },
  "timestamp": "2024-01-15T10:30:45Z",
  "version": "0.12.0"
}
```

## 💼 Automated Usage

### Script Integration

#### 🚀 GitHub Actions Example

```yaml
name: Performance Benchmarks
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build benchmark tool
        run: cargo build --release --bin bench
      - name: Run benchmarks
        run: |
          ./target/release/bench --iterations 5000 --format json > benchmark_results.json
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: benchmark_results.json
```

#### 🔍 Processing Results with jq

```bash
# Extract summary information
cat benchmark_results.json | jq '.summary'

# Get average times for single operations
cat benchmark_results.json | jq '.categories.single_operations[].average_time_formatted'

# Find slowest operations
cat benchmark_results.json | jq -r '.categories[] | .[] | "\(.name): \(.average_time_formatted)"' | sort -V

# Performance alerts (fail if any operation > 100μs)
SLOW_OPS=$(cat benchmark_results.json | jq '.categories[][] | select(.average_time_ns > 100000000)')
if [ ! -z "$SLOW_OPS" ]; then
  echo "Performance regression detected!"
  exit 1
fi
```

### Performance Comparison

#### 📊 Simple Comparison Script

```bash
#!/bin/bash
# compare_benchmarks.sh

BASELINE="baseline.json"
CURRENT="current.json"
THRESHOLD=1.1  # 10% regression threshold

# Run current benchmark
./target/release/bench --format json > "$CURRENT"

# Compare with baseline (if exists)
if [ -f "$BASELINE" ]; then
  echo "🔍 Checking for performance changes..."

  # Extract and compare key metrics
  jq -r '.categories[][] | "\(.name) \(.average_time_ns)"' "$BASELINE" > baseline_times.txt
  jq -r '.categories[][] | "\(.name) \(.average_time_ns)"' "$CURRENT" > current_times.txt

  # Performance regression analysis
  python3 << 'EOF'
import json
import sys

with open('baseline.json') as f:
    baseline = json.load(f)
with open('current.json') as f:
    current = json.load(f)

threshold = 1.1
regressions = []

for category in baseline['categories']:
    for i, bench in enumerate(baseline['categories'][category]):
        current_bench = current['categories'][category][i]
        ratio = current_bench['average_time_ns'] / bench['average_time_ns']

        if ratio > threshold:
            regressions.append({
                'name': bench['name'],
                'baseline': bench['average_time_formatted'],
                'current': current_bench['average_time_formatted'],
                'ratio': f"{ratio:.2f}x"
            })

if regressions:
    print("⚠️  Performance changes detected:")
    for reg in regressions:
        print(f"  {reg['name']}: {reg['baseline']} → {reg['current']} ({reg['ratio']})")
    sys.exit(1)
else:
    print("✅ No significant performance changes")
EOF
else
  echo "📁 No baseline found, creating baseline from current run..."
  cp "$CURRENT" "$BASELINE"
fi
```

## 🔧 Development Guide

### Adding New Benchmarks

#### 📝 Step-by-Step Process

1. **🎯 Identify the Operation Category**

   ```rust
   // Choose the appropriate method in src/bin/bench.rs
   fn run_single_operation_benchmarks()     // Individual operations
   fn run_multiple_simple_benchmarks()     // Operation chains
   fn run_multiple_map_benchmarks()        // Map operations
   fn run_complex_benchmarks()             // Complex scenarios
   ```

2. **✍️ Follow the Naming Convention**

   ```rust
   // Pattern: "Category: descriptive_name"
   ("Single: operation_name", "{template}")
   ("Multi: operation1 + operation2", "{template}")
   ("Map: split + map(operation)", "{template}")
   ("Complex: detailed_description", "{template}")
   ```

3. **🧪 Create Valid Templates**

   ```rust
   // ✅ Good examples
   ("Single: upper", "{upper}"),
   ("Multi: split + sort + join", "{split:,:..|sort|join:,}"),
   ("Map: split + map(trim)", "{split:,:..|map:{trim}|join:,}"),

   // ❌ Avoid these patterns
   ("Single: split", "{split:,}"),  // Missing range/join
   ("Map: nested", "{split:,:..|map:{map:{upper}}}"),  // Nested maps not supported
   ```

4. **🔍 Test with Small Iterations**

   ```bash
   # Test new benchmarks first
   cargo run --bin bench -- --iterations 10
   ```

### Performance Considerations

#### ⚡ Basic Guidelines

| Consideration | Impact | Recommendation |
|---------------|--------|----------------|
| **Build Mode** | 3-10x performance difference | Use `--release` for better measurements |
| **Iteration Count** | Result stability | 1000+ for automation, 2000+ for comparison |
| **Data Size** | Timing consistency | Current 208-char dataset works well |
| **System Load** | Measurement variance | Run on quiet systems when possible |
| **Memory** | Allocation overhead | Consider memory usage for intensive operations |

#### 🏗️ Architecture Insights

```rust
// Performance-critical path in benchmark execution
fn benchmark_template(&self, name: &str, template_str: &str) -> BenchmarkResult {
    // 1. Template compilation (one-time cost)
    let template = Template::parse(template_str, None).unwrap();

    // 2. Hot loop (measured operations)
    for _ in 0..self.iterations {
        let start = Instant::now();
        let _ = template.format(&self.test_data).unwrap();  // Core measurement
        let duration = start.elapsed();
        times.push(duration);
    }

    // 3. Basic analysis (post-processing)
    BenchmarkResult::new(name.to_string(), times)
}
```

### Best Practices

#### ✅ Do's

1. **🏭 Use Release Builds for Better Measurements**

   ```bash
   # Development/testing
   cargo run --bin bench -- --iterations 100

   # More accurate benchmarks
   cargo build --release --bin bench
   ./target/release/bench --iterations 2000
   ```

2. **📊 Choose Appropriate Iteration Counts**

   ```bash
   # Quick development feedback (30-60 seconds)
   --iterations 50

   # Automated scripts (2-5 minutes)
   --iterations 1000

   # Thorough analysis (5-15 minutes)
   --iterations 5000
   ```

3. **🔍 Validate Templates Before Adding**

   ```bash
   # Test individual templates
   cargo run --bin string-pipeline -- "{new_template}" "test_data"
   ```

4. **📈 Monitor Trends, Not Just Absolutes**

   ```bash
   # Track performance over time
   git log --oneline | head -10 | while read commit; do
     git checkout $commit
     ./target/release/bench --format json >> performance_history.jsonl
   done
   ```

#### ❌ Don'ts

1. **🚫 Don't Mix Debug and Release Results**

   ```bash
   # Wrong: Comparing different build modes
   cargo run --bin bench > debug_results.txt
   cargo run --release --bin bench > release_results.txt
   # These results are not comparable!
   ```

2. **🚫 Don't Ignore System Conditions**

   ```bash
   # Wrong: Running during high system load
   # Make sure system is idle before benchmarking

   # Right: Check system load
   top -bn1 | grep "load average"
   ```

3. **🚫 Don't Skip Outlier Analysis**

   ```bash
   # Wrong: Assuming outliers are always noise
   # High outlier counts may indicate:
   # - System interference
   # - Memory allocation issues
   # - Template complexity problems
   ```

## ⚠️ Troubleshooting

### Common Issues

#### 🐛 Build Problems

**Problem:** `error: failed to remove file benchmark.exe`

```bash
# Solution: Process is still running
taskkill /F /IM bench.exe  # Windows
killall bench             # Linux/macOS

# Wait a moment, then rebuild
cargo build --release --bin bench
```

**Problem:** `Parse error: Expected operation`

```bash
# Check template syntax
cargo run --bin string-pipeline -- "{your_template}" "test"

# Common fixes:
"{split:,}"          → "{split:,:..|join:,}"
"{map:{map:{upper}}}" → "{split:,:..|map:{upper}}"
```

#### ⚡ Performance Issues

**Problem:** Benchmarks taking too long

```bash
# Reduce iterations for development
cargo run --bin bench -- --iterations 100

# Check system resources
htop  # Linux/macOS
taskmgr  # Windows
```

**Problem:** Inconsistent results

```bash
# Possible causes and solutions:
# 1. System load → Run on idle system
# 2. Debug build → Use --release
# 3. Too few iterations → Increase --iterations
# 4. Background processes → Close unnecessary applications
```

#### 📊 Data Analysis Issues

**Problem:** JSON parsing errors

```bash
# Validate JSON output
./target/release/bench --format json | jq '.'

# Check for truncated output
./target/release/bench --format json > results.json
jq '.' results.json  # Should not error
```

**Problem:** Unexpected performance patterns

```bash
# Debug with template analysis
cargo run --bin string-pipeline -- "{!your_template}" "test_data"

# Profile memory usage
valgrind --tool=massif ./target/release/bench --iterations 100
```

> 💡 **Need More Help?**
>
> 🔍 **Template Issues**: Check the [Template System Documentation](template-system.md) for syntax help
>
> 🐛 **Debug Mode**: Use `{!template}` syntax to see step-by-step execution
>
> 📊 **Performance Analysis**: Consider using `cargo flamegraph` for detailed profiling
