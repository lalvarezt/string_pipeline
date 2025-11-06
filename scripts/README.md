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

## GitHub Actions Workflows

### Automatic Benchmarks (`.github/workflows/benchmark.yml`)

Runs automatically on:
- Pushes to `main` branch
- Pull requests

**Workflow Steps:**

1. **Build** - Compiles the `bench_throughput` tool in release mode
2. **Run Benchmarks** - Executes benchmarks with multiple input sizes (1K, 5K, 10K paths)
3. **Download Baseline** - Fetches the baseline from manual update workflow
4. **Compare** - Runs the comparison script
5. **Comment on PR** - Posts results as a comment on pull requests
6. **Upload Artifacts** - Stores current results for historical tracking
7. **Check Regressions** - Warns if significant regressions detected

**Note:** This workflow does NOT update the baseline. Baselines are updated manually (see below).

### Manual Baseline Update (`.github/workflows/update-baseline.yml`)

Manual workflow to establish a new performance baseline.

**How to trigger:**
1. Go to Actions tab in GitHub
2. Select "Update Benchmark Baseline" workflow
3. Click "Run workflow"
4. Specify:
   - **ref**: Branch, tag, or commit to benchmark (e.g., `main`, `v0.13.0`)
   - **iterations**: Number of iterations (default: 100)

**When to update baseline:**
- After merging performance improvements
- After releasing a new version
- When establishing a new performance standard

**Why manual?**
Prevents random commits from becoming baselines. Only vetted, intentional versions should be promoted.

### On-Demand Comparison (`.github/workflows/bench-command.yml`)

Compare any two refs (commits, branches, tags) via PR comment command.

**Command syntax:**
```
/bench <ref1> <ref2>
```

**Examples:**
```
/bench main v0.13.0          # Compare main branch vs release tag
/bench abc123 def456         # Compare two commits
/bench feature-branch main   # Compare feature branch vs main
```

**Security:**
- ⚠️ **Owner-only**: Only the repository owner can trigger this command
- ✅ Works only on pull request comments (not regular issues)
- ✅ No arbitrary code execution - only git refs

**Workflow:**
1. Post `/bench <ref1> <ref2>` as a PR comment
2. Bot reacts with 👀 and posts acknowledgment
3. Runs benchmarks on both refs
4. Posts detailed comparison report to PR
5. Reacts with 🚀 on success or 😕 on failure

**Use cases:**
- Compare feature branch performance vs stable release
- Validate optimization between two specific commits
- Test performance across version boundaries
- Ad-hoc performance debugging

### Artifacts

The workflows store these artifacts:

1. **benchmark-current** (from `benchmark.yml`)
   - Current run results (JSON + comparison)
   - Retained for 30 days
   - Available for download from workflow runs

2. **benchmark-baseline** (from `update-baseline.yml`)
   - Baseline for comparison
   - Updated only manually via update-baseline workflow
   - Retained for 90 days
   - Used by benchmark.yml for comparing PRs

3. **benchmark-comparison-<comment_id>** (from `bench-command.yml`)
   - On-demand comparison results
   - Retained for 30 days
   - Includes both refs and comparison report

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

Default parameters in the CI workflows:
- **Input sizes:** 1,000, 5,000, 10,000 paths
- **Iterations:** 100 (per size)
- **Output format:** JSON

To change these, edit the respective workflow files:
- `.github/workflows/benchmark.yml` - Automatic benchmarks
- `.github/workflows/update-baseline.yml` - Baseline updates
- `.github/workflows/bench-command.yml` - On-demand comparisons

```yaml
./target/release/bench_throughput \
  --sizes 1000,5000,10000,50000 \   # Add more sizes
  --iterations 200 \                 # More iterations = more stable results
  --format json \
  --output benchmark_results.json
```

**Note:** Keep parameters consistent across all three workflows for valid comparisons.

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
