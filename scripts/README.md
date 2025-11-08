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

### Run benchmarks and save to JSON

```bash
cargo build --release --bin bench-throughput

./target/release/bench-throughput \
  --sizes 1000,5000,10000 \
  --iterations 100 \
  --output my_benchmark.json
```

### Compare two benchmark runs

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
