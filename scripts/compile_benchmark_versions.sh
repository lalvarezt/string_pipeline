#!/usr/bin/env bash

set -euo pipefail

# Script to compile benchmark binaries for multiple git commits
# This makes it easy to compare performance across different versions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCH_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/string_pipeline/benchmarks"
DEFAULT_START_COMMIT="78594af" # First commit with stabilized bench-throughput tool (v1.0.0)
VERBOSE=0
DRY_RUN=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Usage information
usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Compile the throughput benchmark for multiple git commits to enable
version-to-version performance comparisons.

Compiled binaries are stored as: bench_throughput_<shortsha>
Location: \$XDG_DATA_HOME/string_pipeline/benchmarks/

OPTIONS:
    --start COMMIT      Starting commit (default: $DEFAULT_START_COMMIT)
    --end COMMIT        Ending commit (default: HEAD)
    --list              List already compiled versions and exit
    --dry-run           Show what would be compiled without doing it
    --clean             Remove all compiled benchmarks and exit
    --verbose           Show detailed output
    -h, --help          Show this help message

EXAMPLES:
    # Compile all versions from 78594af (stabilized benchmark tool) to HEAD
    $(basename "$0")

    # Compile specific range
    $(basename "$0") --start abc1234 --end def5678

    # List available compiled versions
    $(basename "$0") --list

    # See what would be compiled
    $(basename "$0") --dry-run

    # Clean up old compiled versions
    $(basename "$0") --clean

USAGE AFTER COMPILATION:
    # Quick overall comparison with hyperfine
    ./scripts/compare_benchmark_versions.sh abc1234 def5678 --all

    # Detailed per-template analysis with statistical confidence
    ./scripts/analyze_all_templates.sh abc1234 def5678 --runs 100

    # Analyze specific template
    ./scripts/compare_benchmark_versions.sh abc1234 def5678 \\
        --template "{split:/:-1}" --runs 100
EOF
}

# Print colored message
log_info() {
  echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
  echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
  echo -e "${YELLOW}⚠${NC} $*"
}

log_error() {
  echo -e "${RED}✗${NC} $*" >&2
}

log_verbose() {
  if [ "$VERBOSE" -eq 1 ]; then
    echo -e "${BLUE}[verbose]${NC} $*"
  fi
}

# List compiled versions
list_versions() {
  if [ ! -d "$BENCH_DIR" ]; then
    log_warning "No benchmark directory found at: $BENCH_DIR"
    return
  fi

  local count=0
  log_info "Compiled benchmark versions in: $BENCH_DIR"
  echo ""

  while IFS= read -r -d '' binary; do
    local filename
    filename=$(basename "$binary")
    local sha="${filename#bench_throughput_}"
    local size
    size=$(du -h "$binary" | cut -f1)
    local date
    date=$(stat -c '%y' "$binary" 2>/dev/null || stat -f '%Sm' "$binary" 2>/dev/null || echo "unknown")

    echo "  $sha  ($size, compiled: ${date%.*})"
    count=$((count + 1))
  done < <(find "$BENCH_DIR" -type f -name "bench_throughput_*" -print0 2>/dev/null | sort -z)

  if [ "$count" -eq 0 ]; then
    log_warning "No compiled benchmarks found"
  else
    echo ""
    log_success "Found $count compiled version(s)"
  fi
}

# Clean compiled versions
clean_versions() {
  if [ ! -d "$BENCH_DIR" ]; then
    log_warning "No benchmark directory found at: $BENCH_DIR"
    return
  fi

  local count=0
  while IFS= read -r -d '' binary; do
    log_verbose "Removing: $binary"
    rm -f "$binary"
    count=$((count + 1))
  done < <(find "$BENCH_DIR" -type f -name "bench_throughput_*" -print0 2>/dev/null)

  if [ "$count" -eq 0 ]; then
    log_info "No compiled benchmarks to clean"
  else
    log_success "Removed $count compiled version(s)"
  fi
}

# Get short SHA for a commit
get_short_sha() {
  local commit=$1
  git rev-parse --short=7 "$commit" 2>/dev/null
}

# Check if binary exists for a commit
binary_exists() {
  local short_sha=$1
  [ -f "$BENCH_DIR/bench_throughput_$short_sha" ]
}

