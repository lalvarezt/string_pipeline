# 🔗 String Pipeline

[![Crates.io](https://img.shields.io/crates/v/string_pipeline.svg)](https://crates.io/crates/string_pipeline)
[![Docs.rs](https://docs.rs/string_pipeline/badge.svg)](https://docs.rs/string_pipeline)
[![CI](https://github.com/lalvarezt/string_pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/lalvarezt/string_pipeline/actions)
[![License](https://img.shields.io/crates/l/string_pipeline.svg)](https://github.com/lalvarezt/string_pipeline/blob/main/LICENSE)

A string transformation library and CLI tool for Rust. Chain operations like split, join, replace, and filter using template syntax.

---

## 📋 Table of Contents

- [🌟 Why String Pipeline?](#-why-string-pipeline)
- [⚡ Examples](#-examples)
- [🚀 Installation](#-installation)
- [🏃 Quick Start](#-quick-start)
- [🧪 Testing](#-testing)
- [⚡ Performance & Benchmarking](#-performance--benchmarking)
- [📚 Documentation](#-documentation)
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
- **🗺️ Powerful Mapping**: Apply sub-pipelines to each list item
- **🔍 Regex Support**: sed-like patterns for complex transformations
- **🐛 Debug Mode**: Step-by-step operation visualization

## ⚡ Examples

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
string-pipeline "{split: :..|filter:^[A-Z]|sort:desc}" "apple Banana cherry Date"
# Output: Date,Banana
```

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
string_pipeline = "0.12.0"
```

## 🏃 Quick Start

### 💻 CLI Usage

```bash
# With argument
string-pipeline '{template}' "input string"

# With stdin
echo "input" | string-pipeline '{template}'

# Debug mode
string-pipeline --debug "{split:,:..|map:{upper}}" "hello,world"
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
# HELLO,WORLD
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

## 📚 Documentation

[📖 Template System](docs/template-system.md)
[⚙️  CLI Options & Usage](docs/command-line-options.md)
[🔍 Comprehensive Debug System Guide](docs/debug-system.md)
[📊 Performance Benchmarking Guide](docs/benchmarking.md)

## 🤝 Contributing

We welcome contributions! 🎉

- 🐛 **Report bugs** via [GitHub Issues](https://github.com/lalvarezt/string_pipeline/issues)
- 💡 **Suggest features** or improvements
- 🔧 **Submit pull requests**

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**⚡ Fast and composable string transformations made simple!**
