# Benchmark CI/CD Scripts

This directory contains scripts used by the GitHub Actions CI/CD pipeline to track and compare performance benchmarks.

## Overview

The benchmark system uses an **on-demand approach** triggered via PR comments. There are no automatic benchmark runs - all comparisons are triggered manually by the repository owner using the `/bench` command.

**Updated for bench-throughput v2.0.0**: The tool has been simplified to focus on workload execution, with hyperfine handling statistical analysis.

## The `/bench` Command

### Command Syntax

```bash
/bench <ref1> <ref2> [size]
```

**Parameters:**

- `ref1` (required): Baseline git reference (commit, branch, or tag)
- `ref2` (required): Current git reference to compare against baseline
- `size` (optional): Input size - number of paths to process (default: 10000)

### Examples

```bash
# Basic comparison with default size (10000 paths)
/bench main v0.13.0

# Compare two commits
/bench abc123 def456

# Custom input size
/bench main HEAD 50000

# Compare feature branch vs main
/bench feature-branch main
```

### What Gets Benchmarked

- **All 26 predefined templates** are tested
- **Single input size** per run
- **Hyperfine wraps execution** for statistical confidence (5 warmup + 50 runs)
- **Per-template breakdown** from internal timing

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
4. **Install hyperfine** in CI environment
5. **Build** benchmark binaries for both refs
6. **Run with hyperfine**:
   - 5 warmup runs
   - 50 measurement runs
   - Tests all 26 templates in each run
   - Statistical analysis of total execution time
7. **Results posted** as PR comment with hyperfine comparison
   - Mean execution time for each version
   - Standard deviation, min/max ranges
   - Relative speed comparison (e.g., "1.05x faster")
8. **Success reaction** 🚀 (or 😕 on failure)
9. **Artifacts uploaded** for 30 days

## Files

### `analyze_all_templates.sh`

**New in v2.0.0**: Comprehensive per-template analysis script using hyperfine's `--parameter-list`.

Efficiently benchmarks all 26 predefined templates with full statistical confidence by running hyperfine twice (once per version) instead of 26 times.

**Usage:**

```bash
./scripts/analyze_all_templates.sh <baseline-sha> <current-sha> [options]

Options:
  --size <n>       Input size in paths (default: 10000)
  --warmup <n>     Warmup runs (default: 5)
  --runs <n>       Benchmark runs (default: 50)
  --export-dir     Output directory (default: ./template_analysis)
```

**Features:**

- Uses hyperfine's `--parameter-list` to iterate over all templates
- Two-pass approach: runs hyperfine once for baseline, once for current
- Generates comprehensive comparison report with statistical data
- Much faster than running hyperfine 26 separate times
- Full mean/min/max/stddev for every template

**Output:**
- Hyperfine JSON files with complete statistical data
- Markdown report with per-template comparison
- Highlights regressions and improvements

**Workflow integration:**
```bash
# 1. Compile versions
./scripts/compile_benchmark_versions.sh abc1234 def5678

# 2. Run comprehensive analysis
./scripts/analyze_all_templates.sh abc1234 def5678 --runs 100

# 3. View results
cat template_analysis/comparison_report.md
```

### `compare_template_results.py`

Helper script that parses hyperfine JSON outputs and generates per-template comparison reports.

Called automatically by `analyze_all_templates.sh` - you typically won't run this directly.

**Features:**

- Parses hyperfine JSON format (mean, stddev, min, max)
- Generates markdown tables with statistical data
- Detects regressions and improvements
- Highlights templates with high variance
- Color-coded indicators based on change magnitude

### `compare_benchmarks.py`

**⚠️ DEPRECATED**: This script compares single-run JSON outputs from `bench-throughput`.

**Why deprecated:**
- Single-run measurements lack statistical confidence
- Cannot distinguish noise from real performance changes
- Superseded by `analyze_all_templates.sh` which uses hyperfine
- Conflicts with v2.0.0 philosophy (hyperfine for benchmarking)

**Migration:**
```bash
# Old approach (unreliable)
bench-throughput --template all --output baseline.json
bench-throughput --template all --output current.json
python3 scripts/compare_benchmarks.py baseline.json current.json

# New approach (statistically sound)
./scripts/compile_benchmark_versions.sh baseline current
./scripts/analyze_all_templates.sh baseline current
```

**Status:** Kept for backward compatibility but not recommended. May be removed in future versions.

## GitHub Actions Workflow

### Benchmark Command (`.github/workflows/bench-command.yml`)

The single workflow that handles all benchmark comparisons.

**Triggers:**

- PR comments starting with `/bench`
- Owner-only access control

**What it does:**

