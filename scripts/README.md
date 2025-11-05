# Benchmark CI/CD Scripts

This directory contains scripts used by the GitHub Actions CI/CD pipeline to track and compare performance benchmarks.

## Overview

The benchmark CI/CD system automatically:
1. Runs performance benchmarks on every push to `main` and on pull requests
2. Compares results against the baseline (last `main` branch results)
3. Generates a detailed comparison report
4. Comments on PRs with performance changes
5. Warns about significant performance regressions

## Files

### `compare_benchmarks.py`

Python script that compares two benchmark JSON files and generates a markdown report.

**Usage:**
```bash
python3 scripts/compare_benchmarks.py baseline.json current.json > report.md
```

**Features:**
- Detects performance regressions (>5% slower)
- Highlights improvements (>5% faster)
- Compares avg/path latency, p99, and throughput
- Color-coded indicators:
  - 🟢 Significant improvement (>5% faster)
  - ✅ Improvement (2-5% faster)
  - ➖ Neutral (<2% change)
  - 🟡 Caution (2-5% slower)
  - ⚠️ Warning (5-10% slower)
  - 🔴 Regression (>10% slower)

## GitHub Actions Workflow

The benchmark workflow (`.github/workflows/benchmark.yml`) runs automatically on:
- Pushes to `main` branch
- Pull requests

### Workflow Steps

1. **Build** - Compiles the `bench_throughput` tool in release mode
2. **Run Benchmarks** - Executes benchmarks with multiple input sizes (100, 1K, 10K paths)
3. **Download Baseline** - Fetches the last benchmark from `main` branch
4. **Compare** - Runs the comparison script
5. **Comment on PR** - Posts results as a comment on pull requests
6. **Upload Artifacts** - Stores results for historical tracking
7. **Update Baseline** - Saves results as new baseline (main branch only)
8. **Check Regressions** - Warns if significant regressions detected

### Artifacts

The workflow stores three artifacts:

1. **benchmark-current** - Current run results (JSON, text, comparison)
   - Retained for 30 days
   - Available for download from workflow runs

2. **benchmark-baseline** - Baseline for comparison
   - Updated only on `main` branch pushes
   - Retained for 90 days
   - Used for comparing future PRs

## Running Benchmarks Locally

### Run benchmarks and save to JSON:
```bash
cargo build --release --bin bench_throughput

./target/release/bench_throughput \
  --sizes 100,1000,10000 \
  --iterations 50 \
  --format json \
  --output my_benchmark.json
```

### Compare two benchmark runs:
```bash
python3 scripts/compare_benchmarks.py \
  baseline_benchmark.json \
  my_benchmark.json > comparison.md

# View the report
cat comparison.md
```

## Configuration

### Benchmark Parameters

Default parameters in the CI workflow:
- **Input sizes:** 100, 1,000, 10,000 paths
- **Iterations:** 50 (per size)
- **Output format:** JSON + human-readable text

To change these, edit `.github/workflows/benchmark.yml`:
```yaml
./target/release/bench_throughput \
  --sizes 100,1000,10000,100000 \  # Add more sizes
  --iterations 100 \                # More iterations = more stable results
  --format json \
  --output benchmark_results.json
```

### Regression Thresholds

The comparison script uses these thresholds:

| Change | Classification | Emoji |
|--------|---------------|-------|
| >5% faster | Significant improvement | 🟢 |
| 2-5% faster | Improvement | ✅ |
| <2% change | Neutral (noise) | ➖ |
| 2-5% slower | Caution | 🟡 |
| 5-10% slower | Warning | ⚠️ |
| >10% slower | Regression | 🔴 |

To adjust thresholds, edit `scripts/compare_benchmarks.py`:
```python
def calculate_change(baseline: float, current: float):
    # Modify these values:
    if abs(change_pct) < 2:  # Noise threshold
        ...
    elif change_pct < -5:    # Improvement threshold
        ...
    elif change_pct > 10:    # Regression threshold
        ...
```

### Failing on Regressions

By default, the workflow **warns** about regressions but doesn't fail the build.

To fail on regressions, uncomment this line in `.github/workflows/benchmark.yml`:
```yaml
- name: Fail if significant performance regression
  run: |
    if grep -q "⚠️ PERFORMANCE REGRESSION" comparison.md; then
      echo "::warning::Performance regression detected."
      exit 1  # Uncomment this line
    fi
```

## Troubleshooting

### No baseline found
On the first run, there's no baseline for comparison. The first successful run on `main` will establish the baseline.

### Benchmark variance
Benchmarks can vary due to:
- CI runner load
- Background processes
- Network conditions

The 2% noise threshold accounts for normal variance. For more stable results:
1. Increase iteration count
2. Run benchmarks multiple times
3. Use larger input sizes (less affected by noise)

### Permission errors
The workflow needs these permissions (already configured):
```yaml
permissions:
  contents: write
  pull-requests: write
```

## Example Report

```markdown
# 📊 Benchmark Comparison Report

**Input Size:** 10,000 paths
**Baseline Timestamp:** 1699123456
**Current Timestamp:** 1699123789

## Performance Comparison

| Template | Avg/Path | Change | p99 | Change | Throughput | Change |
|----------|----------|--------|-----|--------|------------|--------|
| Strip ANSI | 304ns | ✅ -3.2% | 327ns | ➖ -1.1% | 3.29M/s | ✅ +3.3% |
| Split all | 519ns | 🔴 +12.5% | 838ns | ⚠️ +8.2% | 1.93M/s | 🔴 -11.1% |

## Summary

- **Total templates compared:** 28
- **Improvements:** 5 🟢
- **Regressions:** 2 🔴
- **Neutral:** 21 ➖

### ⚠️ PERFORMANCE REGRESSIONS

- **Split all**: +12.5% slower
```

## Further Reading

- [Benchmark Tool Documentation](../src/bin/bench_throughput.rs)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Benchmarking Best Practices](https://nnethercote.github.io/perf-book/benchmarking.html)
