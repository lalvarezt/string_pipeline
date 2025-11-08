#!/usr/bin/env python3
"""
Compare benchmark results and generate a markdown report.
Detects performance regressions and improvements.
"""

import json
import sys
from typing import Dict, Tuple
from pathlib import Path


def format_duration_ns(ns: int) -> str:
    """Format nanoseconds to human-readable duration."""
    if ns < 1_000:
        return f"{ns}ns"
    elif ns < 1_000_000:
        return f"{ns / 1_000:.2f}μs"
    elif ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f}ms"
    else:
        return f"{ns / 1_000_000_000:.2f}s"


def format_throughput(paths_per_sec: float) -> str:
    """Format throughput to human-readable format."""
    if paths_per_sec >= 1_000_000:
        return f"{paths_per_sec / 1_000_000:.2f}M/s"
    elif paths_per_sec >= 1_000:
        return f"{paths_per_sec / 1_000:.2f}K/s"
    else:
        return f"{paths_per_sec:.2f}/s"


def calculate_change(baseline: float, current: float) -> Tuple[float, str]:
    """Calculate percentage change and return emoji indicator."""
    if baseline == 0:
        return 0.0, "➖"

    change_pct = ((current - baseline) / baseline) * 100

    # For latency metrics (lower is better)
    if abs(change_pct) < 2:  # Less than 2% change is noise
        emoji = "➖"
    elif change_pct < -5:  # >5% faster is significant improvement
        emoji = "🟢"
    elif change_pct < -2:  # 2-5% faster is improvement
        emoji = "✅"
    elif change_pct > 10:  # >10% slower is regression
        emoji = "🔴"
    elif change_pct > 5:  # 5-10% slower is warning
        emoji = "⚠️"
    else:  # 2-5% slower is caution
        emoji = "🟡"

    return change_pct, emoji


def load_benchmark_results(filepath: str) -> Dict:
    """Load benchmark results from JSON file."""
    with open(filepath, "r") as f:
        return json.load(f)


def compare_benchmarks(baseline_path: str, current_path: str) -> str:
    """Compare two benchmark results and generate markdown report."""
    baseline = load_benchmark_results(baseline_path)
    current = load_benchmark_results(current_path)

    # Build lookup dictionaries for easier comparison
    baseline_results = {}
    for bench in baseline["benchmarks"]:
        template_name = bench["template_name"]
        # Get the largest input size result
        if bench["results"]:
            baseline_results[template_name] = bench["results"][-1]

    current_results = {}
    for bench in current["benchmarks"]:
        template_name = bench["template_name"]
        if bench["results"]:
            current_results[template_name] = bench["results"][-1]

    # Generate report
    report = []
    report.append("# 📊 Benchmark Comparison Report\n")

    # Get input size from first template
    input_size = 0
    if current["benchmarks"] and current["benchmarks"][0]["results"]:
        input_size = current["benchmarks"][0]["results"][-1]["input_size"]

    report.append(f"**Input Size:** {input_size:,} paths\n")
    report.append(f"**Baseline Timestamp:** {baseline.get('timestamp', 'unknown')}")
    report.append(f"**Current Timestamp:** {current.get('timestamp', 'unknown')}\n")

    # Summary statistics
    regressions = []
    improvements = []
    neutral = []

    # Build comparison table
    report.append("## Performance Comparison\n")
    report.append(
        "| Template | Avg/Path | Change | p95 | Change | Throughput | Change |"
    )
    report.append(
        "|----------|----------|--------|-----|--------|------------|--------|"
    )

    # Sort by template name for consistent ordering
    all_templates = sorted(set(baseline_results.keys()) | set(current_results.keys()))

    for template_name in all_templates:
        if (
            template_name not in baseline_results
            or template_name not in current_results
        ):
            continue  # Skip if not in both sets

        base = baseline_results[template_name]
        curr = current_results[template_name]

        # Compare avg time per path
        base_avg_ns = base["avg_time_per_path"]
        curr_avg_ns = curr["avg_time_per_path"]
        avg_change, avg_emoji = calculate_change(base_avg_ns, curr_avg_ns)

        # Compare p95
        base_p95 = base["latency_stats"]["p95"]
        curr_p95 = curr["latency_stats"]["p95"]
        p95_change, p95_emoji = calculate_change(base_p95, curr_p95)

        # Compare throughput (higher is better, so invert the change)
        base_throughput = base["throughput_paths_per_sec"]
        curr_throughput = curr["throughput_paths_per_sec"]
        throughput_change = (
            (curr_throughput - base_throughput) / base_throughput
        ) * 100
        # Invert emoji logic for throughput
        if abs(throughput_change) < 2:
            throughput_emoji = "➖"
        elif throughput_change > 5:
            throughput_emoji = "🟢"
        elif throughput_change > 2:
            throughput_emoji = "✅"
        elif throughput_change < -10:
            throughput_emoji = "🔴"
        elif throughput_change < -5:
            throughput_emoji = "⚠️"
        elif throughput_change < -2:
            throughput_emoji = "🟡"
        else:
            throughput_emoji = "➖"

        # Track regressions/improvements based on avg latency
        if avg_change > 10:
            regressions.append((template_name, avg_change))
        elif avg_change < -5:
            improvements.append((template_name, avg_change))
        else:
            neutral.append(template_name)

        # Format table row
        report.append(
            f"| {template_name} "
            f"| {format_duration_ns(curr_avg_ns)} "
            f"| {avg_emoji} {avg_change:+.1f}% "
            f"| {format_duration_ns(curr_p95)} "
            f"| {p95_emoji} {p95_change:+.1f}% "
            f"| {format_throughput(curr_throughput)} "
            f"| {throughput_emoji} {throughput_change:+.1f}% |"
        )

    report.append("")

    # Summary section
    report.append("## Summary\n")
    report.append(f"- **Total templates compared:** {len(all_templates)}")
    report.append(f"- **Improvements:** {len(improvements)} 🟢")
    report.append(f"- **Regressions:** {len(regressions)} 🔴")
    report.append(f"- **Neutral:** {len(neutral)} ➖\n")

    # Highlight significant changes
    if regressions:
        report.append("### ⚠️ PERFORMANCE REGRESSIONS\n")
        for template, change in sorted(regressions, key=lambda x: x[1], reverse=True):
            report.append(f"- **{template}**: {change:+.1f}% slower")
        report.append("")

    if improvements:
        report.append("### ✨ Performance Improvements\n")
        for template, change in sorted(improvements, key=lambda x: x[1]):
            report.append(f"- **{template}**: {abs(change):.1f}% faster")
        report.append("")

    # Legend
    report.append("---\n")
    report.append("### Legend")
    report.append("- 🟢 Significant improvement (>5% faster)")
    report.append("- ✅ Improvement (2-5% faster)")
    report.append("- ➖ Neutral (<2% change)")
    report.append("- 🟡 Caution (2-5% slower)")
    report.append("- ⚠️ Warning (5-10% slower)")
    report.append("- 🔴 Regression (>10% slower)")

    return "\n".join(report)


def main():
    if len(sys.argv) != 3:
        print("Usage: compare_benchmarks.py <baseline.json> <current.json>")
        sys.exit(1)

    baseline_path = sys.argv[1]
    current_path = sys.argv[2]

    if not Path(baseline_path).exists():
        print(f"Error: Baseline file not found: {baseline_path}")
        sys.exit(1)

    if not Path(current_path).exists():
        print(f"Error: Current file not found: {current_path}")
        sys.exit(1)

    try:
        report = compare_benchmarks(baseline_path, current_path)
        print(report)
    except Exception as e:
        print(f"Error comparing benchmarks: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
