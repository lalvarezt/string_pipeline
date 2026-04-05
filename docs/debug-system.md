# Debug System

This document describes how debug output works in `string_pipeline`.

## Contents

- [Overview](#overview)
- [Enable Debug Output](#enable-debug-output)
- [Output Channels](#output-channels)
- [Output Structure](#output-structure)
- [Example](#example)
- [Common Workflows](#common-workflows)
- [Notes](#notes)

## Overview

Debug mode shows:

- template/session boundaries
- section-level processing for templates with literal text
- cache events (`FAST SPLIT`, `CACHE HIT`, `CACHE MISS`)
- per-operation input/result/timing
- `map` item processing and sub-pipeline steps
- final result timing and cache sizes

Debug output is intended for interactive analysis during template development and troubleshooting.

## Enable Debug Output

You can enable debug mode in two ways.

### Inline debug flag

Add `!` immediately after `{`:

```bash
string-pipeline '{!split:,:..|map:{upper}|join:-}' 'hello,world'
```

### CLI debug flag

Use `--debug` (or `-d`):

```bash
string-pipeline --debug '{split:,:..|map:{upper}|join:-}' 'hello,world'
```

### Quiet mode interaction

`--quiet` suppresses debug logs, even when debug is enabled inline or with `--debug`.

```bash
string-pipeline -d -q '{split:,:..|map:{upper}|join:-}' 'hello,world'
```

## Output Channels

- final formatted value -> `stdout`
- debug lines -> `stderr`

This allows redirecting debug logs independently from the final output.

## Output Structure

The current debug output has a tree-style layout.

### 1) Session header

Includes template string, input, and section counts.

```text
DEBUG: 📂 MULTI-TEMPLATE
DEBUG: ├── 🏁 MULTI-TEMPLATE START
DEBUG: ├── Template: "{!split:,:..|map:{upper}|join:-}"
DEBUG: ├── ➡️ Input: "hello,world"
DEBUG: ├── 1 sections (literal: 0, template: 1)
```

### 2) Section and cache lines

Each template section is listed, then cache behavior is reported.

```text
DEBUG: ├── 📊 SECTION 1/1: [template: split(',', ..) | map { operations: [upper] } | join { sep: "-" }]
DEBUG: ├── 💾 CACHE MISS computing section
```

### 3) Pipeline operations

Shows operation list plus per-step input/result/timing.

```text
DEBUG: │   ├── 🚀 PIPELINE START: 3 operations
DEBUG: │   ├── 1. Split(',')
DEBUG: │   ├── 2. Map(1)
DEBUG: │   ├── 3. Join('-')
DEBUG: │   ├── ⚙️ Step 1: Split
DEBUG: │   │   ├── ➡️ Input: String(hello,world)
DEBUG: │   │   ├── 🎯 Result: List["hello", "world"]
DEBUG: │   │   └── Time: ...
```

### 4) Map item sub-pipelines

For `map`, each item is traced with its own sub-pipeline.

```text
DEBUG: │   │   ├── 🗂️ Item 1/2
DEBUG: │   │   │   ├── ➡️ Input: "hello"
DEBUG: │   │   │   ├── 📂 Sub-Pipeline
DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 1 operations
DEBUG: │   │   │   │   ├── ⚙️ Step 1: Upper
DEBUG: │   │   │   │   └── Time: ...
DEBUG: │   │   │   └── Output: "HELLO"
```

### 5) Session footer

Final section includes total elapsed time and cache sizes.

```text
DEBUG: ├── 🏁 ✅ MULTI-TEMPLATE COMPLETE
DEBUG: ├── 🎯 Final result: "HELLO-WORLD"
DEBUG: ├── Total execution time: ...
DEBUG: └── Cache stats: <regex_count> regex patterns, <split_count> split operations cached
```

## Example

```bash
string-pipeline '{!split:,:..|map:{upper}|join:-}' 'hello,world'
```

Final output (`stdout`):

```text
HELLO-WORLD
```

Debug output (`stderr`) contains the tree structure shown above.

## Common Workflows

### Verify operation ordering

```bash
string-pipeline -d '{trim|split: :..|map:{upper}|join:_}' '  hello world  '
```

Use step output to confirm that each operation receives the expected input type.

### Inspect `map` behavior

```bash
string-pipeline -d '{split:,:..|map:{split: :..|join:-}}' 'hello world,foo bar'
```

Use item-level traces to verify each sub-pipeline result.

### Check cache reuse in templates with literal text

```bash
string-pipeline -d 'A:{split:,:0|upper} B:{split:,:1|upper} C:{split:,:0|upper}' 'x,y,z'
```

Look for `CACHE HIT` on repeated sections.

### Keep result-only output while forcing debug mode

```bash
string-pipeline -d -q '{split:,:..|map:{upper}|join:-}' 'a,b,c'
```

Useful in scripts where debug might be enabled but logs should stay suppressed.

## Notes

- Parse failures happen before execution, so step-level debug output is not available for invalid templates.
- Timing values depend on hardware, OS scheduling, and load.
- Debug output format is intended for humans and may change between versions.

Related documentation:

- `docs/template-system.md`
- `docs/command-line-options.md`
- `docs/benchmarking.md`
