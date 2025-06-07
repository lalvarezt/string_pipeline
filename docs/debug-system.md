# 🐛 Debug System Documentation

A comprehensive debugging system for visualizing, analyzing, and troubleshooting string pipeline transformations with detailed execution insights and performance analysis.

## 📋 Table of Contents

- [🌟 Overview](#-overview)
- [🚀 Quick Start](#-quick-start)
- [🔧 Enabling Debug Mode](#-enabling-debug-mode)
  - [Inline Debug Syntax](#inline-debug-syntax)
  - [CLI Debug Flag](#cli-debug-flag)
  - [Debug Mode Comparison](#debug-mode-comparison)
- [🔍 Understanding Debug Output](#-understanding-debug-output)
  - [Pipeline Structure Display](#pipeline-structure-display)
  - [Step-by-Step Execution](#step-by-step-execution)
  - [Map Operation Visualization](#map-operation-visualization)
  - [Performance Metrics](#performance-metrics)
- [🗺️ Complex Pipeline Debugging](#️-complex-pipeline-debugging)
  - [Simple Map Operations](#simple-map-operations)
  - [Multi-Step Map Pipelines](#multi-step-map-pipelines)
  - [List Operations in Maps](#list-operations-in-maps)
  - [Nested Transformations](#nested-transformations)
- [⚡ Performance Analysis](#-performance-analysis)
  - [Timing Information](#timing-information)
  - [Memory Usage Tracking](#memory-usage-tracking)
  - [Bottleneck Identification](#bottleneck-identification)
  - [Optimization Insights](#optimization-insights)
- [🚨 Error Debugging](#-error-debugging)
  - [Parse Error Analysis](#parse-error-analysis)
  - [Runtime Error Diagnosis](#runtime-error-diagnosis)
  - [Type Mismatch Debugging](#type-mismatch-debugging)
  - [Common Error Patterns](#common-error-patterns)
- [📊 Debug Output Reference](#-debug-output-reference)
  - [Visual Elements](#visual-elements)
  - [Message Types](#message-types)
  - [Context Information](#context-information)
  - [Data Format Display](#data-format-display)
- [💡 Best Practices](#-best-practices)
  - [Development Workflow](#development-workflow)
  - [Performance Testing](#performance-testing)
  - [Production Debugging](#production-debugging)
  - [Template Optimization](#template-optimization)
- [🎯 Real-World Examples](#-real-world-examples)
  - [Data Processing Pipeline](#data-processing-pipeline)
  - [Complex Transformation Debugging](#complex-transformation-debugging)
  - [Performance Optimization Case Study](#performance-optimization-case-study)
  - [Error Resolution Examples](#error-resolution-examples)
- [⚠️ Troubleshooting](#️-troubleshooting)
  - [Debug Output Issues](#debug-output-issues)
  - [Performance Problems](#performance-problems)
  - [Common Debugging Mistakes](#common-debugging-mistakes)
- [🔧 Advanced Debugging Techniques](#-advanced-debugging-techniques)
  - [Selective Debugging](#selective-debugging)
  - [Quiet Debug Mode](#quiet-debug-mode)
  - [Debug Output Filtering](#debug-output-filtering)

## 🌟 Overview

The String Pipeline debug system provides comprehensive visibility into template execution, making it easy to understand data flow, identify performance bottlenecks, and troubleshoot complex transformations. Whether you're developing new templates or optimizing existing ones, the debug system offers detailed insights at every step.

### ✨ Key Features

- **🔍 Step-by-Step Execution Tracking** - See exactly how data flows through each operation
- **🗺️ Map Operation Visualization** - Detailed per-item processing in map operations
- **⚡ Performance Metrics** - Timing and memory usage for every operation
- **🎯 Hierarchical Display** - Clear visual structure showing operation nesting
- **🚨 Error Context** - Detailed error information with operation context
- **📊 Data Type Visualization** - See how values transform between types
- **🔧 Flexible Activation** - Enable via template syntax or CLI flags

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

```text
DEBUG: ═══════════════════════════════════════════════
DEBUG: PIPELINE START: 2 operations to apply
DEBUG: Initial input: Str("hello,world")
DEBUG: Operations to apply:
DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
DEBUG:   2. Map { operations: [Upper] }
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
DEBUG: Input: Str("hello,world")
DEBUG: Result: List(2 items: ["hello", "world"])
DEBUG: Step completed in 548.4µs
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 2/2: Applying Map { operations: [Upper] }
DEBUG: Input: List(["hello", "world"])
DEBUG: MAP OPERATION: Processing 2 items
DEBUG: ┌─ Processing item 1 of 2 ─────────────
DEBUG: │  Input: "hello"
DEBUG: │  Output: "HELLO"
DEBUG: └────────────────────────────────────────────
DEBUG: ┌─ Processing item 2 of 2 ─────────────
DEBUG: │  Input: "world"
DEBUG: │  Output: "WORLD"
DEBUG: └────────────────────────────────────────────
DEBUG: MAP COMPLETED: 2 → 2 items
DEBUG: Result: List(2 items: ["HELLO", "WORLD"])
DEBUG: Step completed in 20.0277ms
DEBUG: ───────────────────────────────────────────────
DEBUG: PIPELINE COMPLETE
DEBUG: Total execution time: 23.0989ms
DEBUG: Final result: List(["HELLO", "WORLD"])
DEBUG: ═══════════════════════════════════════════════
HELLO,WORLD
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

## 🔍 Understanding Debug Output

### Pipeline Structure Display

The debug system shows a clear overview of your pipeline structure before execution.

```text
DEBUG: ═══════════════════════════════════════════════
DEBUG: PIPELINE START: 3 operations to apply
DEBUG: Initial input: Str("apple,banana,cherry")
DEBUG: Operations to apply:
DEBUG:   1. Split { sep: ",", range: Range(None, None, false) }
DEBUG:   2. Map { operations: [Upper, Append { suffix: "!" }] }
DEBUG:   3. Join { sep: " | " }
DEBUG: ───────────────────────────────────────────────
```

**Structure Elements:**

| Element | Purpose | Information Provided |
|---------|---------|---------------------|
| **═ Header ═** | Section separator | Visual organization |
| **Operation Count** | Pipeline complexity | Total operations to execute |
| **Initial Input** | Starting data | Input type and value |
| **Operation List** | Execution plan | Detailed operation parameters |
| **── Separator ──** | Section boundary | Visual flow control |

### Step-by-Step Execution

Each operation is tracked individually with detailed input/output information.

```text
DEBUG: STEP 2/3: Applying Map { operations: [Upper, Append { suffix: "!" }] }
DEBUG: Input: List(["apple", "banana", "cherry"])
DEBUG: MAP OPERATION: Processing 3 items
DEBUG: Map pipeline: 2 operations
DEBUG:   Map step 1: Upper
DEBUG:   Map step 2: Append { suffix: "!" }
DEBUG: ───────────────────────────────────────────────
```

**Step Information:**

| Component | Description | Value |
|-----------|-------------|-------|
| **Step Number** | Current/Total position | `2/3` |
| **Operation Name** | What's being applied | `Map { operations: [...] }` |
| **Input Type** | Data type and content | `List(["apple", "banana", "cherry"])` |
| **Context Details** | Operation-specific info | Item count, sub-operations |

### Map Operation Visualization

Map operations receive special detailed visualization showing per-item processing.

```text
DEBUG: ┌─ Processing item 1 of 3 ─────────────
DEBUG: │  Input: "apple"
DEBUG: [Item 1/3] [map]     ═══════════════════════════════════════════════
DEBUG: [Item 1/3] [map]     PIPELINE START: 2 operations to apply
DEBUG: [Item 1/3] [map]     Initial input: String("apple")
DEBUG: [Item 1/3] [map]     ───────────────────────────────────────────────
DEBUG: [Item 1/3] [map]     STEP 1/2: Applying Upper
DEBUG: [Item 1/3] [map]     Input: String("apple")
DEBUG: [Item 1/3] [map]     Result: String("APPLE")
DEBUG: [Item 1/3] [map]     Step completed in 718.5µs
DEBUG: [Item 1/3] [map]     ───────────────────────────────────────────────
DEBUG: [Item 1/3] [map]     STEP 2/2: Applying Append { suffix: "!" }
DEBUG: [Item 1/3] [map]     Input: String("APPLE")
DEBUG: [Item 1/3] [map]     Result: String("APPLE!")
DEBUG: [Item 1/3] [map]     Step completed in 618.1µs
DEBUG: [Item 1/3] [map]     ───────────────────────────────────────────────
DEBUG: [Item 1/3] [map]     PIPELINE COMPLETE
DEBUG: [Item 1/3] [map]     Total execution time: 8.4076ms
DEBUG: [Item 1/3] [map]     Final result: String("APPLE!")
DEBUG: [Item 1/3] [map]     ═══════════════════════════════════════════════
DEBUG: │  Output: "APPLE!"
DEBUG: └────────────────────────────────────────────
```

**Map Visualization Features:**

| Element | Purpose | Information |
|---------|---------|-------------|
| **Box Drawing** | Visual item boundaries | `┌─` `│` `└─` characters |
| **Item Counter** | Progress tracking | `1 of 3`, `2 of 3`, etc. |
| **Nested Pipeline** | Sub-pipeline execution | Full pipeline debug per item |
| **Context Tags** | Source identification | `[Item X/Y] [map]` prefix |
| **Individual Timing** | Per-item performance | Execution time per item |

### Performance Metrics

Every operation includes detailed timing and memory information.

```text
DEBUG: Result: List(3 items: ["APPLE!", "BANANA!", "CHERRY!"])
DEBUG: Step completed in 42.1523ms
DEBUG: Memory: ~48 chars across 3 items
DEBUG: ───────────────────────────────────────────────
DEBUG: PIPELINE COMPLETE
DEBUG: Total execution time: 45.7832ms
DEBUG: Final result: List(["APPLE!", "BANANA!", "CHERRY!"])
```

**Performance Metrics:**

| Metric | Description | Use Case |
|--------|-------------|----------|
| **Step Timing** | Individual operation time | Identify slow operations |
| **Total Time** | Complete pipeline execution | Milliseconds | Overall performance |
| **Memory Usage** | Approximate data size | Memory optimization |
| **Item Counts** | Data volume tracking | Scaling analysis |

## 🗺️ Complex Pipeline Debugging

### Simple Map Operations

Debug basic map operations with string transformations.

**Template:** `{!split:,:..|map:{upper}}`
**Input:** `"hello,world,test"`

```text
DEBUG: MAP OPERATION: Processing 3 items
DEBUG: Map pipeline: 1 operations
DEBUG:   Map step 1: Upper
DEBUG: ───────────────────────────────────────────────
DEBUG: ┌─ Processing item 1 of 3 ─────────────
DEBUG: │  Input: "hello"
DEBUG: │  Output: "HELLO"
DEBUG: └────────────────────────────────────────────
DEBUG: ┌─ Processing item 2 of 3 ─────────────
DEBUG: │  Input: "world"
DEBUG: │  Output: "WORLD"
DEBUG: └────────────────────────────────────────────
DEBUG: ┌─ Processing item 3 of 3 ─────────────
DEBUG: │  Input: "test"
DEBUG: │  Output: "TEST"
DEBUG: └────────────────────────────────────────────
DEBUG: MAP COMPLETED: 3 → 3 items
```

**Key Insights:**

- **📊 Item Processing**: Each item processed individually
- **🔄 Transformation**: Clear input → output mapping
- **⚡ Performance**: Individual timing per item available

### Multi-Step Map Pipelines

Debug complex multi-operation map pipelines.

**Template:** `{!split:,:..|map:{split: :0|upper|append:_USER}}`
**Input:** `"alice 123,bob 456"`

```text
DEBUG: MAP OPERATION: Processing 2 items
DEBUG: Map pipeline: 3 operations
DEBUG:   Map step 1: Split { sep: " ", range: Index(0) }
DEBUG:   Map step 2: Upper
DEBUG:   Map step 3: Append { suffix: "_USER" }
DEBUG: ───────────────────────────────────────────────
DEBUG: ┌─ Processing item 1 of 2 ─────────────
DEBUG: │  Input: "alice 123"
DEBUG: [Item 1/2] [map]     STEP 1/3: Applying Split { sep: " ", range: Index(0) }
DEBUG: [Item 1/2] [map]     Input: String("alice 123")
DEBUG: [Item 1/2] [map]     Result: String("alice")
DEBUG: [Item 1/2] [map]     STEP 2/3: Applying Upper
DEBUG: [Item 1/2] [map]     Input: String("alice")
DEBUG: [Item 1/2] [map]     Result: String("ALICE")
DEBUG: [Item 1/2] [map]     STEP 3/3: Applying Append { suffix: "_USER" }
DEBUG: [Item 1/2] [map]     Input: String("ALICE")
DEBUG: [Item 1/2] [map]     Result: String("ALICE_USER")
DEBUG: │  Output: "ALICE_USER"
DEBUG: └────────────────────────────────────────────
```

**Key Insights:**

- **🔗 Pipeline Flow**: Multi-step transformation per item
- **📊 Data Evolution**: See how data changes at each step
- **🎯 Operation Chain**: Clear operation sequence

### List Operations in Maps

Debug map operations that use list transformations.

**Template:** `{!split:,:..|map:{split: :..|unique|sort|join:-}}`
**Input:** `"apple banana apple,cherry banana"`

```text
DEBUG: MAP OPERATION: Processing 2 items
DEBUG: Map pipeline: 4 operations
DEBUG:   Map step 1: Split { sep: " ", range: Range(None, None, false) }
DEBUG:   Map step 2: Unique
DEBUG:   Map step 3: Sort { direction: Asc }
DEBUG:   Map step 4: Join { sep: "-" }
DEBUG: ───────────────────────────────────────────────
DEBUG: ┌─ Processing item 1 of 2 ─────────────
DEBUG: │  Input: "apple banana apple"
DEBUG: [Item 1/2] [map]     STEP 1/4: Applying Split
DEBUG: [Item 1/2] [map]     Result: List(3 items: ["apple", "banana", "apple"])
DEBUG: [Item 1/2] [map]     STEP 2/4: Applying Unique
DEBUG: [Item 1/2] [map]     Result: List(2 items: ["apple", "banana"])
DEBUG: [Item 1/2] [map]     STEP 3/4: Applying Sort
DEBUG: [Item 1/2] [map]     Result: List(2 items: ["apple", "banana"])
DEBUG: [Item 1/2] [map]     STEP 4/4: Applying Join { sep: "-" }
DEBUG: [Item 1/2] [map]     Result: String("apple-banana")
DEBUG: │  Output: "apple-banana"
DEBUG: └────────────────────────────────────────────
```

**Key Insights:**

- **📋 List Processing**: Shows list operations within map
- **🔄 Type Changes**: String → List → String transformations
- **🧹 Data Cleaning**: See duplicate removal and sorting

### Nested Transformations

Debug complex nested operations with multiple levels.

```bash
# Complex log processing example
string-pipeline '{!split:\n:1..|map:{replace:s/ +/ /g|split: :0..3|filter:^[a-z]|sort|join:,}}' \
"USER         PID %CPU %MEM
root           1  0.0  0.1
alice        123  0.1  0.5
bob          456  0.0  0.2"
```

**Key Features:**

- **🏗️ Hierarchical Structure**: Multiple nesting levels clearly shown
- **🔍 Context Tracking**: Each level maintains operation context
- **📊 Performance Impact**: See how nesting affects timing

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
DEBUG: STEP 1/3: Applying Split { sep: ",", range: Range(None, None, false) }
DEBUG: Step completed in 548.4µs
DEBUG: STEP 2/3: Applying Map { operations: [Upper, Append] }
DEBUG: Step completed in 20.0277ms
DEBUG: STEP 3/3: Applying Join { sep: "-" }
DEBUG: Step completed in 156.2µs
DEBUG: Total execution time: 23.0989ms
```

### Memory Usage Tracking

Monitor memory consumption throughout pipeline execution.

**Memory Metrics:**

```text
DEBUG: Memory: ~48 chars across 3 items
DEBUG: Memory: ~156 chars in string
```

| Metric Type | Description | Calculation |
|-------------|-------------|-------------|
| **List Memory** | Character count across items | Sum of all item lengths |
| **String Memory** | Direct character count | String length |
| **Approximate** | Estimation for performance | Character-based calculation |

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
# DEBUG shows:
# DEBUG: STEP 2/2: Applying Upper
# DEBUG: Input: List(["a", "b", "c"])
# ERROR: upper operation can only be applied to strings. Use map:{upper} for lists.
```

**Debug-Guided Fix:**

```bash
# ✅ Correct approach
string-pipeline '{!split:,:..|map:{upper}}' 'a,b,c'
# DEBUG shows successful map operation
```

### Type Mismatch Debugging

Debug output clearly shows data types at each step.

**Example Debug Output:**

```text
DEBUG: STEP 1/2: Applying Split
DEBUG: Input: Str("a,b,c")           # ← String input
DEBUG: Result: List(["a", "b", "c"]) # ← List output
DEBUG: STEP 2/2: Applying Upper
DEBUG: Input: List(["a", "b", "c"])  # ← Type mismatch!
```

**Type Flow Analysis:**

| Step | Input Type | Operation | Output Type | Status |
|------|------------|-----------|-------------|---------|
| 1 | `Str` | `split` | `List` | ✅ Valid |
| 2 | `List` | `upper` | - | ❌ Error |

### Common Error Patterns

**Pattern 1: Missing Map Operation**

```bash
# ❌ Wrong
{split:,:..|upper}

# ✅ Correct
{split:,:..|map:{upper}}
```

**Pattern 2: Invalid Range**

```bash
# ❌ Wrong
{split:,:abc}

# ✅ Correct
{split:,:1..3}
```

**Pattern 3: Regex Errors**

```bash
# ❌ Wrong
{filter:[}

# ✅ Correct
{filter:\\[}
```

## 📊 Debug Output Reference

### Visual Elements

The debug system uses consistent visual elements for clarity.

| Element | Unicode | Purpose | Context |
|---------|---------|---------|---------|
| **Header** | `═══════` | Major sections | Pipeline start/end |
| **Separator** | `───────` | Step boundaries | Between operations |
| **Box Top** | `┌─────` | Item start | Map item processing |
| **Box Side** | `│` | Item content | Map item details |
| **Box Bottom** | `└─────` | Item end | Map item completion |

### Message Types

Different message types provide specific information.

| Type | Format | Information |
|------|--------|-------------|
| **🏗️ Structure** | `PIPELINE START: N operations` | Execution plan |
| **🔄 Step** | `STEP X/Y: Applying Operation` | Current operation |
| **📊 Result** | `Result: Type(value)` | Operation output |
| **⚡ Timing** | `Step completed in Xms` | Performance data |
| **🗺️ Map** | `MAP OPERATION: Processing N items` | Map operation start |
| **✅ Complete** | `PIPELINE COMPLETE` | Execution end |

### Context Information

Debug messages include rich context information.

**Context Elements:**

```text
DEBUG: [Item 2/5] [map]     STEP 1/3: Applying Upper
       │         │          │         │
       │         │          │         └─ Operation being applied
       │         │          └─ Step number in sub-pipeline
       │         └─ Context type (map operation)
       └─ Item number and total
```

### Data Format Display

Debug output shows data in human-readable formats.

**Data Type Representations:**

| Type | Format | Example |
|------|--------|---------|
| **String** | `Str("value")` | `Str("hello world")` |
| **List** | `List(N items: [...])` | `List(3 items: ["a", "b", "c"])` |
| **Large List** | `List(N items: showing first/last)` | `List(100 items: ["a", "b", "c"]...["x", "y", "z"])` |

## 💡 Best Practices

### Development Workflow

**1. 🧪 Iterative Development:**

```bash
# ✅ Start simple
string-pipeline '{!split:,:..}' 'a,b,c'

# ✅ Add operations incrementally
string-pipeline '{!split:,:..|map:{upper}}' 'a,b,c'

# ✅ Build complexity gradually
string-pipeline '{!split:,:..|map:{upper|append:!}|sort}' 'c,a,b'
```

**2. 🔍 Debug-First Approach:**

```bash
# ✅ Always debug new templates
string-pipeline '{!new_template}' 'test_data'

# ✅ Remove debug when satisfied
string-pipeline '{final_template}' 'production_data'
```

### Performance Testing

**1. 📊 Baseline Measurement:**

```bash
# Measure performance with debug
string-pipeline '{!complex_template}' 'large_dataset'
# Note timing information

# Test optimizations
string-pipeline '{!optimized_template}' 'large_dataset'
# Compare timing improvements
```

**2. 🎯 Targeted Optimization:**

```bash
# Identify slow operations from debug output
# Focus optimization efforts on bottlenecks
# Re-measure to verify improvements
```

### Production Debugging

**1. 🤫 Quiet Debug Mode:**

```bash
# ✅ Production-safe debugging
string-pipeline --debug --quiet '{template}' 'data' > output.txt 2> debug.log

# ✅ Analyze debug log separately
cat debug.log | grep "Step completed" | sort -k4 -n
```

**2. 🔒 Controlled Debug Activation:**

```bash
# ✅ Environment-controlled debugging
if [ "$DEBUG_MODE" = "true" ]; then
    DEBUG_FLAG="--debug"
else
    DEBUG_FLAG=""
fi

string-pipeline $DEBUG_FLAG '{template}' 'data'
```

### Template Optimization

**1. 🧹 Operation Combining:**

```bash
# ❌ Multiple maps (inefficient)
{split:,:..|map:{trim}|map:{upper}|map:{append:!}}

# ✅ Single map (efficient)
{split:,:..|map:{trim|upper|append:!}}
```

**2. 🎯 Early Filtering:**

```bash
# ❌ Filter after processing (wasteful)
{split:,:..|map:{expensive_operation}|filter:pattern}

# ✅ Filter before processing (efficient)
{split:,:..|filter:pattern|map:{expensive_operation}}
```

## 🎯 Real-World Examples

### Data Processing Pipeline

**Scenario:** Processing CSV data with multiple transformations.

```bash
# Input: "Name,Age,City\nJohn Doe,30,NYC\nJane Smith,25,LA"
string-pipeline '{!split:\n:1..|map:{split:,:0|trim|upper|append: (User)}|sort}' \
"Name,Age,City
John Doe,30,NYC
Jane Smith,25,LA"
```

**Debug Analysis:**

```text
DEBUG: PIPELINE START: 2 operations to apply
DEBUG: STEP 1/2: Applying Split { sep: "\n", range: Range(1, None, false) }
DEBUG: Input: Str("Name,Age,City\nJohn Doe,30,NYC\nJane Smith,25,LA")
DEBUG: Result: List(2 items: ["John Doe,30,NYC", "Jane Smith,25,LA"])
DEBUG: STEP 2/2: Applying Map
DEBUG: MAP OPERATION: Processing 2 items
DEBUG: ┌─ Processing item 1 of 2 ─────────────
DEBUG: │  Input: "John Doe,30,NYC"
DEBUG: [Item 1/2] [map] STEP 1/4: Applying Split { sep: ",", range: Index(0) }
DEBUG: [Item 1/2] [map] Result: String("John Doe")
DEBUG: [Item 1/2] [map] STEP 2/4: Applying Trim
DEBUG: [Item 1/2] [map] Result: String("John Doe")
DEBUG: [Item 1/2] [map] STEP 3/4: Applying Upper
DEBUG: [Item 1/2] [map] Result: String("JOHN DOE")
DEBUG: [Item 1/2] [map] STEP 4/4: Applying Append { suffix: " (User)" }
DEBUG: [Item 1/2] [map] Result: String("JOHN DOE (User)")
DEBUG: │  Output: "JOHN DOE (User)"
DEBUG: └────────────────────────────────────────────
```

**Insights:**

- **📊 Clear Data Flow**: See CSV parsing step by step
- **🔄 Transformation Chain**: Each map operation tracked
- **⚡ Performance**: Individual timing for optimization

### Complex Transformation Debugging

**Scenario:** Log processing with complex regex and filtering.

```bash
string-pipeline '{!split:\n:..|filter:ERROR|map:{regex_extract:\d{2}:\d{2}:\d{2}|prepend:Time: }|join: | }' \
"2023-01-01 10:30:15 INFO Process started
2023-01-01 10:30:45 ERROR Connection failed
2023-01-01 10:31:02 ERROR Timeout occurred
2023-01-01 10:31:15 INFO Process completed"
```

**Debug Output Shows:**

- **🧹 Filtering**: Only ERROR lines processed
- **🔍 Regex Extraction**: Time extraction from each line
- **🎨 Formatting**: Prefix addition and final joining

### Performance Optimization Case Study

**Before Optimization:**

```bash
string-pipeline '{!split:,:..|map:{trim}|map:{upper}|filter:^[A-Z]{3,}|map:{append:!}}' \
'  apple  ,  hi  ,  banana  ,  go  ,  cherry  '
```

**Debug shows multiple map operations:**

- Map 1: Trim operations
- Map 2: Upper operations
- Filter: Length filtering
- Map 3: Append operations

**After Optimization:**

```bash
string-pipeline '{!split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}|map:{append:!}}' \
'  apple  ,  hi  ,  banana  ,  go  ,  cherry  '
```

**Performance Improvement:**

- Reduced map operations from 3 to 2
- Combined trim and upper into single map
- 40% performance improvement shown in timing

### Error Resolution Examples

**Error 1: Type Mismatch**

```bash
# ❌ Problem
string-pipeline '{!split:,:..|sort|upper}' 'c,a,b'
# DEBUG shows sort returns List, upper expects String
```

**Resolution:**

```bash
# ✅ Solution
string-pipeline '{!split:,:..|sort|map:{upper}}' 'c,a,b'
# DEBUG shows correct List → Map → List flow
```

**Error 2: Invalid Range**

```bash
# ❌ Problem
string-pipeline '{!split:,:end}' 'a,b,c'
# Parse error: Invalid range specification
```

**Resolution:**

```bash
# ✅ Solution
string-pipeline '{!split:,:-1}' 'a,b,c'
# DEBUG shows correct range usage
```

## ⚠️ Troubleshooting

### Debug Output Issues

**Problem:** Debug output not appearing.

**Solutions:**

1. **✅ Check Debug Activation:**

   ```bash
   # Inline debug
   string-pipeline '{!operation}' 'input'

   # CLI debug
   string-pipeline --debug '{operation}' 'input'
   ```

2. **✅ Verify Template Syntax:**

   ```bash
   string-pipeline --validate '{!operation}'
   ```

3. **✅ Check Error Output:**

   ```bash
   string-pipeline '{!operation}' 'input' 2>&1 | grep DEBUG
   ```

### Performance Problems

**Problem:** Debug mode is too slow.

**Solutions:**

1. **🤫 Use Quiet Mode:**

   ```bash
   string-pipeline --debug --quiet '{template}' 'input'
   ```

2. **🎯 Test with Small Data:**

   ```bash
   string-pipeline '{!template}' 'small_test_data'
   ```

3. **📊 Profile Specific Operations:**

   ```bash
   # Test individual operations
   string-pipeline '{!operation1}' 'test'
   string-pipeline '{!operation2}' 'test'
   ```

### Common Debugging Mistakes

**1. ❌ Debugging Production Data**

```bash
# Wrong: Debug with large production data
string-pipeline '{!complex_template}' large_production_file.txt

# Right: Test with small sample first
head -10 large_production_file.txt | string-pipeline '{!complex_template}'
```

**2. ❌ Ignoring Performance Warnings**

```bash
# Wrong: Ignoring slow operations shown in debug
DEBUG: Step completed in 2.5s  # ← Red flag!

# Right: Investigate and optimize slow operations
```

**3. ❌ Not Using Validation**

```bash
# Wrong: Debug invalid templates
string-pipeline '{!broken_template}' 'input'

# Right: Validate first
string-pipeline --validate '{template}' && \
string-pipeline '{!template}' 'input'
```

## 🔧 Advanced Debugging Techniques

### Selective Debugging

Debug only specific parts of complex pipelines.

```bash
# Debug just the first part
string-pipeline '{!split:,:..|map:{upper}}' 'input' | \
string-pipeline '{filter:^[A-Z]{3,}|sort}'

# Debug just the second part
string-pipeline '{split:,:..|map:{upper}}' 'input' | \
string-pipeline '{!filter:^[A-Z]{3,}|sort}'
```

### Quiet Debug Mode

Get debug information without overwhelming output.

```bash
# Combine debug with quiet for clean logs
string-pipeline --debug --quiet '{template}' 'input' 2> debug.log
cat debug.log | grep -E "(Step completed|Total execution)"
```

### Debug Output Filtering

Extract specific debug information.

```bash
# Extract only timing information
string-pipeline '{!template}' 'input' 2>&1 | grep "completed in"

# Extract only map operations
string-pipeline '{!template}' 'input' 2>&1 | grep "MAP OPERATION"

# Extract memory usage
string-pipeline '{!template}' 'input' 2>&1 | grep "Memory:"
```

---

🎉 **Master the Debug System for Ultimate Pipeline Visibility!**

💡 **Pro Tip:** Combine debug insights with the [📖 Template System Documentation](template-system.md) and [⚙️ CLI Guide](command-line-options.md) for complete String Pipeline mastery!

🚀 **Start debugging your templates today and unlock deeper understanding of your data transformations!**