1. **Validates** user permissions and parameters
2. **Installs** hyperfine for statistical benchmarking
3. **Checks** both refs for benchmark tool existence
4. **Builds** the benchmark tool for each ref
5. **Runs** benchmarks with hyperfine directly
   - 5 warmup runs + 50 measurement runs
   - All 26 templates mode (single execution time per run)
   - Statistical analysis and comparison from hyperfine
   - Results exported as markdown table
6. **Posts** detailed report to PR with markdown table
7. **Uploads** artifacts (markdown results + build logs)

**Artifacts:**

- **benchmark-comparison-<comment_id>**
  - Hyperfine comparison results (markdown table)
  - Build logs for both refs (baseline and current)
  - Retained for 30 days

## Running Benchmarks Locally

### Quick Single-Template Test

```bash
cargo build --release --bin bench-throughput

# Single template, single run (quick smoke test)
./target/release/bench-throughput --template "{split:/:-1}" --size 10000

# With JSON output for inspection
./target/release/bench-throughput --template all --size 10000 --output my_benchmark.json
```

### Statistical Analysis with Hyperfine

For reliable performance measurements, always use hyperfine:

```bash
# Quick overall check (all 26 templates in one run)
hyperfine --warmup 5 --runs 50 \
  './target/release/bench-throughput --template all --size 10000 --output /dev/null'

# Detailed analysis of specific template
hyperfine --warmup 10 --runs 100 \
  './target/release/bench-throughput --template "{split:/:-1}" --size 10000 --output /dev/null'
```

### Per-Template Detailed Analysis

**New in v2.0.0**: Analyze all 26 templates with full statistical confidence using a single command:

```bash
# First, compile the versions you want to compare
./scripts/compile_benchmark_versions.sh abc1234 def5678

# Run comprehensive per-template analysis
./scripts/analyze_all_templates.sh abc1234 def5678

# With custom parameters
./scripts/analyze_all_templates.sh abc1234 def5678 \
  --size 50000 \
  --runs 100 \
  --export-dir ./my_analysis
```

**What it does:**
1. Runs hyperfine with `--parameter-list` on ALL 26 templates (baseline version)
2. Runs hyperfine with `--parameter-list` on ALL 26 templates (current version)
3. Generates comprehensive markdown report comparing each template

**Output:**
- `template_analysis/baseline_results.json` - Full hyperfine statistics for baseline
- `template_analysis/current_results.json` - Full hyperfine statistics for current
- `template_analysis/comparison_report.md` - Per-template comparison with:
  - Mean execution time and change percentage
  - Min/Max/StdDev for each template
  - Regression/improvement highlighting
  - Statistical confidence metrics

**Efficiency:** Instead of running hyperfine 26 times (one per template), this runs it **twice** (once per version) and compares the results. Much faster!

## Version Comparison Workflow

For comparing performance across multiple commits (e.g., to find when a regression was introduced), use the `compile_benchmark_versions.sh` script.

### `compile_benchmark_versions.sh`

This script compiles the benchmark tool for every commit in a range, making it easy to run performance comparisons across different versions.

**Features:**

