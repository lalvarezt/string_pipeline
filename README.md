# String Pipeline

[![Crates.io](https://img.shields.io/crates/v/string_pipeline.svg)](https://crates.io/crates/string_pipeline)
[![Docs.rs](https://docs.rs/string_pipeline/badge.svg)](https://docs.rs/string_pipeline)
[![CI](https://github.com/lalvarezt/string_pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/lalvarezt/string_pipeline/actions)
[![License](https://img.shields.io/crates/l/string_pipeline.svg)](https://github.com/lalvarezt/string_pipeline/blob/main/LICENSE)

---

A flexible, composable string transformation CLI tool and Rust library. `string_pipeline` lets you chain operations like split, join, slice, replace, case conversion, trim, and more, using a concise template syntax. It is ideal for quick text manipulation, scripting, and data extraction.

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Examples](#examples)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **🔗 Composable operations**: Chain multiple string operations in a single template
- **✂️ Split and join**: Extract and reassemble parts of strings with flexible separators
- **🎯 Range operations**: Python-like negative indices with Rust-like syntax (`1..3`, `..=5`, `-2..`)
- **🔍 Regex support**: sed-like regex replace and pattern extraction
- **🔧 List operations**: Filter, sort, reverse, unique, slice with regex patterns
- **🗺️ Map operations**: Apply sub-pipelines to each list item individually
- **🎨 Text formatting**: Case conversion, trim, pad, append/prepend
- **🌈 ANSI support**: Strip ANSI escape sequences for clean output
- **🐛 Debug mode**: Visual step-by-step operation debugging with `!`
- **📥 Flexible I/O**: CLI with stdin support and library for embedding
- **⚡ Smart parsing**: Context-aware escaping - pipes work naturally in most cases

## Installation

### CLI Tool

```sh
# From source
git clone https://github.com/lalvarezt/string_pipeline.git
cd string_pipeline
cargo install --path .

# Or run directly
cargo run -- "{template}" "input string"
```

### Rust Library

Find the crate on [crates.io](https://crates.io/crates/string_pipeline):

```toml
[dependencies]
string_pipeline = "0.8.1"
```

## Quick Start

### CLI Usage

```sh
# With input argument
string-pipeline "{template}" "input string"

# With stdin input
echo "input string" | string-pipeline "{template}"

# Debug mode - see each step
string-pipeline "{!split:,:..|map:{upper}}" "a,b,c"
```

### Library Usage

```rust
use string_pipeline::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::parse("{split:,:..|map:{upper}|join:-}")?;
    let result = template.format("hello,world,test")?;
    assert_eq!(result, "HELLO-WORLD-TEST");
    Ok(())
}
```

## Documentation

📚 **[Complete Documentation](docs/template-system.md)** - Comprehensive guide covering:

- 🏗️ Template syntax and structure
- 📊 All operations with examples
- 🎯 Range specifications and negative indexing
- 🔤 Escaping rules and special characters
- 🗺️ Map operations for per-item processing
- 🐛 Debug mode for troubleshooting
- 💡 Real-world examples and best practices
- ⚠️ Common pitfalls and solutions

## Examples

### Basic Operations

```sh
# Extract second item from comma-separated list
string-pipeline "{split:,:1}" "a,b,c,d"
# Output: b

# Shorthand for splitting on spaces
string-pipeline "{1}" "foo bar baz"
# Output: bar

# Range selection with inclusive end
string-pipeline "{split:,:1..=3}" "a,b,c,d,e"
# Output: b,c,d
```

### Text Transformation

```sh
# Replace and convert case
string-pipeline "{replace:s/ /_/g|upper}" "hello world"
# Output: HELLO_WORLD

# Process each item in a list
string-pipeline "{split:,:..|map:{trim|upper|append:!}}" " a, b , c "
# Output: A!,B!,C!

# Extract and format data
string-pipeline "{split:,:..|map:{regex_extract:\\d+|pad:3:0:left}}" "a1,b22,c333"
# Output: 001,022,333
```

### Advanced Processing

```sh
# Filter, sort, and format
string-pipeline "{split:,:..|filter:\\.txt$|sort|map:{upper}}" "file.txt,doc.pdf,readme.txt,image.png"
# Output: FILE.TXT,README.TXT

# Complex data cleaning
string-pipeline "{split:,:..|map:{trim: *|lower}|unique|sort}" "  *APPLE*, *banana*, *APPLE*  "
# Output: apple,banana

# Debug mode to see each step
string-pipeline "{!split: :..|map:{upper}|join:_}" "hello world test"
# DEBUG: Initial value: Str("hello world test")
# DEBUG: Applying operation 1: Split { sep: " ", range: Range(None, None, false) }
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "hello"
# DEBUG:   [1]: "world"
# DEBUG:   [2]: "test"
# DEBUG: ---
# DEBUG: Applying operation 2: Map { operations: [Upper] }
# DEBUG: Map operation starting with 3 items
# DEBUG: Map operations to apply: 1 steps
# DEBUG:   Step 1: Upper
# DEBUG: Processing item 1 of 3: "hello"
# DEBUG:   Item 1/3 initial value: Str("hello")
# DEBUG:   Item 1/3 applying step 1: Upper
# DEBUG:   Item 1/3 step 1 result: String("HELLO")
# DEBUG: Processing item 2 of 3: "world"
# DEBUG:   Item 2/3 initial value: Str("world")
# DEBUG:   Item 2/3 applying step 1: Upper
# DEBUG:   Item 2/3 step 1 result: String("WORLD")
# DEBUG: Processing item 3 of 3: "test"
# DEBUG:   Item 3/3 initial value: Str("test")
# DEBUG:   Item 3/3 applying step 1: Upper
# DEBUG:   Item 3/3 step 1 result: String("TEST")
# DEBUG: Map operation completed. Results:
# DEBUG:   Item 1: "HELLO"
# DEBUG:   Item 2: "WORLD"
# DEBUG:   Item 3: "TEST"
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "HELLO"
# DEBUG:   [1]: "WORLD"
# DEBUG:   [2]: "TEST"
# DEBUG: ---
# DEBUG: Applying operation 3: Join { sep: "_" }
# DEBUG: Result: String("HELLO_WORLD_TEST")
# DEBUG: ---
# HELLO_WORLD_TEST
```

### Real-world Use Cases

```sh
# Extract domains from URLs
echo "https://github.com,https://google.com" | string-pipeline "{split:,:..|map:{regex_extract://([^/]+):1}}"
# Output: github.com,google.com

# Format CSV data
echo "John,25,Engineer" | string-pipeline "{split:,:..|map:{pad:15: :both}|join:\||append:\||prepend:\|}"
# Output: |     John      |      25       |   Engineer    |

# Clean log data
echo "2023-01-01 ERROR Failed,2023-01-02 INFO Success" | string-pipeline "{split:,:..|map:{regex_extract:\\d{4}-\\d{2}-\\d{2}}|join:\\n}"
# Output: 2023-01-01
#         2023-01-02
```

## Testing

```sh
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Contributing

Contributions welcome! Please see our [documentation](docs/template-system.md) for syntax details.

- 🐛 Report bugs via [GitHub Issues](https://github.com/lalvarezt/string_pipeline/issues)
- 💡 Suggest features or improvements
- 🔧 Submit pull requests

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**⚡ Fast, composable string transformations made simple!**

📖 **[Read the full documentation](docs/template-system.md)** for complete syntax reference and advanced examples.
