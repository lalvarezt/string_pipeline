#!/usr/bin/env python3
"""
Compare hyperfine JSON results for per-template analysis.

This script parses two hyperfine JSON files (baseline and current) where each
file contains results from running all 26 templates. It generates a markdown
comparison report with statistical confidence.
"""

import json
import sys
import argparse
from typing import Dict, List, Tuple


def parse_args():
    parser = argparse.ArgumentParser(
        description="Compare hyperfine per-template results"
    )
    parser.add_argument("baseline_json", help="Baseline hyperfine JSON results")
    parser.add_argument("current_json", help="Current hyperfine JSON results")
    parser.add_argument(
        "--baseline-name", default="baseline", help="Name for baseline version"
    )
    parser.add_argument(
        "--current-name", default="current", help="Name for current version"
    )
    parser.add_argument("--size", type=int, help="Input size used")
    return parser.parse_args()


def load_hyperfine_json(filepath: str) -> Dict:
    """Load hyperfine JSON results."""
    with open(filepath, "r") as f:
        return json.load(f)


def extract_template_from_command(command: str) -> str:
    """Extract template string from hyperfine command.

    Command format: 'binary --template {template} --size N --output /dev/null'
    """
    parts = command.split("--template ")
    if len(parts) < 2:
        return "unknown"

    template_part = parts[1].split(" ")[0]
    return template_part


def format_time_ms(seconds: float) -> str:
    """Format time in seconds to human-readable string."""
    ms = seconds * 1000
    if ms < 1:
        return f"{ms * 1000:.2f}μs"
    elif ms < 1000:
        return f"{ms:.2f}ms"
    else:
        return f"{ms / 1000:.2f}s"


def calculate_change(baseline: float, current: float) -> Tuple[float, str]:
    """Calculate percentage change and return emoji indicator.

    For timing metrics, lower is better:
    - Negative change = improvement (faster)
    - Positive change = regression (slower)
    """
    if baseline == 0:
        return 0.0, "➖"

    change_pct = ((current - baseline) / baseline) * 100

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


def generate_comparison_report(
    baseline_data: Dict,
    current_data: Dict,
    baseline_name: str,
    current_name: str,
    input_size: int = None,
) -> str:
    """Generate markdown comparison report from hyperfine JSON data."""

    # Build lookup by template
    baseline_by_template = {}
    for result in baseline_data["results"]:
        template = extract_template_from_command(result["command"])
        baseline_by_template[template] = result

    current_by_template = {}
    for result in current_data["results"]:
        template = extract_template_from_command(result["command"])
        current_by_template[template] = result

    # Find common templates
    common_templates = sorted(
        set(baseline_by_template.keys()) & set(current_by_template.keys())
    )

    if not common_templates:
        return "Error: No common templates found between baseline and current results."

    # Generate report
    lines = []
    lines.append("# 📊 Per-Template Benchmark Analysis\n")
    lines.append(f"**Baseline:** `{baseline_name}`")
    lines.append(f"**Current:** `{current_name}`")
    if input_size:
        lines.append(f"**Input size:** {input_size:,} paths per run")
    lines.append(f"**Templates analyzed:** {len(common_templates)}\n")

    # Summary statistics
    regressions = []
    improvements = []
    neutral = []

    # Build comparison table
    lines.append("## Performance Comparison\n")
    lines.append(
        "| Template | Baseline Mean | Current Mean | Change | Min | Max | StdDev | Notes |"
    )
    lines.append(
        "|----------|---------------|--------------|--------|-----|-----|--------|-------|"
    )

    for template in common_templates:
        baseline = baseline_by_template[template]
        current = current_by_template[template]

        # Extract timing statistics (all in seconds from hyperfine)
        baseline_mean = baseline["mean"]
        current_mean = current["mean"]
        current_min = current["min"]
        current_max = current["max"]
        current_stddev = current["stddev"]

        # Calculate change
        change_pct, emoji = calculate_change(baseline_mean, current_mean)

        # Track significant changes
        if change_pct > 10:
            regressions.append((template, change_pct))
        elif change_pct < -5:
            improvements.append((template, change_pct))
        else:
            neutral.append(template)

        # Build notes (check if variation is high)
        notes = []
        cv = (current_stddev / current_mean * 100) if current_mean > 0 else 0
        if cv > 10:
            notes.append("high variance")

        # Format timing data
        baseline_str = format_time_ms(baseline_mean)
        current_str = format_time_ms(current_mean)
        min_str = format_time_ms(current_min)
        max_str = format_time_ms(current_max)
        stddev_str = format_time_ms(current_stddev)

        notes_str = ", ".join(notes) if notes else "—"

        lines.append(
            f"| `{template}` "
            f"| {baseline_str} "
            f"| {current_str} "
            f"| {emoji} {change_pct:+.1f}% "
            f"| {min_str} "
            f"| {max_str} "
            f"| ±{stddev_str} "
            f"| {notes_str} |"
        )

    lines.append("")

    # Summary section
    lines.append("## Summary\n")
    lines.append(f"- **Total templates:** {len(common_templates)}")
    lines.append(f"- **Improvements:** {len(improvements)} 🟢")
    lines.append(f"- **Regressions:** {len(regressions)} 🔴")
    lines.append(f"- **Neutral:** {len(neutral)} ➖\n")

    # Highlight significant changes
    if regressions:
        lines.append("### ⚠️ Performance Regressions\n")
        for template, change in sorted(regressions, key=lambda x: x[1], reverse=True):
            baseline = baseline_by_template[template]
            current = current_by_template[template]
            lines.append(
                f"- **`{template}`**: {change:+.1f}% slower "
                f"({format_time_ms(baseline['mean'])} → {format_time_ms(current['mean'])})"
            )
        lines.append("")

    if improvements:
        lines.append("### ✨ Performance Improvements\n")
        for template, change in sorted(improvements, key=lambda x: x[1]):
            baseline = baseline_by_template[template]
            current = current_by_template[template]
            lines.append(
                f"- **`{template}`**: {abs(change):.1f}% faster "
                f"({format_time_ms(baseline['mean'])} → {format_time_ms(current['mean'])})"
            )
        lines.append("")

    # Statistical notes
    lines.append("## Statistical Notes\n")
    lines.append(
        "All measurements include statistical confidence from hyperfine:"
    )
    lines.append("- **Mean**: Average execution time across all runs")
    lines.append("- **Min/Max**: Fastest and slowest runs observed")
    lines.append("- **StdDev**: Standard deviation (measure of consistency)")
    lines.append(
        "- **High variance**: Templates with coefficient of variation >10%\n"
    )

    # Legend
    lines.append("---\n")
    lines.append("### Legend")
    lines.append("- 🟢 Significant improvement (>5% faster)")
    lines.append("- ✅ Improvement (2-5% faster)")
    lines.append("- ➖ Neutral (<2% change)")
    lines.append("- 🟡 Caution (2-5% slower)")
    lines.append("- ⚠️ Warning (5-10% slower)")
    lines.append("- 🔴 Regression (>10% slower)")

    return "\n".join(lines)


def main():
    args = parse_args()

    try:
        baseline_data = load_hyperfine_json(args.baseline_json)
        current_data = load_hyperfine_json(args.current_json)

        report = generate_comparison_report(
            baseline_data,
            current_data,
            args.baseline_name,
            args.current_name,
            args.size,
        )

        print(report)

    except FileNotFoundError as e:
        print(f"Error: File not found: {e}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
