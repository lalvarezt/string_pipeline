#!/usr/bin/env bash

set -euo pipefail

# Script to compare two compiled benchmark binaries using hyperfine
# This makes it easy to see performance differences between versions

BENCH_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/string_pipeline/benchmarks"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
WARMUP=5
RUNS=50
SIZE="10000"

# Usage information
usage() {
  cat <<EOF
Usage: $(basename "$0") <sha1> <sha2> [OPTIONS]

Compare performance of two compiled benchmark binaries using hyperfine.

The script will run both benchmark versions with the same parameters and
use hyperfine to measure and compare their execution time.

ARGUMENTS:
    <sha1>              Short SHA of first benchmark version (baseline)
    <sha2>              Short SHA of second benchmark version (current)

OPTIONS:
    --warmup N          Number of warmup runs (default: $WARMUP)
    --runs N            Number of benchmark runs (default: $RUNS)
    --size SIZE         Input size (default: $SIZE)
    -h, --help          Show this help message

EXAMPLES:
    # Basic comparison with defaults
    $(basename "$0") 78594af c5a8a11

    # Custom warmup and runs
    $(basename "$0") 78594af c5a8a11 --warmup 5 --runs 20

    # Compare with specific size
    $(basename "$0") abc1234 def5678 --size 50000

NOTES:
    - Binaries must be compiled first using compile_benchmark_versions.sh
    - Both binaries will be run with the same benchmark parameters
    - hyperfine must be installed (https://github.com/sharkdp/hyperfine)
    - Results show execution time comparison, not benchmark throughput
EOF
}

# Print colored message
log_info() {
  echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
  echo -e "${GREEN}✓${NC} $*"
}

log_error() {
  echo -e "${RED}✗${NC} $*" >&2
}

# Check if hyperfine is installed
check_hyperfine() {
  if ! command -v hyperfine &>/dev/null; then
    log_error "hyperfine is not installed"
    echo ""
    echo "Install hyperfine:"
    echo "  - Debian/Ubuntu: apt install hyperfine"
    echo "  - macOS: brew install hyperfine"
    echo "  - Cargo: cargo install hyperfine"
    echo "  - GitHub: https://github.com/sharkdp/hyperfine"
    echo ""
    exit 1
  fi
}

# Check if binary exists
check_binary() {
  local sha=$1
  local binary_path="$BENCH_DIR/bench_throughput_$sha"

  if [ ! -f "$binary_path" ]; then
    log_error "Benchmark binary not found: bench_throughput_$sha"
    echo ""
    echo "The binary for commit $sha has not been compiled yet."
    echo ""
    echo "Compile it first using:"
    echo -e "  ${YELLOW}./scripts/compile_benchmark_versions.sh --start $sha --end $sha${NC}"
    echo ""
    echo "Or compile a range of versions:"
    echo -e "  ${YELLOW}./scripts/compile_benchmark_versions.sh${NC}"
    echo ""
    exit 1
  fi
}

# Parse command line arguments
if [ $# -lt 2 ]; then
  usage
  exit 1
fi

SHA1=$1
SHA2=$2
shift 2

while [ $# -gt 0 ]; do
  case $1 in
  --warmup)
    WARMUP="$2"
    shift 2
    ;;
  --runs)
    RUNS="$2"
    shift 2
    ;;
  --size)
    SIZE="$2"
    shift 2
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    log_error "Unknown option: $1"
    echo ""
    usage
    exit 1
    ;;
  esac
done

# Validate inputs
check_hyperfine
check_binary "$SHA1"
check_binary "$SHA2"

BINARY1="$BENCH_DIR/bench_throughput_$SHA1"
BINARY2="$BENCH_DIR/bench_throughput_$SHA2"

# Prepare hyperfine command
HYPERFINE_ARGS=(
  "--warmup" "$WARMUP"
  "--runs" "$RUNS"
)

# Print comparison info
echo ""
log_info "Comparing benchmark versions using hyperfine"
echo ""
echo "  Baseline: $SHA1"
echo "  Current:  $SHA2"
echo ""
echo "Benchmark parameters:"
echo "  Size:      $SIZE"
echo ""
echo "Hyperfine parameters:"
echo "  Warmup runs:     $WARMUP"
echo "  Benchmark runs:  $RUNS"
echo ""

# Run hyperfine comparison
hyperfine \
  "${HYPERFINE_ARGS[@]}" \
  --command-name "$SHA1" \
  "$BINARY1 --sizes $SIZE --output /dev/null" \
  --command-name "$SHA2" \
  "$BINARY2 --sizes $SIZE --output /dev/null"

echo ""
log_success "Comparison complete!"
