# Benchmarking

String Pipeline includes a throughput-based benchmarking tool for measuring performance across varying input sizes.

## Quick Start

```bash
# Run with default settings
cargo run --release --bin bench-throughput

# Specify input sizes and iterations
cargo run --release --bin bench-throughput -- --sizes 1000,5000,10000 --iterations 100

# Generate JSON output to custom location
cargo run --release --bin bench-throughput -- --output results.json

# Verbose mode shows per-template details
cargo run --release --bin bench-throughput -- --verbose
```

## Command Line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--sizes` | `-s` | `10000` | Comma-separated input sizes |
| `--iterations` | `-i` | `1` | Number of iterations per size |
| `--output` | `-o` | `$XDG_DATA_HOME/string-pipeline/benchmarks/bench-<timestamp>.json` | Override default JSON output location |
| `--verbose` | `-v` | false | Show detailed per-template results |

## Methodology

The benchmark tool measures batch processing performance:

1. **Parse Phase**: Template is parsed once and timed across multiple iterations
2. **Warmup**: Each input size runs once without timing to stabilize caches (skipped when iterations = 1)
3. **Measurement**: Multiple iterations are timed to calculate statistics
4. **Analysis**: Results include average, percentiles (p50, p95, p99), and standard deviation

The tool always outputs both a human-readable console report and JSON data for tracking over time.

### Test Data

The benchmark generates realistic absolute file paths with varying depths (2-10 levels) using common directory names,
filenames, and extensions. Each benchmark run processes the specified number of unique paths.

### Templates Tested

The benchmark suite includes:

- **Core operations**: split, join, upper, lower, trim, replace, substring, reverse, filter, sort, unique, pad, strip_ansi
- **Path manipulation**: extract filename, extract directory, basename without extension, file extension, regex extraction
- **String transformations**: uppercase components, remove hidden directories, normalize filenames, slug generation
- **Complex chains**: multiple operations combined with map, filter, and other transformations

### Metrics

For each template and input size:

- **Parse time**: Average time to parse the template
- **Total format time**: Time to format all paths
- **Average per path**: Total time divided by input size
- **Throughput**: Paths processed per second
- **Parse percentage**: Proportion of total time spent parsing (decreases with larger inputs)
- **Scaling factor**: How performance scales relative to the baseline size
- **Latency statistics**: min, p50, p95, p99, max, and standard deviation

## Comparing Results

Use the included Python script to compare two benchmark runs:

```bash
# Run baseline benchmark
cargo run --release --bin bench-throughput -- --output baseline.json

# Make changes to the code

# Run current benchmark
cargo run --release --bin bench-throughput -- --output current.json

# Compare results
python3 scripts/compare_benchmarks.py baseline.json current.json
```

The comparison report shows:

- Performance changes for each template
- Indicators for improvements (faster) and regressions (slower)
- Summary of affected templates

### Change Indicators

For latency metrics (lower is better):

- Significant improvement: >5% faster
- Improvement: 2-5% faster
- Neutral: <2% change
- Caution: 2-5% slower
- Warning: 5-10% slower
- Regression: >10% slower

For throughput (higher is better), the thresholds are inverted.

## CI/CD Integration

### On-Demand Comparisons

Repository owners can trigger benchmark comparisons via PR comments:

```
/bench <ref1> <ref2> [iterations] [sizes]
```

Examples:

```
/bench main HEAD
/bench v0.13.0 main
/bench abc123f def456g 100 1000,5000,10000
```

The workflow:

1. Validates both refs exist and contain the benchmark tool
2. Determines which ref is older (baseline) and newer (current)
3. Builds and runs benchmarks on both refs
4. Compares results using the comparison script
5. Posts a detailed report as a PR comment

### Output Format

Results are posted showing:

- Performance comparison table with avg/path, p95, p99, throughput
- Change percentages and indicators
- Summary of improvements and regressions
- Benchmark artifacts are uploaded for 30 days

## Interpreting Results

### Scaling Analysis

The tool reports how performance scales with input size:

- **Sub-linear (<0.95x)**: Performance improves with scale (parse overhead amortization)
- **Linear (0.95-1.05x)**: Perfect scaling
- **Super-linear (>1.05x)**: Performance degrades with scale

### Latency Consistency

The p99/p50 ratio indicates performance predictability:

- **<2.0x**: Excellent - very predictable
- **2.0-3.0x**: Good - mostly consistent
- **3.0-5.0x**: Fair - some variance
- **>5.0x**: Poor - high variance

### Parse Cost Reduction

As input size increases, the percentage of time spent parsing decreases. This demonstrates that template parsing overhead is amortized across larger batch operations.

## Notes

- Always use `--release` builds for meaningful benchmarks
- Results will vary based on system load and hardware
- The tool uses realistic data patterns to reflect production usage
- Percentile calculations use the nearest-rank method
- Standard deviation is calculated across iteration averages
