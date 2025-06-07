# 🔗 String Pipeline

[![Crates.io](https://img.shields.io/crates/v/string_pipeline.svg)](https://crates.io/crates/string_pipeline)
[![Docs.rs](https://docs.rs/string_pipeline/badge.svg)](https://docs.rs/string_pipeline)
[![CI](https://github.com/lalvarezt/string_pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/lalvarezt/string_pipeline/actions)
[![License](https://img.shields.io/crates/l/string_pipeline.svg)](https://github.com/lalvarezt/string_pipeline/blob/main/LICENSE)

A powerful string transformation CLI tool and Rust library that makes complex text processing simple. Transform data using intuitive **template syntax** — chain operations like **split**, **join**, **replace**, **filter**, and **20+ others** in a single readable expression.

---

## 📋 Table of Contents

- [🌟 Why String Pipeline?](#-why-string-pipeline)
- [⚡ Quick Examples](#-quick-examples)
- [🚀 Installation](#-installation)
- [🏃 Quick Start](#-quick-start)
- [📚 Documentation](#-documentation)
- [🧪 Testing](#-testing)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

## 🌟 Why String Pipeline?

**Transform complex text processing into simple, readable templates:**

```bash
# Traditional approach (multiple commands)
echo "john.doe@email.com,jane.smith@company.org" | \
  tr ',' '\n' | \
  grep -o '@[^,]*' | \
  tr -d '@' | \
  sort | \
  tr '\n' ','

# String Pipeline (single template)
string-pipeline "{split:,:..|map:{regex_extract:@(.+):1}|sort}" "john.doe@email.com,jane.smith@company.org"
# Output: "company.org,email.com"
```

### ✨ Key Features

- **🔗 Chainable Operations**: Pipe operations together naturally
- **🎯 Precise Control**: Python-like ranges with Rust syntax (`-2..`, `1..=3`)
- **🗺️ Powerful Mapping**: Apply sub-pipelines to each list item
- **🔍 Regex Support**: sed-like patterns for complex transformations
- **🐛 Debug Mode**: Step-by-step operation visualization
- **⚡ Performance Tools**: Comprehensive benchmarking and optimization
- **📥 Flexible I/O**: CLI tool + embeddable Rust library

## ⚡ Quick Examples

### 🔥 Basic Transformations

```bash
# Extract middle items from list
string-pipeline "{split:,:1..3}" "a,b,c,d,e"
# Output: "b,c"

# Clean and format names
string-pipeline '{split:,:..|map:{trim|upper|append:!}}' "  john  , jane , bob  "
# Output: "JOHN!,JANE!,BOB!"

# Extract numbers and pad with zeros
string-pipeline '{split:,:..|map:{regex_extract:\d+|pad:3:0:left}}' "item1,thing22,stuff333"
# Output: "001,022,333"
```

### 🧠 Advanced Processing

```bash
# Filter files, format as list
string-pipeline '{split:,:..|filter:\.py$|sort|map:{prepend:• }|join:\n}' "app.py,readme.md,test.py,data.json"
# Output: "• app.py\n• test.py"

# Extract domains from URLs
string-pipeline '{split:,:..|map:{regex_extract://([^/]+):1|upper}}' "https://github.com,https://google.com"
# Output: "GITHUB.COM,GOOGLE.COM"

# Debug complex processing
string-pipeline "{!split: :..|filter:^[A-Z]|sort:desc}" "apple Banana cherry Date"
# Shows step-by-step processing + final output: "Date,Banana"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: PIPELINE START: 3 operations to apply
# DEBUG: Initial input: Str("apple Banana cherry Date")
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/3: Applying Split { sep: " ", range: Range(None, None, false) }
# DEBUG: Input: Str("apple Banana cherry Date")
# DEBUG: Result: List(4 items: ["apple", "Banana", "cherry", "Date"])
# DEBUG: Step completed in 425.1µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/3: Applying Filter { pattern: "^[A-Z]" }
# DEBUG: Input: List(["apple", "Banana", "cherry", "Date"])
# DEBUG: Result: List(2 items: ["Banana", "Date"])
# DEBUG: Step completed in 287.6µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 3/3: Applying Sort { direction: Desc }
# DEBUG: Input: List(["Banana", "Date"])
# DEBUG: Result: List(2 items: ["Date", "Banana"])
# DEBUG: Step completed in 156.3µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: PIPELINE COMPLETE
# DEBUG: Total execution time: 1.2431ms
# DEBUG: ═══════════════════════════════════════════════
# Date Banana
```

> 💡 **Want to see more?** Check out the [📚 Documentation](#-documentation) with 20+ operations and real-world examples!

## 🚀 Installation

### 📦 CLI Tool

```bash
# Install from crates.io
cargo install string_pipeline

# Or build from source
git clone https://github.com/lalvarezt/string_pipeline.git
cd string_pipeline
cargo install --path .
```

### 📚 Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
string_pipeline = "0.10.0"
```

## 🏃 Quick Start

### 💻 CLI Usage

```bash
# With argument
string-pipeline '{template}' "input string"

# With stdin
echo "input" | string-pipeline '{template}'

# Debug mode (shows each step)
string-pipeline '{!split:,:..|map:{upper}}' "a,b,c"
# DEBUG: ═══════════════════════════════════════════════
# DEBUG: PIPELINE START: 2 operations to apply
# DEBUG: Initial input: Str("a,b,c")
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 1/2: Applying Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Input: Str("a,b,c")
# DEBUG: Result: List(3 items: ["a", "b", "c"])
# DEBUG: Step completed in 342.7µs
# DEBUG: ───────────────────────────────────────────────
# DEBUG: STEP 2/2: Applying Map { operations: [Upper] }
# DEBUG: Input: List(["a", "b", "c"])
# DEBUG: MAP OPERATION: Processing 3 items
# DEBUG: ┌─ Processing item 1 of 3 ─────────────
# DEBUG: │  Input: "a" → Output: "A"
# DEBUG: └────────────────────────────────────────────
# DEBUG: ┌─ Processing item 2 of 3 ─────────────
# DEBUG: │  Input: "b" → Output: "B"
# DEBUG: └────────────────────────────────────────────
# DEBUG: ┌─ Processing item 3 of 3 ─────────────
# DEBUG: │  Input: "c" → Output: "C"
# DEBUG: └────────────────────────────────────────────
# DEBUG: MAP COMPLETED: 3 → 3 items
# DEBUG: Result: List(3 items: ["A", "B", "C"])
# DEBUG: Step completed in 15.2841ms
# DEBUG: ───────────────────────────────────────────────
# DEBUG: PIPELINE COMPLETE
# DEBUG: Total execution time: 18.7456ms
# DEBUG: ═══════════════════════════════════════════════
# A,B,C
```

### 🦀 Library Usage

```rust
use string_pipeline::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::parse("{split:,:..|map:{upper}|join:-}")?;
    let result = template.format("hello,world,rust")?;
    println!("{}", result); // "HELLO-WORLD-RUST"
    Ok(())
}
```

## 📚 Documentation

🎯 **[📖 Template System](docs/template-system.md)**

🔗 **[⚙️  CLI Options & Usage](docs/command-line-options.md)**

🐛 **[🔍 Comprehensive Debug System Guide](docs/debug-system.md)**

⚡ **[📊 Performance Benchmarking Guide](docs/benchmarking.md)**

**Everything you need to master String Pipeline:**

- **🏗️ Template Syntax** - Structure, chaining, escaping rules
- **📊 Operations Reference** - 20+ operations with examples
  - 🔪 **Split & Join** - Parse and reassemble text
  - ✂️ **Slice & Range** - Extract with Python-like indices
  - 🎨 **Transform** - Case, trim, pad, append/prepend
  - 🔍 **Regex** - Pattern matching and replacement
  - 🗂️ **List Ops** - Filter, sort, unique, reverse
  - 🗺️ **Map** - Apply operations to each item
- **🎯 Range Specifications** - Negative indexing, edge cases
- **🛡️ Escaping Rules** - When and how to escape characters
- **🐛 Debug Mode** - Visual operation debugging
- **💡 Real-world Examples** - Data processing, log analysis, formatting
- **⚠️ Troubleshooting** - Common errors and best practices

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## ⚡ Performance & Benchmarking

String Pipeline includes simple benchmarking tools for measuring performance:

```bash
# Build the benchmark tool
cargo build --release --bin bench

# Run benchmarks (1000 iterations)
./target/release/bench

# Quick performance check (100 iterations)
./target/release/bench --iterations 100

# Generate JSON for scripts
./target/release/bench --format json > benchmark_results.json
```

**Performance Examples:**

- **Fast basic operations**: 100-150ns (upper, lower, trim)
- **List processing**: 3-6μs (split, join, sort)
- **Complex transformations**: 10-60μs (map operations, regex)
- **Release builds**: 3-10x faster than debug builds

See the [📊 Performance Benchmarking Guide](docs/benchmarking.md) for timing details and measurement tips.

## 🤝 Contributing

We welcome contributions! 🎉

- 🐛 **Report bugs** via [GitHub Issues](https://github.com/lalvarezt/string_pipeline/issues)
- 💡 **Suggest features** or improvements
- 🔧 **Submit pull requests**

📖 Please see our [comprehensive documentation](docs/template-system.md) for syntax details and examples.

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**⚡ Fast, composable string transformations made simple!**
