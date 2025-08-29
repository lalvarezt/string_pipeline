# 🐛 Debug System Documentation

_NOTE: what follows has mostly been assembled using AI as an experiment and as a basis for further improvements._

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
  - [Visual Tree System](#visual-tree-system)
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
- **🌳 Tree-Based Hierarchy** - Clear visual structure using tree notation showing operation nesting and sub-pipelines
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
| **🏭 Production** | CLI `--quiet` | Suppress diagnostic output (from both inline `{!...}` and `--debug` flag |

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

The debug system uses a tree-based hierarchical structure to organize execution information, with distinct visual markers and consistent formatting patterns.

#### Level 1: Template Session Container

```text
DEBUG: 📂 Single Template
DEBUG: ├── 🏁 SINGLE TEMPLATE START
DEBUG: ├── Template: "{!split:,:..|map:{upper}}"
DEBUG: ├── ➡️ Input: "hello,world"
DEBUG: │
```

**Container Structure:**

- **Tree notation (`├──`, `└──`)**: Shows hierarchical relationships between components
- **Session icons**: Identifies execution context type (`📂`, `🏁`)
- **Template declaration**: Shows the raw template syntax being processed
- **Initial input**: Displays the starting data with explicit formatting

#### Level 2: Pipeline Execution Flow

```text
DEBUG: │   ├── 📂 Main Pipeline
DEBUG: │   ├── 🚀 PIPELINE START: 2 operations
DEBUG: │   ├── ➡️ Input: String(hello,world)
DEBUG: │   ├── 1. Split(',')
DEBUG: │   ├── 2. Map(1)
DEBUG: │   │   ├── ⚙️ Step 1: Split
DEBUG: │   │   ├── ➡️ Input: String(hello,world)
DEBUG: │   │   ├── 🎯 Result: String(HELLO)
DEBUG: │   │   └── Time: 35.612µs
DEBUG: │
```

**Pipeline Structure Elements:**

- **Nested tree structure**: Sub-components indented with appropriate tree prefixes
- **Step icons**: `⚙️` for operations, `🚀` for pipeline start, `✅` for completion
- **Operation descriptors**: Compact operation names with key parameters
- **Input/Output tracking**: Clear data flow with type annotations
- **Performance markers**: Step-level timing with `🎯 Result` indicators

#### Level 3: Sub-Pipeline Nesting

```text
DEBUG: │   │   │   ├── 📂 Sub-Pipeline
DEBUG: │   │   │   ├── 🔧 SUB-PIPELINE START: 1 operations
DEBUG: │   │   │   ├── ➡️ Input: String(hello)
DEBUG: │   │   │   │   ├── ⚙️ Step 1: Upper
DEBUG: │   │   │   │   ├── ➡️ Input: String(hello)
DEBUG: │   │   │   │   ├── 🎯 Result: String(HELLO)
DEBUG: │   │   │   │   └── Time: 35.612µs
DEBUG: │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
DEBUG: │   │   │   ├── 🎯 Result: String(HELLO)
DEBUG: │   │   │   └── Time: 103.315µs
DEBUG: │
```

**Nested Structure Characteristics:**

- **Deep tree nesting**: Sub-pipelines maintain clear hierarchical depth
- **Context preservation**: Different icons distinguish main from sub-pipelines (`🚀` vs `🔧`)
- **Independent operation tracking**: Sub-pipelines have their own step sequences
- **Isolated scope indicators**: Clear start/complete markers for nested execution

### Visual Tree System

#### Tree Notation Guide

| Symbol | Position | Usage | Purpose |
|--------|----------|-------|---------|
| `├──` | Branch | Has siblings below | Shows continuing structure |
| `└──` | Terminal | Last item in group | Indicates end of section |
| `│` | Vertical | Continuation line | Maintains visual hierarchy |
| `│   ├──` | Nested | Indented branch | Shows deeper nesting level |

#### Icon Semantics

| Icon | Meaning | Context | Information Type |
|------|---------|---------|------------------|
| `📂` | Container | Sessions, pipelines | Major execution contexts |
| `🏁` | Session | Template processing | Session boundaries |
| `🚀` | Main Pipeline | Top-level execution | Primary processing flow |
| `🔧` | Sub-Pipeline | Nested execution | Map item processing |
| `⚙️` | Operation | Individual steps | Step-by-step execution |
| `🗂️` | Map Item | Item processing | Individual item tracking |
| `➡️` | Input | Data flow | Input values |
| `🎯` | Result | Output data | Operation results |
| `✅` | Completion | Section end | Successful execution |
| `📦` | Summary | Operation stats | Completion statistics |

#### Data Type Representation

```text
DEBUG: ├── ➡️ Input: String(hello)           # Simple scalar with content
DEBUG: ├── 🎯 Result: List["hello", "world"] # Collection with preview
DEBUG: ├── ➡️ Input: List[a, b, ...+3]       # Large collection with truncation
```

**Type Display Pattern:**

- **Type identification**: Clear type names (`String`, `List`)
- **Content preview**: Truncated content for readability (40 char limit for strings)
- **Collection handling**: Smart truncation for lists (show first few items)
- **Size indicators**: Item counts and overflow notation for large collections

### Performance Integration

#### Timing Hierarchy

```text
DEBUG: │   │   │   └── Time: 35.612µs        # Individual operation timing
DEBUG: │   │   └── Time: 103.315µs           # Sub-pipeline total time
DEBUG: │   └── Time: 1.763036ms              # Main pipeline total time
DEBUG: └── Cache stats: 0 regex patterns, 1 split operations cached
```

**Performance Structure:**

- **Granular timing**: Every operation receives individual timing measurement
- **Cumulative tracking**: Pipeline and sub-pipeline level totals
- **Resource metrics**: Cache usage statistics for optimization insights
- **Unit consistency**: Automatic unit scaling (µs, ms, s) based on magnitude

This tree-based hierarchical structure enables developers to follow execution flow naturally while maintaining clear visual separation and consistent information density at each level.

## 🗺️ Complex Pipeline Debugging

### Simple Map Operations

Debug basic map operations with string transformations.

```bash
string-pipeline "{!split:,:..|map:{upper}}" "hello,world"
# DEBUG: 📂 MULTI-TEMPLATE
# DEBUG: ├── 🏁 MULTI-TEMPLATE START
# DEBUG: ├── Template: "{!split:,:..|map:{upper}}"
# DEBUG: ├── ➡️ Input: "hello,world"
# DEBUG: ├── 1 sections to process (literal: 0, template: 1)
# DEBUG: │
# DEBUG: ├── 📊 SECTION 1/1: [template: split(',',..) | map { operations: [upper] }]
# DEBUG: ├── 💾 CACHE MISS Computing and storing result
# DEBUG: │
# DEBUG: ├── 📂 Main Pipeline
# DEBUG: │   ├── 🚀 PIPELINE START: 2 operations
# DEBUG: │   ├── ➡️ Input: String(hello,world)
# DEBUG: │   ├── 1. Split(',')
# DEBUG: │   ├── 2. Map(1)
# DEBUG: │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   ├── ➡️ Input: String(hello,world)
# DEBUG: │   │   ├── 🎯 Result: List["hello", "world"]
# DEBUG: │   │   └── Time: 332.41µs
# DEBUG: │   ├── ⚙️ Step 2: Map
# DEBUG: │   │   ├── ➡️ Input: List["hello", "world"]
# DEBUG: │   │   ├── 🎯 Result: String(processing...)
# DEBUG: │   │   └── Time: 0ns
# DEBUG: │   │   ├── 🗂️ Item 1/2
# DEBUG: │   │   │   ├── ➡️ Input: "hello"
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 1 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String(hello)
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Upper
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(hello)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(HELLO)
# DEBUG: │   │   │   │   │   └── Time: 875ns
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(HELLO)
# DEBUG: │   │   │   │   └── Time: 16.37µs
# DEBUG: │   │   │   └── Output: "HELLO"
# DEBUG: │   │   ├── 🗂️ Item 2/2
# DEBUG: │   │   │   ├── ➡️ Input: "world"
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 1 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String(world)
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Upper
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(world)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(WORLD)
# DEBUG: │   │   │   │   │   └── Time: 93ns
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(WORLD)
# DEBUG: │   │   │   │   └── Time: 15.749µs
# DEBUG: │   │   │   └── Output: "WORLD"
# DEBUG: │   │   └── 📦 MAP COMPLETED: 2 → 2 items
# DEBUG: │   ├── ✅ PIPELINE COMPLETE
# DEBUG: │   ├── 🎯 Result: List["HELLO", "WORLD"]
# DEBUG: │   └── Time: 457.193µs
# DEBUG: │
# DEBUG: ├── 🏁 ✅ MULTI-TEMPLATE COMPLETE
# DEBUG: ├── 🎯 Final result: "HELLO,WORLD"
# DEBUG: ├── Total execution time: 568.533µs
# DEBUG: └── Cache stats: 0 regex patterns, 1 split operations cached
HELLO,WORLD
```

**Key Insights:**

- **📊 Item Processing**: Each item processed as a complete sub-pipeline with its own tree structure
- **🔄 Transformation**: Clear input → output mapping with detailed steps and timing
- **⚡ Performance**: Individual timing per sub-pipeline execution shows processing overhead

### Multi-Step Map Pipelines

Debug complex multi-operation map pipelines.

```bash
string-pipeline "{!split:,:..|map:{trim|upper}}" "  apple  , banana "
# DEBUG: 📂 MULTI-TEMPLATE
# DEBUG: ├── 🏁 MULTI-TEMPLATE START
# DEBUG: ├── Template: "{!split:,:..|map:{trim|upper}}"
# DEBUG: ├── ➡️ Input: "  apple  , banana "
# DEBUG: ├── 1 sections to process (literal: 0, template: 1)
# DEBUG: │
# DEBUG: ├── 📊 SECTION 1/1: [template: split(',',..) | map { operations: [trim { chars: "", direction: both }, upper] }]
# DEBUG: ├── 💾 CACHE MISS Computing and storing result
# DEBUG: │
# DEBUG: ├── 📂 Main Pipeline
# DEBUG: │   ├── 🚀 PIPELINE START: 2 operations
# DEBUG: │   ├── ➡️ Input: String(  apple  , banana )
# DEBUG: │   ├── 1. Split(',')
# DEBUG: │   ├── 2. Map(2)
# DEBUG: │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   ├── ➡️ Input: String(  apple  , banana )
# DEBUG: │   │   ├── 🎯 Result: List["  apple  ", " banana "]
# DEBUG: │   │   └── Time: 48.938µs
# DEBUG: │   ├── ⚙️ Step 2: Map
# DEBUG: │   │   ├── ➡️ Input: List["  apple  ", " banana "]
# DEBUG: │   │   ├── 🎯 Result: String(processing...)
# DEBUG: │   │   └── Time: 0ns
# DEBUG: │   │   ├── 🗂️ Item 1/2
# DEBUG: │   │   │   ├── ➡️ Input: "  apple  "
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 2 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String(  apple  )
# DEBUG: │   │   │   │   ├── 1. Trim
# DEBUG: │   │   │   │   ├── 2. Upper
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Trim
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(  apple  )
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(apple)
# DEBUG: │   │   │   │   │   └── Time: 3.953µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 2: Upper
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(apple)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(APPLE)
# DEBUG: │   │   │   │   │   └── Time: 909ns
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(APPLE)
# DEBUG: │   │   │   │   └── Time: 114.376µs
# DEBUG: │   │   │   └── Output: "APPLE"
# DEBUG: │   │   ├── 🗂️ Item 2/2
# DEBUG: │   │   │   ├── ➡️ Input: " banana "
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 2 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String( banana )
# DEBUG: │   │   │   │   ├── 1. Trim
# DEBUG: │   │   │   │   ├── 2. Upper
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Trim
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String( banana )
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(banana)
# DEBUG: │   │   │   │   │   └── Time: 13.048µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 2: Upper
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(banana)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(BANANA)
# DEBUG: │   │   │   │   │   └── Time: 174ns
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(BANANA)
# DEBUG: │   │   │   │   └── Time: 40.815µs
# DEBUG: │   │   │   └── Output: "BANANA"
# DEBUG: │   │   └── 📦 MAP COMPLETED: 2 → 2 items
# DEBUG: │   ├── ✅ PIPELINE COMPLETE
# DEBUG: │   ├── 🎯 Result: List["APPLE", "BANANA"]
# DEBUG: │   └── Time: 400.879µs
# DEBUG: │
# DEBUG: ├── 🏁 ✅ MULTI-TEMPLATE COMPLETE
# DEBUG: ├── 🎯 Final result: "APPLE,BANANA"
# DEBUG: ├── Total execution time: 546.721µs
# DEBUG: └── Cache stats: 0 regex patterns, 1 split operations cached
APPLE,BANANA
```

**Key Insights:**

- **🔗 Pipeline Flow**: Multi-step transformation per item shown as complete sub-pipeline with nested tree structure
- **📊 Data Evolution**: See how data changes at each step with timing information and clear visual hierarchy
- **🎯 Operation Chain**: Clear operation sequence with detailed execution trace using tree notation

### List Operations in Maps

Debug map operations that use list transformations.

```bash
string-pipeline "{!split:,:..|map:{split: :..|unique|sort|join:-}}" "apple banana apple,cherry banana"
# DEBUG: 📂 MULTI-TEMPLATE
# DEBUG: ├── 🏁 MULTI-TEMPLATE START
# DEBUG: ├── Template: "{!split:,:..|map:{split: :..|unique|sort|join:-}}"
# DEBUG: ├── ➡️ Input: "apple banana apple,cherry banana"
# DEBUG: ├── 1 sections to process (literal: 0, template: 1)
# DEBUG: │
# DEBUG: ├── 📊 SECTION 1/1: [template: split(',',..) | map { operations: [split { sep: " ", range: range(none, none, false) }, unique, sort { direction: asc }, join { sep: "-" }] }]
# DEBUG: ├── 💾 CACHE MISS Computing and storing result
# DEBUG: │
# DEBUG: ├── 📂 Main Pipeline
# DEBUG: │   ├── 🚀 PIPELINE START: 2 operations
# DEBUG: │   ├── ➡️ Input: String(apple banana apple,cherry banana)
# DEBUG: │   ├── 1. Split(',')
# DEBUG: │   ├── 2. Map(4)
# DEBUG: │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   ├── ➡️ Input: String(apple banana apple,cherry banana)
# DEBUG: │   │   ├── 🎯 Result: List["apple banana apple", "cherry banana"]
# DEBUG: │   │   └── Time: 51.152µs
# DEBUG: │   ├── ⚙️ Step 2: Map
# DEBUG: │   │   ├── ➡️ Input: List["apple banana apple", "cherry banana"]
# DEBUG: │   │   ├── 🎯 Result: String(processing...)
# DEBUG: │   │   └── Time: 0ns
# DEBUG: │   │   ├── 🗂️ Item 1/2
# DEBUG: │   │   │   ├── ➡️ Input: "apple banana apple"
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 4 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String(apple banana apple)
# DEBUG: │   │   │   │   ├── 1. Split(' ')
# DEBUG: │   │   │   │   ├── 2. Unique
# DEBUG: │   │   │   │   ├── 3. Sort
# DEBUG: │   │   │   │   ├── 4. Join('-')
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(apple banana apple)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["apple", "banana", "apple"]
# DEBUG: │   │   │   │   │   └── Time: 4.494µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 2: Unique
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["apple", "banana", "apple"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["apple", "banana"]
# DEBUG: │   │   │   │   │   └── Time: 9.507µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 3: Sort
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["apple", "banana"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["apple", "banana"]
# DEBUG: │   │   │   │   │   └── Time: 605.684µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 4: Join
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["apple", "banana"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(apple-banana)
# DEBUG: │   │   │   │   │   └── Time: 6.818µs
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(apple-banana)
# DEBUG: │   │   │   │   └── Time: 789.876µs
# DEBUG: │   │   │   └── Output: "apple-banana"
# DEBUG: │   │   ├── 🗂️ Item 2/2
# DEBUG: │   │   │   ├── ➡️ Input: "cherry banana"
# DEBUG: │   │   │   ├── 📂 Sub-Pipeline
# DEBUG: │   │   │   │   ├── 🔧 SUB-PIPELINE START: 4 operations
# DEBUG: │   │   │   │   ├── ➡️ Input: String(cherry banana)
# DEBUG: │   │   │   │   ├── 1. Split(' ')
# DEBUG: │   │   │   │   ├── 2. Unique
# DEBUG: │   │   │   │   ├── 3. Sort
# DEBUG: │   │   │   │   ├── 4. Join('-')
# DEBUG: │   │   │   │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   │   │   │   ├── ➡️ Input: String(cherry banana)
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["cherry", "banana"]
# DEBUG: │   │   │   │   │   └── Time: 6.573µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 2: Unique
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["cherry", "banana"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["cherry", "banana"]
# DEBUG: │   │   │   │   │   └── Time: 18.154µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 3: Sort
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["cherry", "banana"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: List["banana", "cherry"]
# DEBUG: │   │   │   │   │   └── Time: 1.091µs
# DEBUG: │   │   │   │   ├── ⚙️ Step 4: Join
# DEBUG: │   │   │   │   │   ├── ➡️ Input: List["banana", "cherry"]
# DEBUG: │   │   │   │   │   ├── 🎯 Result: String(banana-cherry)
# DEBUG: │   │   │   │   │   └── Time: 833ns
# DEBUG: │   │   │   │   ├── ✅ SUB-PIPELINE COMPLETE
# DEBUG: │   │   │   │   ├── 🎯 Result: String(banana-cherry)
# DEBUG: │   │   │   │   └── Time: 84.65µs
# DEBUG: │   │   │   └── Output: "banana-cherry"
# DEBUG: │   │   └── 📦 MAP COMPLETED: 2 → 2 items
# DEBUG: │   ├── ✅ PIPELINE COMPLETE
# DEBUG: │   ├── 🎯 Result: List["apple-banana", "banana-cherry"]
# DEBUG: │   └── Time: 1.18133ms
# DEBUG: │
# DEBUG: ├── 🏁 ✅ MULTI-TEMPLATE COMPLETE
# DEBUG: ├── 🎯 Final result: "apple-banana,banana-cherry"
# DEBUG: ├── Total execution time: 1.359647ms
# DEBUG: └── Cache stats: 0 regex patterns, 3 split operations cached
apple-banana,banana-cherry
```

**Key Insights:**

- **📋 List Processing**: Shows complex list operations within map using deep tree nesting
- **🔄 Type Changes**: String → List → String transformations clearly visible in tree structure
- **🧹 Data Cleaning**: See duplicate removal and sorting with step-by-step execution

## ⚡ Performance Analysis

### Timing Information

The debug system provides comprehensive timing data for performance analysis using the tree structure.

**Timing Metrics Available:**

| Metric | Scope | Precision | Use Case |
|--------|-------|-----------|----------|
| **Step Time** | Individual operations | Microseconds | Operation optimization |
| **Total Time** | Complete pipeline | Milliseconds | Overall performance |
| **Map Item Time** | Per-item in maps | Microseconds | Map optimization |
| **Cumulative Time** | Progressive timing | Milliseconds | Bottleneck identification |

**Example Output:**

```text
DEBUG: │   │   │   ├── ⚙️ Step 1: Split
DEBUG: │   │   │   ├── ➡️ Input: String(hello,world)
DEBUG: │   │   │   ├── 🎯 Result: List["hello", "world"]
DEBUG: │   │   │   └── Time: 594.8µs
DEBUG: │   │   │   ├── ⚙️ Step 2: Map
DEBUG: │   │   │   └── Time: 10.8661ms
DEBUG: │   │   ├── ✅ PIPELINE COMPLETE
DEBUG: │   │   └── Time: 21.018ms
DEBUG: └── Cache stats: 0 regex patterns, 1 split operations cached
```

### Memory Usage Tracking

Monitor memory consumption throughout pipeline execution for large datasets.

**Memory Metrics (Large Datasets Only):**

```text
DEBUG: ├── Memory: ~48 chars across 3 items
DEBUG: ├── Memory: ~156 chars in string
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
# Slow operation identified in tree structure
DEBUG: │   │   │   ├── ⚙️ Step 2: Map
DEBUG: │   │   │   └── Time: 890.5ms    # ← Bottleneck!
DEBUG: │   │   │   ├── ⚙️ Step 3: Sort
DEBUG: │   │   │   └── Time: 1.2ms
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
# Shows 3 separate map operations in tree structure
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
# DEBUG: 📂 MULTI-TEMPLATE
# DEBUG: ├── 🏁 MULTI-TEMPLATE START
# DEBUG: ├── Template: "{!split:,:..|upper}"
# DEBUG: ├── ➡️ Input: "a,b,c"
# DEBUG: ├── 1 sections to process (literal: 0, template: 1)
# DEBUG: │
# DEBUG: ├── 📊 SECTION 1/1: [template: split(',',..) | upper]
# DEBUG: ├── 💾 CACHE MISS Computing and storing result
# DEBUG: │
# DEBUG: ├── 📂 Main Pipeline
# DEBUG: │   ├── 🚀 PIPELINE START: 2 operations
# DEBUG: │   ├── ➡️ Input: String(a,b,c)
# DEBUG: │   ├── 1. Split(',')
# DEBUG: │   ├── 2. Upper
# DEBUG: │   ├── ⚙️ Step 1: Split
# DEBUG: │   │   ├── ➡️ Input: String(a,b,c)
# DEBUG: │   │   ├── 🎯 Result: List["a", "b", "c"]
# DEBUG: │   │   └── Time: 49.27µs
Error formatting input: Upper operation can only be applied to strings. Use map:{upper} for lists.
```

**Debug-Guided Fix:**

```bash
# ✅ Correct approach
string-pipeline '{!split:,:..|map:{upper}}' 'a,b,c'
# DEBUG shows successful map operation with tree structure
```

---

🎉 **Master the Debug System for Ultimate Pipeline Visibility!**

💡 **Pro Tip:** Combine debug insights with the [📖 Template System Documentation](template-system.md) and [⚙️ CLI Guide](command-line-options.md) for complete String Pipeline mastery!

🚀 **Start debugging your templates today and unlock deeper understanding of your data transformations!**