# Compile benchmark for a commit using git worktree
compile_for_commit() {
  local commit=$1
  local short_sha=$2
  local binary_path="$BENCH_DIR/bench_throughput_$short_sha"

  if binary_exists "$short_sha"; then
    log_verbose "Skipping $short_sha (already compiled)"
    return 0
  fi

  log_info "Compiling $short_sha..."

  if [ "$DRY_RUN" -eq 1 ]; then
    echo "  [DRY RUN] Would create worktree for $commit and compile"
    return 0
  fi

  # Create temporary directory for worktree
  local worktree_dir
  worktree_dir=$(mktemp -d -t "bench_compile_${short_sha}_XXXXXX")

  log_verbose "Created worktree directory: $worktree_dir"

  # Add worktree for this commit
  if ! git worktree add -q --detach "$worktree_dir" "$commit" 2>/dev/null; then
    log_error "Failed to create worktree for $commit"
    rm -rf "$worktree_dir"
    return 1
  fi

  # Try to compile in the worktree
  local compile_success=0
  if (cd "$worktree_dir" && cargo build --release --bin bench-throughput >/dev/null 2>&1); then
    # Copy binary to benchmark directory
    if [ -f "$worktree_dir/target/release/bench-throughput" ]; then
      cp "$worktree_dir/target/release/bench-throughput" "$binary_path"
      log_success "Compiled $short_sha"
      compile_success=1
    else
      log_error "Binary not found after compilation for $short_sha"
    fi
  else
    log_warning "Compilation failed for $short_sha"
  fi

  # Cleanup worktree
  log_verbose "Cleaning up worktree for $short_sha"
  git worktree remove --force "$worktree_dir" 2>/dev/null || true
  rm -rf "$worktree_dir"

  [ "$compile_success" -eq 1 ]
}

# Main compilation logic
compile_versions() {
  local start_commit=$1
  local end_commit=$2

  # Verify commits exist
  if ! git rev-parse "$start_commit" >/dev/null 2>&1; then
    log_error "Invalid start commit: $start_commit"
    exit 1
  fi

  if ! git rev-parse "$end_commit" >/dev/null 2>&1; then
    log_error "Invalid end commit: $end_commit"
    exit 1
  fi

  # Create benchmark directory
  mkdir -p "$BENCH_DIR"

  # Get list of commits
  log_info "Collecting commits from $start_commit to $end_commit..."
  local commits
  mapfile -t commits < <(git rev-list --reverse "$start_commit^..$end_commit")

  local total=${#commits[@]}
  log_info "Found $total commit(s) to process"
  echo ""

  # Counters
  local compiled=0
  local skipped=0
  local failed=0

  # Process each commit
  for commit in "${commits[@]}"; do
    local short_sha
    short_sha=$(get_short_sha "$commit")

    if binary_exists "$short_sha"; then
      log_success "[$((compiled + skipped + failed + 1))/$total] $short_sha (already exists)"
      skipped=$((skipped + 1))
    else
      echo -n "[$((compiled + skipped + failed + 1))/$total] "
      if compile_for_commit "$commit" "$short_sha"; then
        compiled=$((compiled + 1))
      else
        failed=$((failed + 1))
      fi
    fi
  done

  # Print summary
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Summary:"
  echo "  Total commits:      $total"
  echo "  Newly compiled:     $compiled"
  echo "  Already compiled:   $skipped"
  echo "  Failed:             $failed"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  log_info "Binaries location: $BENCH_DIR"

  if [ "$compiled" -gt 0 ] || [ "$skipped" -gt 0 ]; then
    echo ""
    log_success "Ready for version comparison!"
    echo ""
    echo "Example usage:"
    echo "  # Run benchmark with a specific version"
    local example_sha
    example_sha=$(get_short_sha "$end_commit")
    echo "  $BENCH_DIR/bench_throughput_$example_sha \\"
    echo "    --sizes 1000,5000,10000 --iterations 100 \\"
    echo "    --output results.json"
  fi
}

# Parse command line arguments
START_COMMIT="$DEFAULT_START_COMMIT"
END_COMMIT="HEAD"
ACTION="compile"

while [ $# -gt 0 ]; do
  case $1 in
  --start)
    START_COMMIT="$2"
    shift 2
    ;;
  --end)
    END_COMMIT="$2"
    shift 2
    ;;
  --list)
    ACTION="list"
    shift
    ;;
  --clean)
    ACTION="clean"
    shift
    ;;
  --dry-run)
    DRY_RUN=1
    shift
    ;;
  --verbose)
    VERBOSE=1
    shift
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

# Change to project root
cd "$PROJECT_ROOT"

# Execute action
case $ACTION in
list)
  list_versions
  ;;
clean)
  clean_versions
  ;;
compile)
  compile_versions "$START_COMMIT" "$END_COMMIT"
  ;;
esac
