#!/bin/bash
set -euo pipefail

# Analyze all 26 templates with full statistical confidence
# Uses hyperfine's --parameter-list to run efficiently

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/string_pipeline/benchmarks"

usage() {
  cat <<EOF
Usage: $(basename "$0") <baseline-sha> <current-sha> [options]

Analyze all 30 predefined templates with statistical confidence.

Arguments:
  baseline-sha    Git SHA/ref for baseline version
  current-sha     Git SHA/ref for current version

Options:
  --size <n>      Input size in paths (default: 10000)
  --warmup <n>    Number of warmup runs (default: 5)
  --runs <n>      Number of benchmark runs (default: 50)
  --export-dir    Directory for output files (default: ./template_analysis)

Examples:
  $(basename "$0") abc1234 def5678
  $(basename "$0") main HEAD --size 50000 --runs 100
  $(basename "$0") v1.0.0 v1.1.0 --export-dir ./results

Output:
  - Hyperfine JSON for baseline and current versions
  - Markdown comparison report with per-template analysis
  - Statistical confidence for each template
EOF
  exit 1
}

# Default values
SIZE="10000"
WARMUP=5
RUNS=50
EXPORT_DIR="./template_analysis"

# Parse arguments
if [ $# -lt 2 ]; then
  usage
fi

BASELINE_SHA="$1"
CURRENT_SHA="$2"
shift 2

while [ $# -gt 0 ]; do
  case "$1" in
    --size)
      SIZE="$2"
      shift 2
      ;;
    --warmup)
      WARMUP="$2"
      shift 2
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    --export-dir)
      EXPORT_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      ;;
    *)
      echo "Error: Unknown option $1"
      usage
      ;;
  esac
done

# Check binaries exist
BASELINE_BIN="$BENCH_DIR/bench_throughput_$BASELINE_SHA"
CURRENT_BIN="$BENCH_DIR/bench_throughput_$CURRENT_SHA"

if [ ! -f "$BASELINE_BIN" ]; then
  echo "Error: Baseline binary not found: $BASELINE_BIN"
  echo "Run compile_benchmark_versions.sh first"
  exit 1
fi

if [ ! -f "$CURRENT_BIN" ]; then
  echo "Error: Current binary not found: $CURRENT_BIN"
  echo "Run compile_benchmark_versions.sh first"
  exit 1
fi

# Create export directory
mkdir -p "$EXPORT_DIR"

# All 30 predefined templates (from bench-throughput.rs TemplateSet)
# Designed for path data - realistic path manipulation operations
TEMPLATES=(
  # String Operations (direct, no split needed)
  "{upper}"
  "{lower}"
  "{reverse}"
  "{replace:s/\\.txt$/.md/}"
  "{replace:s/\\/\\/+/\\//g}"
  "{regex_extract:[^/]+$}"
  "{strip_ansi}"
  "{trim}"
  "{pad:80: :right}"
  "{pad:80:_:left}"
  # Split Operations
  "{split:/:..}"
  "{split:/:-1}"
  "{split:/:0..-1}"
  "{split:/:0..3}"
  # List Operations on path components
  "{split:/:..|join:/}"
  "{split:/:..|filter:^[a-z]+$}"
  "{split:/:..|filter_not:^\\.}"
  "{split:/:..|sort}"
  "{split:/:..|sort:desc}"
  "{split:/:..|unique}"
  "{split:/:..|slice:2..5}"
  "{split:/:..|slice:-3..}"
  "{split:/:..|map:{upper}}"
  "{split:/:..|map:{trim}}"
  # Real-world path manipulation
  "{split:/:-1|split:.:0}"
  "{split:/:-1|split:.:-1}"
  "{split:/:0..-1|join:/}"
  "{split:/:..|map:{upper}|join:/}"
  "{replace:s/\\/\\/+/\\//g|lower}"
  "{split:/:..|slice:-3..|join: > }"
)

# Convert array to comma-separated list for hyperfine
TEMPLATE_LIST=$(IFS=,; echo "${TEMPLATES[*]}")

echo "========================================="
echo "Per-Template Benchmark Analysis"
echo "========================================="
echo "Baseline:    $BASELINE_SHA"
echo "Current:     $CURRENT_SHA"
echo "Templates:   30 predefined templates"
echo "Input size:  $SIZE paths"
echo "Warmup:      $WARMUP runs"
echo "Runs:        $RUNS measurements"
echo "Output dir:  $EXPORT_DIR"
echo "========================================="
echo ""

# Run hyperfine for baseline version (all templates)
echo "Phase 1/3: Benchmarking baseline version ($BASELINE_SHA)..."
hyperfine \
  --warmup "$WARMUP" \
  --runs "$RUNS" \
  --parameter-list template "$TEMPLATE_LIST" \
  --export-json "$EXPORT_DIR/baseline_results.json" \
  --style basic \
  "$BASELINE_BIN --template {template} --size $SIZE --output /dev/null"

echo ""
echo "Phase 2/3: Benchmarking current version ($CURRENT_SHA)..."
# Run hyperfine for current version (all templates)
hyperfine \
  --warmup "$WARMUP" \
  --runs "$RUNS" \
  --parameter-list template "$TEMPLATE_LIST" \
  --export-json "$EXPORT_DIR/current_results.json" \
  --style basic \
  "$CURRENT_BIN --template {template} --size $SIZE --output /dev/null"

echo ""
echo "Phase 3/3: Generating comparison report..."

# Generate comparison report using Python
python3 "$SCRIPT_DIR/compare_template_results.py" \
  "$EXPORT_DIR/baseline_results.json" \
  "$EXPORT_DIR/current_results.json" \
  --baseline-name "$BASELINE_SHA" \
  --current-name "$CURRENT_SHA" \
  --size "$SIZE" \
  > "$EXPORT_DIR/comparison_report.md"

echo ""
echo "✓ Analysis complete!"
echo ""
echo "Results:"
echo "  - Baseline data:  $EXPORT_DIR/baseline_results.json"
echo "  - Current data:   $EXPORT_DIR/current_results.json"
echo "  - Report:         $EXPORT_DIR/comparison_report.md"
echo ""
echo "View report:"
echo "  cat $EXPORT_DIR/comparison_report.md"