- **Idempotent**: Only compiles versions that don't already exist
- **Safe**: Uses git worktrees in temporary directories (doesn't affect your working directory)
- **Convenient**: Stores binaries with commit SHA for easy identification
- **Non-intrusive**: Works even with uncommitted changes in your main working directory
- **Storage**: Uses `$XDG_DATA_HOME/string_pipeline/benchmarks/` (typically `~/.local/share/string_pipeline/benchmarks/`)

**Usage:**

```bash
# Compile all versions from 78594af (stabilized benchmark tool v1.0.0) to HEAD
./scripts/compile_benchmark_versions.sh

# Compile specific range
./scripts/compile_benchmark_versions.sh --start abc1234 --end def5678

# See what would be compiled (dry run)
./scripts/compile_benchmark_versions.sh --dry-run

# List already compiled versions
./scripts/compile_benchmark_versions.sh --list

# Remove all compiled versions
./scripts/compile_benchmark_versions.sh --clean

# Verbose output for debugging
./scripts/compile_benchmark_versions.sh --verbose
```

**Example Workflow - Finding a Performance Regression:**

```bash
# 1. Compile all versions
./scripts/compile_benchmark_versions.sh

# 2. Set up benchmark directory path
BENCH_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/string_pipeline/benchmarks"

# 3. Run benchmarks on two versions (all templates mode)
$BENCH_DIR/bench_throughput_abc1234 --template all --size 10000 --output before.json
$BENCH_DIR/bench_throughput_def5678 --template all --size 10000 --output after.json

# 4. Compare results
python3 scripts/compare_benchmarks.py before.json after.json

# 5. If regression found in specific template, use hyperfine for detailed analysis
hyperfine --warmup 10 --runs 100 \
  "$BENCH_DIR/bench_throughput_abc1234 --template '{split:/:-1}' --size 10000" \
  "$BENCH_DIR/bench_throughput_def5678 --template '{split:/:-1}' --size 10000"
```

### `compare_benchmark_versions.sh`

After compiling benchmark binaries, use this script to quickly compare performance between two versions using hyperfine.

**Updated for v2.0.0**: Supports both all-templates mode and specific template mode.

**Features:**

- **Fast comparison**: Uses hyperfine for accurate benchmark timing
- **Automatic validation**: Checks that both binaries exist before running
- **Flexible parameters**: Customize warmup, runs, size, and template
- **Two modes**: All templates or specific template
- **Clear output**: Shows which version is faster with statistical confidence

**Requirements:**

- hyperfine must be installed (`apt install hyperfine` or `brew install hyperfine`)

**Usage:**

```bash
# Specific template mode (default)
./scripts/compare_benchmark_versions.sh 78594af dc06069

# Custom template
./scripts/compare_benchmark_versions.sh 78594af dc06069 --template "{upper}"

# All templates mode
./scripts/compare_benchmark_versions.sh 78594af dc06069 --all

# Custom parameters
./scripts/compare_benchmark_versions.sh abc1234 def5678 \
  --template "{split:/:-1}" \
  --warmup 10 --runs 100 --size 50000
```

**Example Workflow - Performance Comparison:**

```bash
# 1. Compile the versions you want to compare
./scripts/compile_benchmark_versions.sh --start 78594af --end dc06069

# 2. Run hyperfine comparison on specific template
./scripts/compare_benchmark_versions.sh 78594af dc06069 \
  --template "{split:/:-1}" \
  --warmup 10 --runs 100

# Output shows:
# - Mean execution time for each version
# - Standard deviation
# - Min/max range
# - Relative speed comparison (e.g., "1.05x faster")

# 3. For comprehensive check, use all-templates mode
./scripts/compare_benchmark_versions.sh 78594af dc06069 --all --runs 20
```

**Important Notes:**

- In **specific template mode**: Hyperfine measures execution time with statistical confidence
- In **all templates mode**: Hyperfine times the entire 26-template run
- Both versions run with identical parameters for fair comparison
- For per-template breakdown, use the JSON output with `compare_benchmarks.py`

## Architecture Changes (v2.0.0)

### What Changed

**Removed from bench-throughput:**
- `--iterations` flag (hyperfine handles this)
- `--sizes` plural (now `--size` singular)
- Internal statistics calculation (p50, p95, p99, stddev)
- Warmup phase
- Iteration loops

**Added:**
- `--template` flag: `all` (default) or template string
- Hyperfine integration in CI/CD
- Two operating modes: all-templates and specific-template

**Philosophy Shift:**
- **Before**: bench-throughput mimicked hyperfine
- **After**: bench-throughput executes workloads, hyperfine benchmarks them

**Benefits:**
- Simpler codebase (~30% code reduction)
- Professional statistical analysis via hyperfine
- No code duplication
- Clear separation of concerns

### Migration Guide

**Old CI command:**
```bash
/bench main HEAD 100 1000,5000,10000
```

**New CI command:**
```bash
/bench main HEAD 10000
```

**Old local workflow:**
```bash
./target/release/bench-throughput \
  --sizes 1000,5000,10000 \
  --iterations 100 \
  --output results.json
```

**New local workflow:**
```bash
# For all templates (single run, per-template data)
./target/release/bench-throughput \
  --template all --size 10000 \
  --output results.json

# For specific template with hyperfine (statistical confidence)
hyperfine --warmup 10 --runs 100 \
  './target/release/bench-throughput \
    --template "{split:/:-1}" --size 10000'
```

## Configuration

### Benchmark Parameters

Default parameters:

- **Input size:** 10,000 paths
- **Templates:** All 26 predefined templates
- **Hyperfine warmup:** 5 runs (CI only)
- **Hyperfine runs:** 50 runs (CI only)

These can be overridden:

```bash
# Custom size
/bench main HEAD 50000

# Local: Custom hyperfine parameters
hyperfine --warmup 20 --runs 200 \
  './bench-throughput --template "{upper}" --size 100000'
```

## Offline vs CI Benchmarking

**CI/CD (Quick check):**
- Uses hyperfine with 5 warmup + 50 runs
- Tests all 26 templates at once
- Provides overall execution time + per-template breakdown
- Good for regression detection
- Fast feedback (~3-5 minutes)

**Offline (Comprehensive analysis):**
- Use `compare_benchmark_versions.sh` locally
- Full control over hyperfine parameters (warmup, runs)
- Focus on specific templates
- Statistical confidence with 50-200 runs
- Export results in multiple formats
- Ideal for performance investigation

**Recommended workflow:**
1. CI detects potential regression via `/bench`
2. Investigate offline with hyperfine + specific templates
3. Narrow down the problematic operation
4. Fix and verify with both CI and offline tools
