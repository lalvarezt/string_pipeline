# Bench Throughput Usage Guide

## Overview

`bench_throughput` is a comprehensive benchmarking tool for analyzing the performance of the string_pipeline library at scale. It measures throughput, latency, and operation-level performance across varying input sizes.

## Building

```bash
# Build the binary
cargo build --bin bench_throughput --release

# The binary will be at: target/release/bench_throughput
```

## Basic Usage

### Default Run

Runs all 28+ templates with default size progression (100 → 100K paths):

```bash
./target/release/bench_throughput
```

Output includes:
- Per-template performance tables
- Scaling analysis
- Summary comparison of all templates

### Custom Input Sizes

Specify which input sizes to test:

```bash
./target/release/bench_throughput --sizes 1000,10000,50000
```

### Adjust Iterations

Control measurement stability (higher = more stable, but slower):

```bash
./target/release/bench_throughput --iterations 100
```

## Advanced Features

### Detailed Profiling Mode

Enable operation-level breakdown and latency statistics:

```bash
./target/release/bench_throughput --detailed
```

Detailed mode provides:
- **Operation Breakdown**: Time spent in each operation (split, join, map, etc.)
- **Call Counts**: Number of times each operation is invoked
- **Percentage Attribution**: Which operations dominate total time
- **Latency Statistics**: min, p50, p95, p99, max, stddev

Example output:
```
🔍 Operation Breakdown (at 100K inputs):
Operation            Calls    Total Time      Avg/Call    % Total
-----------------------------------------------------------------
Split              100,000        45.2ms        452ns       35.2%
Map                100,000        52.8ms        528ns       41.1%
  ↳ trim           100,000         8.2ms         82ns       15.5% (of map)
  ↳ upper          100,000        18.6ms        186ns       35.2% (of map)
Join               100,000        15.3ms        153ns       11.9%

📈 Latency Statistics (at 100K inputs):
   Min:    452ns
   p50:    1.28μs
   p95:    1.45μs
   p99:    1.82μs
   Max:    3.21μs
   Stddev: 150.00ns
```

### JSON Output

Export results for tracking over time or generating visualizations:

```bash
# Print JSON to stdout
./target/release/bench_throughput --format json

# Write JSON to file
./target/release/bench_throughput --format json --output results.json
```

JSON output includes:
- Timestamp of benchmark run
- Git commit hash (if available)
- Full results for all templates and sizes
- Operation-level metrics (if --detailed used)
- Latency statistics

Example JSON structure:
```json
{
  "timestamp": 1730800000,
  "benchmarks": [
    {
      "template_name": "Extract filename",
      "results": [
        {
          "input_size": 100000,
          "parse_time_ns": 12450,
          "total_format_time_ns": 128500000,
          "throughput_per_sec": 778210.5,
          "latency_stats": {
            "min_ns": 1150,
            "p50_ns": 1280,
            "p95_ns": 1450,
            "p99_ns": 1820,
            "max_ns": 3210,
            "stddev_ns": 150.0
          },
          "operations": [
            {
              "name": "Split",
              "total_time_ns": 45200000,
              "call_count": 100000,
              "avg_time_per_call_ns": 452,
              "percentage_of_total": 35.2
            }
          ]
        }
      ]
    }
  ]
}
```

### Combining Flags

All flags can be combined:

```bash
./target/release/bench_throughput \
  --sizes 10000,50000,100000 \
  --iterations 25 \
  --detailed \
  --format json \
  --output benchmark_$(date +%Y%m%d).json
```

## Template Coverage

The benchmark covers **28+ comprehensive templates**:

### Core Operations (Individual)
1. **Split all** - `{split:/:..}`
2. **Split last index** - `{split:/:-1}`
3. **Join** - `{split:/:..| join:/}`
4. **Upper** - `{split:/:-1|upper}`
5. **Lower** - `{split:/:-1|lower}`
6. **Trim** - `{split:/:-1|trim}`
7. **Replace simple** - `{replace:s/\\.txt$/.md/}`
8. **Replace complex** - `{replace:s/\\/\\/+/\\//g}`
9. **Substring** - `{split:/:-1|substring:0..10}`
10. **Reverse** - `{split:/:-1|reverse}`
11. **Strip ANSI** - `{strip_ansi}`
12. **Filter** - `{split:/:..| filter:^[a-z]|join:/}`
13. **Sort** - `{split:/:..| sort|join:/}`
14. **Unique** - `{split:/:..| unique|join:/}`
15. **Pad** - `{split:/:-1|pad:50: :right}`

### Real-World Path Templates (Television Use Cases)
16. **Extract filename** - `{split:/:-1}`
17. **Extract directory** - `{split:/:0..-1|join:/}`
18. **Basename no ext** - `{split:/:-1|split:.:0}`
19. **File extension** - `{split:/:-1|split:.:-1}`
20. **Regex extract filename** - `{replace:s/^.*\\/([^/]+)$/$1/}`
21. **Uppercase all components** - `{split:/:..| map:{upper}|join:/}`
22. **Remove hidden dirs** - `{split:/:..| filter_not:^\\.|join:/}`
23. **Normalize filename** - `{split:/:-1|trim|lower}`
24. **Slug generation** - `{replace:s/ /_/g|lower}`
25. **Breadcrumb last 3** - `{split:/:..| slice:-3..|join: > }`

### Complex Chains
26. **Chain: trim+upper+pad** - `{split:/:-1|trim|upper|pad:20}`
27. **Chain: split+filter+sort+join** - `{split:/:..| filter:^[a-z]|sort|join:-}`
28. **Chain: map complex** - `{split:/:..| map:{trim|lower|replace:s/_/-/g}|join:/}`

## Use Cases

### 1. Performance Baseline

