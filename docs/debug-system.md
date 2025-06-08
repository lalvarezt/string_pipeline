# 🐛 Debug System Documentation

A comprehensive debugging system for visualizing, analyzing, and troubleshooting string pipeline transformations with detailed execution insights and performance analysis.

## 📋 Table of Contents

- [🌟 Overview](#-overview)
- [🚀 Quick Start](#-quick-start)
- [🔧 Enabling Debug Mode](#-enabling-debug-mode)
  - [Inline Debug Syntax](#inline-debug-syntax)
  - [CLI Debug Flag](#cli-debug-flag)
  - [Debug Mode Comparison](#debug-mode-comparison)
- [🔍 Understanding Debug Output Structure](#-understanding-debug-output-structure)
  - [Hierarchical Debug Architecture](#hierarchical-debug-architecture)
  - [Visual Hierarchy System](#visual-hierarchy-system)
- [🗺️ Complex Pipeline Debugging](#️-complex-pipeline-debugging)
  - [Simple Map Operations](#simple-map-operations)
  - [Multi-Step Map Pipelines](#multi-step-map-pipelines)
  - [List Operations in Maps](#list-operations-in-maps)
- [⚡ Performance Analysis](#-performance-analysis)
  - [Timing Information](#timing-information)
  - [Memory Usage Tracking](#memory-usage-tracking)
  - [Bottleneck Identification](#bottleneck-identification)
  - [Optimization Insights](#optimization-insights)
- [🚨 Error Debugging](#-error-debugging)
  - [Parse Error Analysis](#parse-error-analysis)
  - [Runtime Error Diagnosis](#runtime-error-diagnosis)

## 🌟 Overview

The String Pipeline debug system provides comprehensive visibility into template execution, making it easy to understand data flow, identify performance bottlenecks, and troubleshoot complex transformations. Whether you're developing new templates or optimizing existing ones, the debug system offers detailed insights at every step.

### ✨ Key Features

- **🔍 Step-by-Step Execution Tracking** - See exactly how data flows through each operation
- **🗺️ Map Operation Visualization** - Detailed per-item processing in map operations with full sub-pipeline execution
- **⚡ Performance Metrics** - Timing and memory usage for every operation and sub-operation
- **🎯 Hierarchical Display** - Clear visual structure showing operation nesting and sub-pipelines
- **🚨 Error Context** - Detailed error information with operation context
- **📊 Data Type Visualization** - See how values transform between types with detailed formatting
- **🔧 Flexible Activation** - Enable via template syntax or CLI flags
- **💾 Cache Statistics** - Monitor regex and split operation cache performance

### 🎯 When to Use Debug Mode

| Scenario | Debug Method | Benefits |
|----------|--------------|----------|
| **🧪 Template Development** | Inline `{!...}` | Quick iteration and testing |
| **🔧 Complex Debugging** | CLI `--debug` | Systematic analysis |
| **⚡ Performance Analysis** | Either method | Identify slow operations |
| **🚨 Error Investigation** | CLI `--debug` | Full context information |
| **📚 Learning Templates** | Inline `{!...}` | Understand operation behavior |
| **🏭 Production Issues** | CLI `--debug --quiet` | Clean diagnostic output |

## 🚀 Quick Start

Get started with debug mode in seconds:

```bash
# 🔍 Basic inline debug
string-pipeline '{!split:,:..|map:{upper}}' 'hello,world'

# 🛠️ CLI debug flag
string-pipeline --debug '{split:,:..|map:{upper}}' 'hello,world'

# 🤫 Quiet debug (result only)
string-pipeline --debug --quiet '{split:,:..|map:{upper}}' 'hello,world'
```

**Quick Example Output:**

```bash
string-pipeline '{!split:,:..|map:{upper}}' 'hello,world'
# See detailed debug output in: Simple Map Operations section below
```

## 🔧 Enabling Debug Mode

### Inline Debug Syntax

Enable debug mode directly in the template using the `!` flag.

**Syntax:** `{!operation1|operation2|...}`

```bash
# 🔍 Basic inline debug
string-pipeline '{!upper}' 'hello world'

# 🔍 Complex pipeline debug
string-pipeline '{!split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}|sort}' '  apple  , hi , banana  '

# 🔍 Map operation debug
string-pipeline '{!split: :..|map:{upper|append:!}}' 'hello world test'
```

**Inline Debug Characteristics:**

| Feature | Behavior | Use Case |
|---------|----------|----------|
| **🎯 Template-Specific** | Debug applies only to that template | Development and testing |
| **📝 Portable** | Debug setting travels with template | Sharing debug templates |
| **🔄 Temporary** | Easy to add/remove for testing | Quick debugging sessions |
| **💾 Saveable** | Can be saved in template files | Reusable debug templates |

### CLI Debug Flag

Enable debug mode using command-line flags.

**Syntax:** `--debug` or `-d`

```bash
# 🛠️ Basic CLI debug
string-pipeline --debug '{upper}' 'hello world'

# 🛠️ Debug with file input
string-pipeline -d '{split:,:..|map:{upper}}' -f data.txt

# 🛠️ Debug with template file
string-pipeline --debug -t transform.template 'input data'

# 🤫 Quiet debug mode
string-pipeline --debug --quiet '{split:,:..|map:{upper}}' 'a,b,c'
```

**CLI Debug Characteristics:**

| Feature | Behavior | Use Case |
|---------|----------|----------|
| **🎛️ External Control** | Debug enabled outside template | Systematic testing |
| **🔄 Reusable** | Same template, debug on/off | Production debugging |
| **🤫 Quiet Option** | Combine with `--quiet` | Clean output |
| **📊 Consistent** | Same debug format for all templates | Standardized analysis |

### Debug Mode Comparison

| Aspect | Inline `{!...}` | CLI `--debug` |
|--------|-----------------|---------------|
| **⚡ Speed** | Quick one-off debugging | Systematic analysis |
| **🔄 Reusability** | Template-specific | Any template |
| **📝 Documentation** | Self-documenting | External control |
| **🏭 Production** | Not recommended | With `--quiet` flag |
| **🧪 Development** | Perfect for iteration | Good for testing |
| **📊 Consistency** | Varies by template | Uniform output |

## 🔍 Understanding Debug Output Structure

### Hierarchical Debug Architecture

The debug system uses a multi-level hierarchical structure to organize execution information, with distinct visual markers and consistent formatting patterns.

#### Level 1: Template Session Container

```text
DEBUG: ═══════════════════════════════════════════════
DEBUG: SINGLE TEMPLATE START
DEBUG: Template: "{!split:,:..|map:{upper}}"
DEBUG: Input: "hello,world"
```

**Container Structure:**

- **Double-line border (`═`)**: Marks boundaries for major execution contexts (templates, pipelines, sub-pipelines)
- **Session header**: Identifies execution context type (`SINGLE TEMPLATE START`, `🚀 PIPELINE START`, `🔧 SUB-PIPELINE START`, etc.)
- **Template declaration**: Shows the raw template syntax being processed (template level only)
- **Initial input**: Displays the starting data with explicit type information

#### Level 2: Pipeline Execution Flow

```text
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 2/3: Applying Map { operations: [Upper] }
DEBUG: Input: List(2 items: ["hello", "world"])
DEBUG: 🎯 Result: List(2 items: ["HELLO", "WORLD"])
DEBUG: Step completed in 10.8ms
```

**Pipeline Structure Elements:**

- **Single-line separator (`─`)**: Separates individual steps within execution contexts
- **Step counter**: `STEP X/Y` format showing current position in operation sequence
- **Operation descriptor**: Detailed operation name with parameters (`Map { operations: [...] }`)
- **Input/Output tracking**: Explicit data flow with type annotations
- **Performance markers**: Step-level timing with `🎯 Result` indicators

#### Level 3: Sub-Pipeline Nesting

```text
DEBUG: ═══════════════════════════════════════════════
DEBUG: 🔧 SUB-PIPELINE START: 1 operations to apply
DEBUG: Initial input: String("hello")
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 1/1: Applying Upper
DEBUG: Input: String("hello")
DEBUG: 🎯 Result: String("HELLO")
DEBUG: Step completed in 20.804µs
DEBUG: ───────────────────────────────────────────────
DEBUG: ✅ SUB-PIPELINE COMPLETE
DEBUG: Total execution time: 67.432µs
DEBUG: 🎯 Final result: String("HELLO")
DEBUG: ═══════════════════════════════════════════════
```

**Nested Structure Characteristics:**

- **Nested container borders**: Sub-pipelines use same `═` borders but with `🔧` prefix
- **Independent step counting**: Sub-pipelines maintain their own `STEP X/Y` sequences
- **Isolated scope indicators**: `SUB-PIPELINE START/COMPLETE` markers clearly delineate nested execution
- **Context preservation**: Main pipeline context is maintained around sub-pipeline blocks

### Visual Hierarchy System

#### Border Significance

| Border Type | Level | Usage | Purpose |
|-------------|-------|-------|---------|
| `═══════════` | Container | Template, Pipeline, and Sub-pipeline boundaries | Major execution context boundaries |
| `───────────` | Step | Operation separators within containers | Individual step separation |
| `DEBUG:` | Prefix | All debug lines | Consistent identification |

#### Icon Semantics

| Icon | Meaning | Context | Information Type |
|------|---------|---------|------------------|
| `🔧` | Sub-pipeline | Nested execution | Indicates recursive processing |
| `🎯` | Result | Output data | Final operation result |
| `✅` | Completion | Section end | Successful execution marker |

#### Data Type Representation

```text
DEBUG: Input: String("hello")           # Simple scalar
DEBUG: Input: List(2 items: [...])      # Collection with count
DEBUG: Input: Object(3 fields: {...})   # Structured data with field count
```

**Type Display Pattern:**

- **Type name**: Explicit Rust-style type identification (`String`, `List`, `Object`)
- **Content preview**: Truncated content for readability
- **Metadata**: Quantitative information (item counts, field counts, byte sizes)
- **Nested formatting**: Multi-line display for complex structures with proper indentation

### Performance Integration

#### Timing Hierarchy

```text
DEBUG: Step completed in 829.4µs        # Individual operation timing
DEBUG: Total execution time: 3.0871ms   # Pipeline-level timing
DEBUG: Cache stats: 0 regex, 1 split    # Resource utilization summary
```

**Performance Structure:**

- **Granular timing**: Every operation receives individual timing measurement
- **Cumulative tracking**: Pipeline and template-level total times
- **Resource metrics**: Cache usage statistics for optimization insights
- **Unit consistency**: Automatic unit scaling (µs, ms, s) based on magnitude

This hierarchical structure enables developers to drill down from high-level template execution to individual operation details while maintaining clear visual separation and consistent information density at each level.

## 🗺️ Complex Pipeline Debugging

### Simple Map Operations

Debug basic map operations with string transformations.

```bash
string-pipeline "{!split:,:..|map:{upper}}" "hello,world"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: SINGLE TEMPLATE START
# DEBUG: Template: "{!split:,:..|map:{upper}}"
# DEBUG: Input: "hello,world"
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🚀 PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String("hello,world")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
# DEBUG:   2. Map { operations: [Upper] }
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Input: String("hello,world")
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "hello"
# DEBUG:   "world"
# DEBUG: ])
# DEBUG: Step completed in 1.09718ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Map { operations: [Upper] }
# DEBUG: Input: List(2 items: ["hello", "world"])
# DEBUG: Processing item 1 of 2
# DEBUG: Map item input: "hello"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 1 operations to apply
# DEBUG: Initial input: String("hello")
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/1: Applying Upper
# DEBUG: Input: String("hello")
# DEBUG: 🎯 Result: String("HELLO")
# DEBUG: Step completed in 35.612µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 103.315µs
# DEBUG: 🎯 Final result: String("HELLO")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "HELLO"
# DEBUG: Processing item 2 of 2
# DEBUG: Map item input: "world"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 1 operations to apply
# DEBUG: Initial input: String("world")
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/1: Applying Upper
# DEBUG: Input: String("world")
# DEBUG: 🎯 Result: String("WORLD")
# DEBUG: Step completed in 3.825µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 23.916µs
# DEBUG: 🎯 Final result: String("WORLD")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "WORLD"
# DEBUG: MAP COMPLETED: 2 → 2 items
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "HELLO"
# DEBUG:   "WORLD"
# DEBUG: ])
# DEBUG: Step completed in 223.781µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ PIPELINE COMPLETE
# DEBUG: Total execution time: 1.763036ms
# DEBUG: 🎯 Final result: List(2 items: [
# DEBUG:   "HELLO"
# DEBUG:   "WORLD"
# DEBUG: ])
# DEBUG: Cache stats: 0 regex patterns, 1 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: ✅ SINGLE TEMPLATE COMPLETE
# DEBUG: 🎯 Final result: "HELLO,WORLD"
# DEBUG: Cache stats: 0 regex patterns, 1 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# HELLO,WORLD
```

**Key Insights:**

- **📊 Item Processing**: Each item processed as a complete sub-pipeline
- **🔄 Transformation**: Clear input → output mapping with detailed steps
- **⚡ Performance**: Individual timing per sub-pipeline execution

### Multi-Step Map Pipelines

Debug complex multi-operation map pipelines.

```bash
string-pipeline "{!split:,:..|map:{trim|upper}}" "  apple  , banana "
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: SINGLE TEMPLATE START
# DEBUG: Template: "{!split:,:..|map:{trim|upper}}"
# DEBUG: Input: "  apple  , banana "
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🚀 PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String("  apple  , banana ")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
# DEBUG:   2. Map { operations: [Trim { chars: "", direction: Both }, Upper] }
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Input: String("  apple  , banana ")
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "  apple  "
# DEBUG:   " banana "
# DEBUG: ])
# DEBUG: Step completed in 1.071951ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Map { operations: [Trim { chars: "", direction: Both }, Upper] }
# DEBUG: Input: List(2 items: ["  apple  ", " banana "])
# DEBUG: Processing item 1 of 2
# DEBUG: Map item input: "  apple  "
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String("  apple  ")
# DEBUG: Operations to apply:
# DEBUG:   1. Trim { chars: "", direction: Both }
# DEBUG:   2. Upper
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Trim { chars: "", direction: Both }
# DEBUG: Input: String("  apple  ")
# DEBUG: 🎯 Result: String("apple")
# DEBUG: Step completed in 10.667µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Upper
# DEBUG: Input: String("apple")
# DEBUG: 🎯 Result: String("APPLE")
# DEBUG: Step completed in 5.091µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 93.585µs
# DEBUG: 🎯 Final result: String("APPLE")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "APPLE"
# DEBUG: Processing item 2 of 2
# DEBUG: Map item input: " banana "
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String(" banana ")
# DEBUG: Operations to apply:
# DEBUG:   1. Trim { chars: "", direction: Both }
# DEBUG:   2. Upper
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Trim { chars: "", direction: Both }
# DEBUG: Input: String(" banana ")
# DEBUG: 🎯 Result: String("banana")
# DEBUG: Step completed in 15.748µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Upper
# DEBUG: Input: String("banana")
# DEBUG: 🎯 Result: String("BANANA")
# DEBUG: Step completed in 15.242µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 90.132µs
# DEBUG: 🎯 Final result: String("BANANA")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "BANANA"
# DEBUG: MAP COMPLETED: 2 → 2 items
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "APPLE"
# DEBUG:   "BANANA"
# DEBUG: ])
# DEBUG: Step completed in 297.991µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ PIPELINE COMPLETE
# DEBUG: Total execution time: 1.806645ms
# DEBUG: 🎯 Final result: List(2 items: [
# DEBUG:   "APPLE"
# DEBUG:   "BANANA"
# DEBUG: ])
# DEBUG: Cache stats: 0 regex patterns, 1 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: ✅ SINGLE TEMPLATE COMPLETE
# DEBUG: 🎯 Final result: "APPLE,BANANA"
# DEBUG: Cache stats: 0 regex patterns, 1 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# APPLE,BANANA
```

**Key Insights:**

- **🔗 Pipeline Flow**: Multi-step transformation per item shown as complete sub-pipeline
- **📊 Data Evolution**: See how data changes at each step with timing information
- **🎯 Operation Chain**: Clear operation sequence with detailed execution trace

### List Operations in Maps

Debug map operations that use list transformations.

```bash
string-pipeline "{!split:,:..|map:{split: :..|unique|sort|join:-}}" "apple banana apple,cherry banana"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: SINGLE TEMPLATE START
# DEBUG: Template: "{!split:,:..|map:{split: :..|unique|sort|join:-}}"
# DEBUG: Input: "apple banana apple,cherry banana"
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🚀 PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String("apple banana apple,cherry banana")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
# DEBUG:   2. Map { operations: [
# DEBUG:       1: Split { sep: " ", range: Range(None, None, false) }
# DEBUG:       2: Unique
# DEBUG:       3: Sort { direction: Asc }
# DEBUG:       4: Join { sep: "-" }
# DEBUG:     ] }
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Input: String("apple banana apple,cherry banana")
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "apple banana apple"
# DEBUG:   "cherry banana"
# DEBUG: ])
# DEBUG: Step completed in 1.619179ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Map (with 4 operations)
# DEBUG: Map { operations: [
# DEBUG:     1: Split { sep: " ", range: Range(None, None, false) }
# DEBUG:     2: Unique
# DEBUG:     3: Sort { direction: Asc }
# DEBUG:     4: Join { sep: "-" }
# DEBUG:   ] }
# DEBUG: Input: List(2 items: ["apple banana apple", "cherry banana"])
# DEBUG: Processing item 1 of 2
# DEBUG: Map item input: "apple banana apple"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 4 operations to apply
# DEBUG: Initial input: String("apple banana apple")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: " ", range: Range(None, None, false) }
# DEBUG:   2. Unique
# DEBUG:   3. Sort { direction: Asc }
# DEBUG:   4. Join { sep: "-" }
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/4: Applying Split { sep: " ", range: Range(None, None, false) }
# DEBUG: Input: String("apple banana apple")
# DEBUG: 🎯 Result: List(3 items: [
# DEBUG:   "apple"
# DEBUG:   "banana"
# DEBUG:   "apple"
# DEBUG: ])
# DEBUG: Step completed in 44.309µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/4: Applying Unique
# DEBUG: Input: List(3 items: ["apple", "banana", "apple"])
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "apple"
# DEBUG:   "banana"
# DEBUG: ])
# DEBUG: Step completed in 39.795µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 3/4: Applying Sort { direction: Asc }
# DEBUG: Input: List(2 items: ["apple", "banana"])
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "apple"
# DEBUG:   "banana"
# DEBUG: ])
# DEBUG: Step completed in 451.904µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 4/4: Applying Join { sep: "-" }
# DEBUG: Input: List(2 items: ["apple", "banana"])
# DEBUG: 🎯 Result: String("apple-banana")
# DEBUG: Step completed in 43.291µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 925.081µs
# DEBUG: 🎯 Final result: String("apple-banana")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "apple-banana"
# DEBUG: Processing item 2 of 2
# DEBUG: Map item input: "cherry banana"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🔧 SUB-PIPELINE START: 4 operations to apply
# DEBUG: Initial input: String("cherry banana")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: " ", range: Range(None, None, false) }
# DEBUG:   2. Unique
# DEBUG:   3. Sort { direction: Asc }
# DEBUG:   4. Join { sep: "-" }
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/4: Applying Split { sep: " ", range: Range(None, None, false) }
# DEBUG: Input: String("cherry banana")
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "cherry"
# DEBUG:   "banana"
# DEBUG: ])
# DEBUG: Step completed in 19.503µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/4: Applying Unique
# DEBUG: Input: List(2 items: ["cherry", "banana"])
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "cherry"
# DEBUG:   "banana"
# DEBUG: ])
# DEBUG: Step completed in 22.853µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 3/4: Applying Sort { direction: Asc }
# DEBUG: Input: List(2 items: ["cherry", "banana"])
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "banana"
# DEBUG:   "cherry"
# DEBUG: ])
# DEBUG: Step completed in 27.211µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 4/4: Applying Join { sep: "-" }
# DEBUG: Input: List(2 items: ["banana", "cherry"])
# DEBUG: 🎯 Result: String("banana-cherry")
# DEBUG: Step completed in 8.535µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ SUB-PIPELINE COMPLETE
# DEBUG: Total execution time: 243.62µs
# DEBUG: 🎯 Final result: String("banana-cherry")
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: Map item output: "banana-cherry"
# DEBUG: MAP COMPLETED: 2 → 2 items
# DEBUG: 🎯 Result: List(2 items: [
# DEBUG:   "apple-banana"
# DEBUG:   "banana-cherry"
# DEBUG: ])
# DEBUG: Step completed in 1.302789ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ✅ PIPELINE COMPLETE
# DEBUG: Total execution time: 3.224402ms
# DEBUG: 🎯 Final result: List(2 items: [
# DEBUG:   "apple-banana"
# DEBUG:   "banana-cherry"
# DEBUG: ])
# DEBUG: Cache stats: 0 regex patterns, 3 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: ✅ SINGLE TEMPLATE COMPLETE
# DEBUG: 🎯 Final result: "apple-banana,banana-cherry"
# DEBUG: Cache stats: 0 regex patterns, 3 split operations cached
# DEBUG: ═══════════════════════════════════════════════
# apple-banana,banana-cherry
```

**Key Insights:**

- **📋 List Processing**: Shows list operations within map
- **🔄 Type Changes**: String → List → String transformations
- **🧹 Data Cleaning**: See duplicate removal and sorting

## ⚡ Performance Analysis

### Timing Information

The debug system provides comprehensive timing data for performance analysis.

**Timing Metrics Available:**

| Metric | Scope | Precision | Use Case |
|--------|-------|-----------|----------|
| **Step Time** | Individual operations | Microseconds | Operation optimization |
| **Total Time** | Complete pipeline | Milliseconds | Overall performance |
| **Map Item Time** | Per-item in maps | Microseconds | Map optimization |
| **Cumulative Time** | Progressive timing | Milliseconds | Bottleneck identification |

**Example Output:**

```text
DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
DEBUG: Input: String("hello,world")
DEBUG: 🎯 Result: List(2 items: [
DEBUG:   "hello"
DEBUG:   "world"
DEBUG: ])
DEBUG: Step completed in 594.8µs
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 2/2: Applying Map { operations: [Trim, Upper] }
DEBUG: Step completed in 10.8661ms
DEBUG: ───────────────────────────────────────────────
DEBUG: ✅ PIPELINE COMPLETE
DEBUG: Total execution time: 21.018ms
DEBUG: Cache stats: 0 regex patterns, 1 split operations cached
```

### Memory Usage Tracking

Monitor memory consumption throughout pipeline execution for large datasets.

**Memory Metrics (Large Datasets Only):**

```text
DEBUG: Memory: ~48 chars across 3 items
DEBUG: Memory: ~156 chars in string
```

**Activation Thresholds:**

| Data Type | Threshold | Purpose |
|-----------|-----------|---------|
| **Lists** | > 1000 items | Track memory in large collections |
| **Strings** | > 10,000 characters | Monitor large string processing |

**Memory Information:**

| Metric Type | Description | Calculation |
|-------------|-------------|-------------|
| **List Memory** | Character count across items | Sum of all item lengths |
| **String Memory** | Direct character count | String length |
| **Approximate** | Estimation for performance | Character-based calculation |

> **Note:** Memory tracking only appears when processing large datasets to avoid cluttering debug output for typical use cases.

### Bottleneck Identification

Use timing data to identify performance bottlenecks.

**Performance Analysis Example:**

```text
# Slow operation identified
DEBUG: STEP 2/4: Applying Map { operations: [Complex_Regex_Operation] }
DEBUG: Step completed in 890.5ms    # ← Bottleneck!
DEBUG: STEP 3/4: Applying Sort
DEBUG: Step completed in 1.2ms
```

**Optimization Strategies:**

| Bottleneck Type | Typical Cause | Solution |
|----------------|---------------|----------|
| **🐌 Slow Map** | Complex per-item operations | Simplify map operations |
| **🔍 Slow Regex** | Complex patterns | Optimize regex patterns |
| **📊 Large Data** | Processing volume | Filter early in pipeline |
| **🔄 Redundant Ops** | Unnecessary operations | Combine operations |

### Optimization Insights

Debug output reveals optimization opportunities.

**Before Optimization:**

```bash
string-pipeline '{!split:,:..|map:{trim}|map:{upper}|map:{append:!}}' '  a  ,  b  ,  c  '
# Shows 3 separate map operations
```

**After Optimization:**

```bash
string-pipeline '{!split:,:..|map:{trim|upper|append:!}}' '  a  ,  b  ,  c  '
# Single map with combined operations - much faster
```

## 🚨 Error Debugging

### Parse Error Analysis

Debug template parsing errors with detailed context.

**Common Parse Errors:**

```bash
# ❌ Invalid operation
string-pipeline '{!split:,:..|invalid_op}' 'input'
# Error: Parse error:  --> 1:15
#   |
# 1 | {!split:,:..|invalid_op}
#   |               ^---
#   |
#   = expected operation

# ❌ Missing range
string-pipeline '{!split:,}' 'input'
# Error: Expected range specification after ':'

# ❌ Unclosed template
string-pipeline '{!split:,:.. ' 'input'
# Error: Expected '}'
```

### Runtime Error Diagnosis

Use debug mode to diagnose runtime errors.

**Type Mismatch Example:**

```bash
# ❌ Applying string operation to list
string-pipeline '{!split:,:..|upper}' 'a,b,c'
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: SINGLE TEMPLATE START
# DEBUG: Template: "{!split:,:..|upper}"
# DEBUG: Input: "a,b,c"
# DEBUG: ───────────────────────────────────────────────
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: 🚀 PIPELINE START: 2 operations to apply
# DEBUG: Initial input: String("a,b,c")
# DEBUG: Operations to apply:
# DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
# DEBUG:   2. Upper
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Input: String("a,b,c")
# DEBUG: 🎯 Result: List(3 items: [
# DEBUG:   "a"
# DEBUG:   "b"
# DEBUG:   "c"
# DEBUG: ])
# DEBUG: Step completed in 1.409827ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Upper
# DEBUG: Input: List(3 items: ["a", "b", "c"])
# Error formatting input: Upper operation can only be applied to strings. Use map to apply to lists.
```

**Debug-Guided Fix:**

```bash
# ✅ Correct approach
string-pipeline '{!split:,:..|map:{upper}}' 'a,b,c'
# DEBUG shows successful map operation
```

---

🎉 **Master the Debug System for Ultimate Pipeline Visibility!**

💡 **Pro Tip:** Combine debug insights with the [📖 Template System Documentation](template-system.md) and [⚙️ CLI Guide](command-line-options.md) for complete String Pipeline mastery!

🚀 **Start debugging your templates today and unlock deeper understanding of your data transformations!**
