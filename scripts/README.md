# Benchmark CI/CD Scripts

This directory contains scripts used by the GitHub Actions CI/CD pipeline to track and compare performance benchmarks.

## Overview

The benchmark system uses an **on-demand approach** triggered via PR comments. There are no automatic benchmark runs,
all comparisons are triggered manually by the repository owner using the `/bench` command.

## The `/bench` Command

### Command Syntax

```bash
/bench <ref1> <ref2> [size] [warmup] [runs]
```

**Parameters:**

- `ref1`   (required): First git reference (commit, branch, or tag)
- `ref2`   (required): Second git reference to compare
- `size`   (optional): Number of paths to process per run (default: 10000)
- `warmup` (optional): Number of warmup runs (default: 5)
- `runs`   (optional): Number of measurement runs (default: 50)

**Auto-Ordering:** The workflow automatically determines which ref is older (baseline) and which is newer (current)
based on commit timestamps. You don't need to worry about parameter order - `/bench main feature` and
`/bench feature main` produce the same comparison with correct labeling.

### Examples

```bash
# Basic comparison with all defaults (size=10000, warmup=5, runs=50)
/bench main v0.13.0

# Compare two commits with custom size
/bench abc12345 def56789 50000

# Custom size and warmup
/bench main HEAD 50000 10

# Full custom parameters: size=50000, warmup=10, runs=100
/bench main HEAD 50000 10 100

# Compare feature branch vs main (order doesn't matter)
/bench feature-branch main
```

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
   - Tests all templates in each run
7. **Results posted** as PR comment with hyperfine comparison
   - Mean execution time for each version
   - Standard deviation, min/max ranges
   - Relative speed comparison (e.g., "1.05x faster")
8. **Success reaction** 🚀 (or 😕 on failure)
9. **Artifacts uploaded** for 30 days

## Files

### `analyze_all_templates.sh`

Benchmarks all templates by running hyperfine twice (once per version).

**Usage:**

```bash
./scripts/analyze_all_templates.sh <baseline-sha> <current-sha> [options]

Options:
  --size <n>       Input size in paths (default: 10000)
  --warmup <n>     Warmup runs (default: 5)
  --runs <n>       Benchmark runs (default: 50)
  --export-dir     Output directory (default: ./template_analysis)
```

**Output:**

- Hyperfine JSON files
- Markdown report with per-template comparison
- Highlights regressions and improvements

**Workflow integration:**

```bash
# 1. Compile versions
./scripts/compile_benchmark_versions.sh abc12345 def56789

# 2. Run comprehensive analysis
./scripts/analyze_all_templates.sh abc12345 def56789 --runs 100

# 3. View results
cat template_analysis/comparison_report.md
```

### `compare_template_results.py`

Parses hyperfine JSON outputs and generates per-template comparison reports.

Called automatically by `analyze_all_templates.sh`.

## GitHub Actions Workflow

### Benchmark Command (`.github/workflows/bench-command.yml`)

The single workflow that handles all benchmark comparisons.

**Triggers:**

- PR comments starting with `/bench`

**What it does:**

1. **Validates** user permissions and parameters
2. **Installs** hyperfine
3. **Checks** both refs for benchmark tool existence
4. **Builds** the benchmark tool for each ref
5. **Runs** benchmarks with hyperfine directly
   - 5 warmup runs + 50 measurement runs
   - All templates mode (single execution time per run)
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

### Analysis with Hyperfine

```bash
# Quick overall check (all templates in one run)
hyperfine --warmup 5 --runs 50 \
  './target/release/bench-throughput --template all --size 10000 --output /dev/null'

# Detailed analysis of specific template
hyperfine --warmup 10 --runs 100 \
  './target/release/bench-throughput --template "{split:/:-1}" --size 10000 --output /dev/null'
```

### Per-Template Detailed Analysis

Analyze all templates using a single command:

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

1. Runs hyperfine with `--parameter-list` on all templates (baseline version)
2. Runs hyperfine with `--parameter-list` on all templates (current version)
3. Generates report comparing each template

**Output:**

- `template_analysis/baseline_results.json`
- `template_analysis/current_results.json`
- `template_analysis/comparison_report.md`

## Version Comparison Workflow

For comparing performance across multiple commits (e.g., to find when a regression was introduced), use the
`compile_benchmark_versions.sh` script.

### `compile_benchmark_versions.sh`

This script compiles the benchmark tool for every commit in a range, making it easy to run performance comparisons
across different versions.

**Features:**

- **Idempotent**: Only compiles versions that don't already exist
- **Safe**: Uses git worktrees in temporary directories (doesn't affect your working directory)
- **Convenient**: Stores binaries with commit SHA for easy identification
- **Non-intrusive**: Works even with uncommitted changes in your main working directory
- **Storage**: Uses `$XDG_DATA_HOME/string_pipeline/benchmarks/` (typically `~/.local/share/string_pipeline/benchmarks/`)

**Usage:**

```bash
# Compile all versions since the introduction of the benchmark tool
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

# 3. Quick overall comparison with hyperfine
./scripts/compare_benchmark_versions.sh abc12345 def56789 --all

# 4. If regression detected, run detailed per-template analysis
./scripts/analyze_all_templates.sh abc12345 def56789 --runs 100

# 5. Or analyze a specific template
./scripts/compare_benchmark_versions.sh abc12345 def56789 \
  --template "{split:/:-1}" --runs 100
```

### `compare_benchmark_versions.sh`

After compiling benchmark binaries, use this script to quickly compare performance between two versions using hyperfine.

**Requirements:**

- hyperfine must be installed (`apt install hyperfine` or `brew install hyperfine`)

**Usage:**

```bash
# Specific template mode (default)
./scripts/compare_benchmark_versions.sh abc12345 def56789

# Custom template
./scripts/compare_benchmark_versions.sh abc12345 def56789 --template "{upper}"

# All templates mode
./scripts/compare_benchmark_versions.sh abc12345 def56789 --all

# Custom parameters
./scripts/compare_benchmark_versions.sh abc12345 def56789 \
  --template "{split:/:-1}" \
  --warmup 10 --runs 100 --size 50000
```

**Example Workflow - Performance Comparison:**

```bash
# 1. Compile the versions you want to compare
./scripts/compile_benchmark_versions.sh --start abc12345 --end def56789

# 2. Run hyperfine comparison on specific template
./scripts/compare_benchmark_versions.sh abc12345 def56789 \
  --template "{split:/:-1}" \
  --warmup 10 --runs 100

# 3. For comprehensive check, use all-templates mode
./scripts/compare_benchmark_versions.sh abc12345 def56789 --all --runs 20
```

## Configuration

### Benchmark Parameters

Default parameters:

- **Input size:** 10,000 paths
- **Templates:** All predefined templates
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
- Tests all templates at once
- Provides overall execution time + per-template breakdown
- Good for regression detection
- Fast feedback (~3-5 minutes)

**Offline (Comprehensive analysis):**

- Use `compare_benchmark_versions.sh` locally
- Full control over hyperfine parameters (warmup, runs)
- Focus on specific templates
- Export results in multiple formats
- Ideal for performance investigation

**Recommended workflow:**

1. CI detects potential regression via `/bench`
2. Investigate offline with hyperfine + specific templates
3. Narrow down the problematic operation
4. Fix and verify with both CI and offline tools
