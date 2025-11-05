#!/bin/bash
# Test script for bench_throughput binary

set -e

echo "Building bench_throughput..."
cargo build --bin bench_throughput --release

echo ""
echo "==================================="
echo "Test 1: Basic run with default settings"
echo "==================================="
./target/release/bench_throughput --sizes 100,1000 --iterations 10

echo ""
echo "==================================="
echo "Test 2: Detailed profiling mode"
echo "==================================="
./target/release/bench_throughput --sizes 100,1000 --iterations 10 --detailed

echo ""
echo "==================================="
echo "Test 3: JSON output to file"
echo "==================================="
./target/release/bench_throughput --sizes 100,1000 --iterations 10 --detailed --format json --output bench_results.json

echo ""
echo "Checking JSON output..."
if [ -f bench_results.json ]; then
    echo "✓ JSON file created successfully"
    echo "File size: $(wc -c < bench_results.json) bytes"
    head -20 bench_results.json
else
    echo "✗ JSON file not created"
    exit 1
fi

echo ""
echo "==================================="
echo "Test 4: Help output"
echo "==================================="
./target/release/bench_throughput --help

echo ""
echo "All tests passed! ✓"
