# Benchmarking

String Pipeline includes a throughput-based benchmarking tool designed for both comprehensive template testing and hyperfine integration.

## Quick Start

```bash
# Mode 1: All templates (default)
cargo run --release --bin bench-throughput

# Mode 2: Specific template with hyperfine
hyperfine --warmup 10 --runs 100 \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1}" --size 10000'

# List available templates
cargo run --release --bin bench-throughput -- --list-templates
```

## Operating Modes

### Mode 1: All Templates (Default)

Runs all 28 predefined templates once with a single input size, providing a comprehensive performance overview.

```bash
# Default: runs all templates with 10000 paths
cargo run --release --bin bench-throughput

# Custom size and output location
cargo run --release --bin bench-throughput -- \
  --size 50000 \
  --output results.json

# Verbose mode shows per-template details
cargo run --release --bin bench-throughput -- \
  --size 10000 \
  --verbose
```

**Output:**
- Progress bar during execution
- Summary table sorted by throughput (fastest first)
- JSON export with all results
- Parse time and format time for each template
- Throughput (paths/second) metrics

### Mode 2: Specific Template (Hyperfine Integration)

Executes a single template without internal timing, designed to be orchestrated by hyperfine for statistical analysis.

```bash
# Direct execution (runs once, no statistics)
cargo run --release --bin bench-throughput -- \
  --template "{split:/:-1}" \
  --size 10000

# With hyperfine for statistical benchmarking
hyperfine --warmup 10 --runs 100 \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1}" --size 10000'
```

**Use cases:**
- Detailed performance analysis of a single template
- Comparing template variations
- Profiling specific operations
- Version-to-version performance regression testing

## Command Line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--template` | `-t` | `all` | Template: `all` for predefined set, or template string |
| `--size` | `-s` | `10000` | Number of paths to process |
| `--output` | `-o` | auto | JSON output file (all templates mode only) |
| `--verbose` | `-v` | false | Show detailed per-template results (all templates mode only) |
| `--list-templates` | | | List available predefined templates and exit |

**Note:** `--output` defaults to `$XDG_DATA_HOME/string-pipeline/benchmarks/bench-<timestamp>.json`

## Templates Tested

The benchmark suite includes 28 predefined templates covering:

### Core Operations
- **split**: `{split:/:..}`, `{split:/:-1}`
- **join**: `{split:/:..|join:/}`
- **Case transforms**: `{upper}`, `{lower}`
- **String operations**: `{trim}`, `{reverse}`, `{substring:0..10}`
- **Replace**: `{replace:s/\\.txt$/.md/}`, `{replace:s/\\/\\/+/\\//g}`
- **Collections**: `{filter:^[a-z]}`, `{sort}`, `{unique}`
- **Formatting**: `{pad:50: :right}`, `{strip_ansi}`

### Path Manipulation (Real-world Television Use Cases)
- Extract filename: `{split:/:-1}`
- Extract directory: `{split:/:0..-1|join:/}`
- Basename without extension: `{split:/:-1|split:.:0}`
- File extension: `{split:/:-1|split:.:-1}`
- Regex extraction: `{regex_extract:[^/]+$}`

### Complex Chains
- Uppercase components: `{split:/:..|map:{upper}|join:/}`
- Remove hidden dirs: `{split:/:..|filter_not:^\\.|join:/}`
- Normalize filename: `{split:/:-1|trim|lower}`
- Slug generation: `{replace:s/ /_/g|lower}`
- Multi-operation chains: `{split:/:..|map:{trim|lower|replace:s/_/-/g}|join:/}`

## Test Data

The benchmark generates realistic absolute file paths with:
- Varying depths (2-10 directory levels)
- Common directory names (home, usr, var, projects, src, etc.)
- Realistic filenames and extensions
- Deterministic generation for reproducibility

Example paths:
```
/home/usr/var/projects/src/main.rs
/opt/workspace/repos/tests/docs/config/utils/test.json
```

## Metrics

For each template (in all templates mode):

- **Parse time**: Time to parse the template
- **Total format time**: Time to format all paths
- **Average per path**: Total time divided by input size
- **Throughput**: Paths processed per second

## Comparing Results

### Quick Overall Check

For rapid feedback on overall performance:

```bash
# Quick smoke test (single run, no statistics)
cargo run --release --bin bench-throughput -- --template all --size 10000

# Statistical check with hyperfine
hyperfine --warmup 5 --runs 50 \
  'cargo run --release --bin bench-throughput -- --template all --size 10000 --output /dev/null'
```

### Comparing Two Versions

For comparing performance across git commits:

```bash
# 1. Compile both versions
./scripts/compile_benchmark_versions.sh abc1234 def5678

# 2. Quick overall comparison
./scripts/compare_benchmark_versions.sh abc1234 def5678 --all

# 3. If regression detected, detailed per-template analysis
./scripts/analyze_all_templates.sh abc1234 def5678 --runs 100
```

**Output from analyze_all_templates.sh:**
- Statistical confidence for each of 28 templates
- Mean, min, max, stddev for every template
- Regression/improvement highlighting
- Comprehensive markdown report

