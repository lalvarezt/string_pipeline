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
- [📚 Complete Documentation](#-complete-documentation)
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
# DEBUG: Initial value: Str("apple Banana cherry Date")
# DEBUG: Applying operation 1: Split { sep: " ", range: Range(None, None, false) }
# DEBUG: Result: List with 4 items:
# DEBUG:   [0]: "apple"
# DEBUG:   [1]: "Banana"
# DEBUG:   [2]: "cherry"
# DEBUG:   [3]: "Date"
# DEBUG: ---
# DEBUG: Applying operation 2: Filter { pattern: "^[A-Z]" }
# DEBUG: Result: List with 2 items:
# DEBUG:   [0]: "Banana"
# DEBUG:   [1]: "Date"
# DEBUG: ---
# DEBUG: Applying operation 3: Sort { direction: Desc }
# DEBUG: Result: List with 2 items:
# DEBUG:   [0]: "Date"
# DEBUG:   [1]: "Banana"
# DEBUG: ---
# Date Banana
```

> 💡 **Want to see more?** Check out the [📚 Complete Documentation](#-complete-documentation) with 20+ operations and real-world examples!

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
string_pipeline = "0.9.0"
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
# DEBUG: Initial value: Str("a,b,c")
# DEBUG: Applying operation 1: Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "a"
# DEBUG:   [1]: "b"
# DEBUG:   [2]: "c"
# DEBUG: ---
# DEBUG: Applying operation 2: Map { operations: [Upper] }
# DEBUG: Map operation starting with 3 items
# DEBUG: Map operations to apply: 1 steps
# DEBUG:   Step 1: Upper
# DEBUG: Processing item 1 of 3: "a"
# DEBUG:   Item 1/3 initial value: Str("a")
# DEBUG:   Item 1/3 applying step 1: Upper
# DEBUG:   Item 1/3 step 1 result: String("A")
# DEBUG: Processing item 2 of 3: "b"
# DEBUG:   Item 2/3 initial value: Str("b")
# DEBUG:   Item 2/3 applying step 1: Upper
# DEBUG:   Item 2/3 step 1 result: String("B")
# DEBUG: Processing item 3 of 3: "c"
# DEBUG:   Item 3/3 initial value: Str("c")
# DEBUG:   Item 3/3 applying step 1: Upper
# DEBUG:   Item 3/3 step 1 result: String("C")
# DEBUG: Map operation completed. Results:
# DEBUG:   Item 1: "A"
# DEBUG:   Item 2: "B"
# DEBUG:   Item 3: "C"
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "A"
# DEBUG:   [1]: "B"
# DEBUG:   [2]: "C"
# DEBUG: ---
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

## 📚 Complete Documentation

🎯 **[📖 Read the Full Template System Documentation](docs/template-system.md)**

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
