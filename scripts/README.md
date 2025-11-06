# Benchmark CI/CD Scripts

This directory contains scripts used by the GitHub Actions CI/CD pipeline to track and compare performance benchmarks.

## Overview

The benchmark system uses an **on-demand approach** triggered via PR comments. There are no automatic benchmark runs - all comparisons are triggered manually by the repository owner using the `/bench` command.

## The `/bench` Command

### Command Syntax

```bash
/bench <ref1> <ref2> [iterations] [sizes]
```

**Parameters:**
- `ref1` (required): Baseline git reference (commit, branch, or tag)
- `ref2` (required): Current git reference to compare against baseline
- `iterations` (optional): Number of benchmark iterations (default: 100)
- `sizes` (optional): Comma-separated input sizes (default: 1000,5000,10000)

### Examples

```bash
# Basic comparison with defaults (100 iterations, sizes: 1000,5000,10000)
/bench main v0.13.0

# Compare two commits
/bench abc123 def456

# Custom iterations
/bench main HEAD 200

# Custom iterations and sizes
/bench v0.12.0 v0.13.0 100 1000,5000,10000,50000

# Compare feature branch vs main
/bench feature-branch main
```

### Security

- ⚠️ **Owner-only**: Only the repository owner can trigger benchmarks
- ✅ **PR-only**: Works only on pull request comments (not regular issues)
- ✅ **Safe**: No arbitrary code execution - only validated git refs

### Workflow

1. **Post command** in a PR comment: `/bench main HEAD`
2. **Bot acknowledges** with 👀 reaction and status message
3. **Validation** checks:
   - User is repository owner
   - Both refs exist
   - Benchmark tool exists in both refs
   - Parameters are valid
4. **Benchmarks run** on both refs
5. **Results posted** as PR comment with detailed comparison
6. **Success reaction** 🚀 (or 😕 on failure)
7. **Artifacts uploaded** for 30 days

### Error Handling

The workflow handles several error cases gracefully:

**Missing benchmark tool:**
```
❌ Benchmark comparison failed

Reason: The benchmark tool (bench_throughput) does not exist in ref: v0.10.0

Solution: The benchmark tool was added in commit d264124.
Please use refs that include this commit or later.

Example: /bench main HEAD (if both include the benchmark tool)
```

**Invalid parameters:**
```
❌ Invalid format. Usage: /bench <ref1> <ref2> [iterations] [sizes]
```

**Build failures:**
The workflow will report build errors with logs attached as artifacts.

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
- Compares avg/path latency, p95, p99, and throughput
- Color-coded indicators:
  - 🟢 Significant improvement (>5% faster)
  - ✅ Improvement (2-5% faster)
  - ➖ Neutral (<2% change)
  - 🟡 Caution (2-5% slower)
  - ⚠️ Warning (5-10% slower)
  - 🔴 Regression (>10% slower)

## GitHub Actions Workflow

### Benchmark Command (`.github/workflows/bench-command.yml`)

The single workflow that handles all benchmark comparisons.

**Triggers:**
- PR comments starting with `/bench`
- Owner-only access control

**What it does:**
1. **Validates** user permissions and parameters
2. **Checks** both refs for benchmark tool existence
3. **Builds** the benchmark tool for each ref
4. **Runs** benchmarks with specified parameters
5. **Compares** results using `compare_benchmarks.py`
6. **Posts** detailed report to PR
7. **Uploads** artifacts (results + build logs)

**Artifacts:**

- **benchmark-comparison-<comment_id>**
  - Both benchmark JSON files
  - Comparison markdown report
  - Build logs for debugging
  - Retained for 30 days

## Running Benchmarks Locally

### Run benchmarks and save to JSON:
```bash
cargo build --release --bin bench_throughput

./target/release/bench_throughput \
  --sizes 1000,5000,10000 \
  --iterations 100 \
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

Default parameters:
- **Input sizes:** 1,000, 5,000, 10,000 paths
- **Iterations:** 100 (per size)
- **Output format:** JSON

These can be overridden per-command:
```bash
# Use different sizes for larger datasets
/bench main HEAD 100 10000,50000,100000

# More iterations for stable results
/bench v0.12.0 v0.13.0 500 1000,5000,10000
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

## Use Cases

### 1. Compare Feature Branch vs Main
```bash
/bench main feature-optimize-parsing
```
Use this to see if your optimization actually improves performance.

### 2. Validate Release Performance
```bash
/bench v0.12.0 v0.13.0
```
Compare performance between releases to ensure no regressions.

### 3. Debug Performance Issues
```bash
/bench abc123 def456
```
Bisect between two commits to find which one introduced a regression.

### 4. Stress Test with Large Datasets
```bash
/bench main HEAD 100 10000,50000,100000,500000
```
Test how your code scales with larger input sizes.

### 5. High-Precision Comparison
```bash
/bench main feature-branch 1000 1000,5000,10000
```
Use more iterations for more stable and reliable results.

## Troubleshooting

### No benchmark tool found
The benchmark tool (`bench_throughput`) was added in commit `d264124`. If you're comparing older commits, you'll get an error. Solution: Only compare refs that include the benchmark tool.

### Benchmark variance
Benchmarks can vary due to:
- CI runner load
- Background processes
- Network conditions

The 2% noise threshold accounts for normal variance. For more stable results:
1. Increase iteration count: `/bench main HEAD 500`
2. Use larger input sizes (less affected by noise)
3. Run benchmarks multiple times and compare

### Permission errors
Only the repository owner can trigger benchmarks. Other users will receive a permission denied message.

### Build failures
If the code doesn't compile at one of the refs, the workflow will fail. Check the workflow run logs for build errors. Artifacts include `build_ref1.log` and `build_ref2.log` for debugging.

## Example Report

When you run `/bench main HEAD`, you'll get a report like this:

```markdown
## 🔬 Benchmark Comparison Report

**Requested by:** @username

**Comparison:**
- **Baseline**: `main` (abc123)
- **Current**: `HEAD` (def456)

**Parameters:**
- **Iterations**: 100
- **Sizes**: 1000,5000,10000

---

# 📊 Benchmark Comparison Report

**Input Size:** 10,000 paths

## Performance Comparison

| Template | Avg/Path | Change | p95 | Change | Throughput | Change |
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

### ✨ Performance Improvements

- **Strip ANSI**: 3.2% faster

---

<sub>Triggered by [/bench command](https://github.com/...)</sub>
```

## Further Reading

- [Benchmark Tool Documentation](../src/bin/bench_throughput.rs)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Benchmarking Best Practices](https://nnethercote.github.io/perf-book/benchmarking.html)