### Analyzing Specific Templates

For targeted performance analysis:

```bash
# Compare a specific template across versions
hyperfine --warmup 10 --runs 100 \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1}" --size 10000' \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1|upper}" --size 10000'

# Export results for further analysis
hyperfine --warmup 10 --runs 100 --export-json results.json \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1}" --size 10000'
```

### Version-to-Version Comparison

Using the helper scripts:

```bash
# Compile benchmark binaries for different commits
./scripts/compile_benchmark_versions.sh --start 78594af --end HEAD

# Compare two versions with hyperfine
./scripts/compare_benchmark_versions.sh 78594af dc06069

# Compare specific template
./scripts/compare_benchmark_versions.sh 78594af dc06069 \
  --template "{split:/:-1|upper}" --warmup 10 --runs 100

# Compare all templates mode
./scripts/compare_benchmark_versions.sh 78594af dc06069 --all
```

## CI/CD Integration

### On-Demand Comparisons

Repository owners can trigger benchmark comparisons via PR comments:

```
/bench <ref1> <ref2> [size] [warmup] [runs]
```

**Parameters:**
- `ref1`, `ref2` (required): Git references to compare (commits, branches, or tags)
- `size` (optional): Number of paths to process per run (default: 10000)
- `warmup` (optional): Number of warmup runs (default: 5)
- `runs` (optional): Number of measurement runs (default: 50)

**Examples:**

```
# Basic comparison with defaults
/bench main HEAD

# Compare with custom size
/bench v0.13.0 main 50000

# Full custom parameters
/bench abc123 def456 50000 10 100
```

**How it works:**

1. Validates both refs exist and contain the benchmark tool
2. Automatically determines which ref is older (baseline) and newer (current) by commit timestamp
3. Builds the benchmark tool for both refs
4. Runs hyperfine with all 28 templates on both versions
5. Posts a detailed report with statistical comparison as a PR comment

**Note:** The CI workflow uses all templates mode for comprehensive regression testing. For detailed per-template analysis with full statistical confidence, use the local `analyze_all_templates.sh` script.

## Advanced Usage

### Benchmarking Template Variations

Compare different approaches to the same task:

```bash
hyperfine --warmup 10 --runs 100 \
  'cargo run --release --bin bench-throughput -- \
    --template "{split:/:-1|split:.:0}" --size 10000' \
  'cargo run --release --bin bench-throughput -- \
    --template "{regex_extract:^(.+)\\.[^.]+$}" --size 10000'
```

### Profiling with Different Input Sizes

```bash
for size in 1000 10000 100000; do
  hyperfine --warmup 5 --runs 50 \
    --export-json "results_${size}.json" \
    "cargo run --release --bin bench-throughput -- \
      --template '{split:/:-1}' --size $size"
done
```

### Batch Benchmarking Multiple Templates

```bash
for template in "Split last index" "Extract filename" "Join"; do
  hyperfine --warmup 5 --runs 50 \
    --export-json "results_${template// /_}.json" \
    "cargo run --release --bin bench-throughput -- \
      --template '$template' --size 10000"
done
```

## Interpreting Results

### Throughput Metrics

Higher is better for paths/second:
- **>1M/s**: Excellent - very fast operations
- **100K-1M/s**: Good - efficient processing
- **10K-100K/s**: Fair - moderate complexity
- **<10K/s**: Slow - complex or inefficient operations

### Parse Time vs Format Time

Parse time represents one-time template compilation cost. For batch processing:
- Low parse percentage (<5%): Parse overhead is negligible
- High parse percentage (>20%): Consider caching parsed templates

### Comparing Templates

In the summary table:
- **Green**: Fastest template
- **Yellow**: Slowest template
- Use this to identify optimization opportunities

## Best Practices

1. **Always use `--release` builds** for meaningful benchmarks
2. **Run on a quiet system** to reduce noise in measurements
3. **Use consistent input sizes** when comparing results
4. **Use hyperfine** for statistical significance in specific template tests
5. **Use all templates mode** for comprehensive regression testing
6. **Check parse time** if you're running templates many times

## Architecture

### Design Philosophy

**Version 1.x (Old)**: bench-throughput mimicked hyperfine with internal iterations, warmup, and statistics.

**Version 2.0 (Current)**: bench-throughput is a workload executor, hyperfine handles benchmarking orchestration.

**Benefits:**
- No code duplication with hyperfine
- Professional statistical analysis via hyperfine
- Simpler, focused codebase
- Flexible - works with any benchmark orchestrator
- Clear separation of concerns

### Tool Responsibilities

**bench-throughput:**
- Parse templates
- Generate realistic test data
- Execute template operations
- (In all templates mode) Time and report results

**hyperfine:**
- Warmup runs
- Multiple iterations
- Statistical analysis (mean, stddev, min, max, percentiles)
- Comparison between runs
- Export formats (JSON, CSV, Markdown)

## Notes

- Results will vary based on system load and hardware
- The tool uses realistic data patterns to reflect production usage
- Template parsing is done once per execution (not per path)
- All templates are tested with the same generated paths for fairness