Establish baseline performance before optimizations:

```bash
# Create baseline
./target/release/bench_throughput \
  --sizes 10000,50000,100000 \
  --iterations 50 \
  --detailed \
  --format json \
  --output baseline.json
```

### 2. Before/After Comparison

Compare performance after library changes:

```bash
# Before optimization
git checkout main
cargo build --release --bin bench_throughput
./target/release/bench_throughput --format json --output before.json

# After optimization
git checkout feature-branch
cargo build --release --bin bench_throughput
./target/release/bench_throughput --format json --output after.json

# Compare results (manual or with jq)
jq '.benchmarks[0].results[-1].throughput_per_sec' before.json
jq '.benchmarks[0].results[-1].throughput_per_sec' after.json
```

### 3. Identify Bottlenecks

Find which operations need optimization:

```bash
./target/release/bench_throughput \
  --sizes 100000 \
  --iterations 10 \
  --detailed
```

Look for operations with high `% Total` in the breakdown.

### 4. Television Integration Testing

Test realistic workloads for the television TUI:

```bash
# Simulate large file browser channel (50K files)
./target/release/bench_throughput \
  --sizes 50000 \
  --iterations 25 \
  --detailed
```

Target: < 16ms total for 60 FPS rendering (1000/60 = 16.67ms per frame)

### 5. Scaling Analysis

Understand how performance scales with input size:

```bash
./target/release/bench_throughput \
  --sizes 100,1000,10000,100000,1000000 \
  --iterations 20
```

Look at the "Scaling behavior" output:
- **< 1.0x**: Sub-linear (caching benefits!)
- **1.0x**: Perfect linear scaling
- **> 1.0x**: Super-linear (potential issue)

## Interpreting Results

### Console Output

**Main Table:**
- **Input Size**: Number of paths processed
- **Parse Time**: One-time template compilation cost
- **Total Time**: Time to format all N paths
- **Avg/Path**: Average time per single path
- **Throughput**: Paths processed per second
- **Parse %**: Percentage of time spent parsing (should decrease with size)
- **Scaling**: Relative to baseline size

**Scaling Analysis:**
- **Size increase**: Multiplicative factor in input size
- **Time increase**: Multiplicative factor in execution time
- **Scaling behavior**: Ratio interpretation
- **Parse cost reduction**: How parsing becomes negligible

**Operation Breakdown** (--detailed):
- Shows time attribution per operation type
- Helps identify optimization targets
- Map operations show nested breakdown

**Latency Statistics** (--detailed):
- **Min/Max**: Range of individual path formatting times
- **p50**: Median latency (typical case)
- **p95**: 95th percentile (slow outliers)
- **p99**: 99th percentile (worst-case planning)
- **Stddev**: Consistency measure (lower = more predictable)

### Performance Targets

For television integration:
- **File browser (50K paths)**: < 100ms total, < 2μs avg/path
- **Search results (10K paths)**: < 20ms total, < 2μs avg/path
- **Git files (5K paths)**: < 10ms total, < 2μs avg/path
- **Process list (1K paths)**: < 2ms total, < 2μs avg/path

Throughput targets:
- **Good**: > 500K paths/sec
- **Excellent**: > 1M paths/sec
- **Outstanding**: > 2M paths/sec

## Troubleshooting

### Benchmark Takes Too Long

Reduce iterations or sizes:
```bash
./target/release/bench_throughput --sizes 1000,10000 --iterations 10
```

### High Variance in Results

Increase iterations for more stable measurements:
```bash
./target/release/bench_throughput --iterations 100
```

### JSON Parse Errors

Ensure you're using valid output path:
```bash
./target/release/bench_throughput --format json --output /tmp/results.json
```

## Future Enhancements

Planned features (see `bench_throughput_plan.md`):
- Cache hit/miss tracking
- Memory profiling
- Comparative analysis (baseline vs current)
- Real-world path loading (from actual directories)
- Regression detection
- Optimization recommendations

## Example Workflow

Complete workflow for performance analysis:

```bash
# 1. Initial baseline
./target/release/bench_throughput --detailed --format json --output baseline.json

# 2. Make optimization changes to library
# ... edit src/pipeline/mod.rs ...

# 3. Rebuild and re-benchmark
cargo build --release --bin bench_throughput
./target/release/bench_throughput --detailed --format json --output optimized.json

# 4. Compare key metrics
echo "Baseline throughput:"
jq '.benchmarks[] | select(.template_name == "Extract filename") | .results[-1].throughput_per_sec' baseline.json

echo "Optimized throughput:"
jq '.benchmarks[] | select(.template_name == "Extract filename") | .results[-1].throughput_per_sec' optimized.json

# 5. Calculate improvement
python3 -c "
import json
with open('baseline.json') as f: base = json.load(f)
with open('optimized.json') as f: opt = json.load(f)
base_tp = base['benchmarks'][0]['results'][-1]['throughput_per_sec']
opt_tp = opt['benchmarks'][0]['results'][-1]['throughput_per_sec']
improvement = ((opt_tp - base_tp) / base_tp) * 100
print(f'Improvement: {improvement:.2f}%')
"
```

## Quick Reference

```bash
# Fast test (minimal sizes, low iterations)
./target/release/bench_throughput --sizes 100,1000 --iterations 5

# Standard run (balanced speed/accuracy)
./target/release/bench_throughput

# Comprehensive analysis (slow but thorough)
./target/release/bench_throughput --sizes 100,1000,10000,100000,500000 --iterations 100 --detailed

# Production metrics export
./target/release/bench_throughput --detailed --format json --output "bench_$(date +%Y%m%d_%H%M%S).json"
```

## Help

For all available options:
```bash
./target/release/bench_throughput --help
```
