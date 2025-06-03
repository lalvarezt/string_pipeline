# String Template Processor

[![Crates.io](https://img.shields.io/crates/v/string_pipeline.svg)](https://crates.io/crates/string_pipeline)
[![Docs.rs](https://docs.rs/string_pipeline/badge.svg)](https://docs.rs/string_pipeline)
[![CI](https://github.com/lalvarezt/string_pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/lalvarezt/string_pipeline/actions)
[![License](https://img.shields.io/crates/l/string_pipeline.svg)](https://github.com/lalvarezt/string_pipeline/blob/main/LICENSE)

---

A flexible, composable string transformation CLI tool and Rust library. `string_pipeline` lets you chain operations like split, join, slice, replace, case conversion, trim, and more, using a concise template syntax. It is ideal for quick text manipulation, scripting, and data extraction.

---

## Table of Contents

- [Features](#features)
- [Crate](#crate)
- [Installation](#installation)
  - [CLI](#cli)
  - [Library](#library)
- [Usage](#usage)
  - [As a CLI](#as-a-cli)
  - [As a Library](#as-a-library)
- [Template Syntax](#template-syntax)
  - [Syntax Reference](#syntax-reference)
  - [Supported Operations](#supported-operations)
  - [Range Specifications](#range-specifications)
  - [Escaping](#escaping)
  - [Debug Mode](#debug-mode)
- [Examples](#examples)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Composable operations**: Chain multiple string operations in a single template.
- **Split and join**: Extract and reassemble parts of strings.
- **Slice and range**: Select sub-strings or sub-lists by index or range.
- **Regex replace**: sed-like regex search and replace.
- **Filtering**: Filter lists with regex patterns using `filter` and `filter_not`.
- **Case conversion**: Uppercase and lowercase.
- **Trim and strip**: Remove whitespace, custom characters, ansi sequences.
- **Append and prepend**: Add text before or after.
- **Smart escaping**: Contextual pipe handling - no escaping needed in most cases.
- **Flexible indices**: Python-like negative indices in ranges and slices with Rust-like syntax.
- **Stdin support**: Read input from stdin when no input argument is provided.
- **Debug mode**: Add `!` after `{` to print debug info for each operation.
- **Robust error handling**: Helpful error messages for invalid templates or arguments.
- **Tested**: Comprehensive test suite.

## Crate

Find the crate on [crates.io](https://crates.io/crates/string_pipeline):

```toml
[dependencies]
string_pipeline = "0.7.0"
```

---

## Installation

### CLI

Clone and build:

```sh
git clone https://github.com/lalvarezt/string_pipeline.git
cd string_pipeline
cargo build --release
```

Or run directly with Cargo:

```sh
cargo run -- "{template}" "input string"
```

### Library

Add to your `Cargo.toml` as shown above.

---

## Usage

### As a CLI

```sh
# With input argument
string-pipeline "{template}" "input string"

# With stdin input
echo "input string" | string-pipeline "{template}"
```

**Examples:**

```sh
# Get the second item in a comma-separated list
string-pipeline "{split:,:1}" "a,b,c"
# Output: b

# Replace all spaces with underscores and uppercase
string-pipeline "{replace:s/ /_/g|upper}" "foo bar baz"
# Output: FOO_BAR_BAZ

# Trim, split, and append
string-pipeline "{split:,:..|trim|append:!}" " a, b,c , d , e "
# Output: a!,b!,c!,d!,e!

# Using stdin for processing file content
cat data.txt | string-pipeline "{split:\n:..|trim|prepend:- }"
```

### As a Library

```rust
use string_pipeline::process;

fn main() {
    let result = process("a,b,c", "{split:,:..|join:\\n}").unwrap();
    assert_eq!(result, "a\nb\nc");
}
```

---

## Template Syntax

Templates are enclosed in `{}` and consist of a chain of operations separated by `|`.
Arguments to operations are separated by `:`.

### Syntax Reference

- **Template**: `{ [!] operation_list? }`
  - Add `!` after `{` to enable debug mode.
- **Operation List**: `operation ('|' operation)*`
- **Operation**:
  - `split:<sep>:<range>`
    - **Shorthand for split**:
      - `{index}` (e.g. `{1}`, equivalent to `{split: :1}`)
      - `{range}` (e.g. `{1..3}`, equivalent to `{split: :1..3}`)
  - `join:<sep>`
  - `substring:<range>`
  - `replace:s/<pattern>/<replacement>/<flags>`
  - `upper`
  - `lower`
  - `trim`
  - `strip:<chars>`
  - `append:<suffix>`
  - `prepend:<prefix>`
  - `strip_ansi`
  - `filter:<regex_pattern>`
  - `filter_not:<regex_pattern>`
  - `slice:<range>`

#### Supported Operations

| Operation         | Syntax                                      | Description                                 |
|-------------------|---------------------------------------------|---------------------------------------------|
| Split             | `split:<sep>:<range>`                         | Split by separator, select by index/range   |
| Join              | `join:<sep>`                                  | Join a list with separator                  |
| Substring         | `slice:<range>`                               | Extract substrings                          |
| Replace           | `replace:s/<pattern>/<replacement>/<flags>`   | Regex replace (sed-like)                    |
| Uppercase         | `upper`                                       | Convert to uppercase                        |
| Lowercase         | `lower`                                       | Convert to lowercase                        |
| Trim              | `trim`                                        | Trim whitespace                             |
| Strip             | `strip:<chars>`                               | Trim custom characters                      |
| Append            | `append:<suffix>`                             | Append text                                 |
| Prepend           | `prepend:<prefix>`                            | Prepend text                                |
| StripAnsi         | `strip_ansi`                                  | Removes ansi escape sequences               |
| Filter            | `filter:<regex_pattern>`                      | Keep only items matching regex pattern      |
| FilterNot         | `filter_not:<regex_pattern>`                  | Remove items matching regex pattern         |
| Slice             | `filter_not:<regex_pattern>`                  | Select elements from a list                 |

#### Range Specifications

Ranges use Rust-like syntax and support negative indices like Python:

| Range | Description | Example |
|-------|-------------|---------|
| `N` | Single index | `{split:,:1}` → second element |
| `N..M` | Exclusive range | `{split:,:1..3}` → elements 1,2 |
| `N..=M` | Inclusive range | `{split:,:1..=3}` → elements 1,2,3 |
| `N..` | From N to end | `{split:,:2..}` → from 2nd to end |
| `..N` | From start to N | `{split:,:..3}` → first 3 elements |
| `..=N` | From start to N inclusive | `{split:,:..=2}` → first 3 elements |
| `..` | All elements | `{split:,:..)` → all elements |

Negative indices count from the end:

- `-1` = last element
- `-2` = second to last element
- `-3..` = last 3 elements

#### Escaping

The parser intelligently handles pipe characters (`|`) based on context:

**Pipes are automatically allowed in:**

- **Split separators**: `{split:|:..}` (splits on pipe)
- **Regex patterns**: `{filter:\.(txt|md|log)}` (alternation)
- **Sed replacements**: `{replace:s/test/a|b/}` (pipe in replacement)

**Manual escaping needed for:**

- **Other arguments**: Use `\|` for literal pipes in join, append, prepend, etc.
- **Special characters**: Use `\:` for literal colons, `\\` for backslashes
- **Escape sequences**: Use `\n`, `\t`, `\r` for newline, tab, carriage return

#### Enable Debug Mode

- Add `!` after `{` to enable debug output for each operation:
  - Example: `{!split:,:..|upper|join:-}`

---

## Examples

### Basic

```sh
# Get the last item
string-pipeline "{split:,:-1}" "a,b,c"
# Output: c

# Get a range of items
string-pipeline "{split:,:1..=3}" "a,b,c,d,e"
# Output: b,c,d

# Replace 'foo' with 'bar' globally
string-pipeline "{replace:s/foo/bar/g}" "foo foo"
# Output: bar bar

# Chain operations: uppercase, then append
string-pipeline "{upper|append:!}" "hello"
# Output: HELLO!

# Prepend with a colon (escaped)
string-pipeline "{prepend:\:foo}" "bar"
# Output: :foobar
```

### Advanced

```sh
# Complex chaining: split, select range, join, replace, uppercase
string-pipeline "{split:,:0..2|join:-|replace:s/a/X/|upper}" "a,b,c"
# Output: X-B

# Slice string characters
string-pipeline "{slice:1..=3}" "hello"
# Output: ell

# Split, trim each item, then prepend
echo " a , b , c " | string-pipeline "{split:,:..|trim|prepend:item_}"
# Output: item_a,item_b,item_c

# Strip custom characters
string-pipeline "{strip:xy}" "xyhelloxy"
# Output: hello
```

### Real-World

```sh
# Process CSV-like data
echo "name,age,city" | string-pipeline "{split:,:1..}"
# Output: age,city

# Format file paths
echo "/home/user/documents/file.txt" | string-pipeline "{split:/:-1}"
# Output: file.txt

# Extract file extension
echo "document.pdf" | string-pipeline "{split:.:-1|upper}"
# Output: PDF

# Process log entries with timestamps
echo "2023-01-01 ERROR Failed to connect" | string-pipeline "{split: :1..|join: |lower}"
# Output: error failed to connect

# Clean colored git output
git log --oneline --color=always | string-pipeline "{split:\\n:..|strip_ansi|join:\\n}"

# Process ls colored output
ls --color=always | string-pipeline "{strip_ansi}"

# Clean grep colored output
grep --color=always "pattern" file.txt | string-pipeline "{strip_ansi|upper}"

# Chain with other operations
echo -e "\x1b[31mred\x1b[0m,\x1b[32mgreen\x1b[0m" | \
  string-pipeline "{split:,:..|strip_ansi|upper|join: \| }"
# Output: RED | GREEN

# Process log files with ANSI codes
cat colored.log | string-pipeline "{split:\n:-10..|strip_ansi|join:\\n}"
```

### Shorthand

```sh
# Get the second word (space-separated)
string-pipeline "{1}" "foo bar baz"
# Output: bar

# Get a range of words
string-pipeline "{1..3}" "foo bar baz qux"
# Output: bar baz
```

### Debug Mode

```sh
# Print debug info for each operation
string-pipeline "{!split:,:..|upper|join:-}" "a,b,c"
# DEBUG: Initial value: Str("a,b,c")
# DEBUG: Applying operation 1: Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "a"
# DEBUG:   [1]: "b"
# DEBUG:   [2]: "c"
# DEBUG: ---
# DEBUG: Applying operation 2: Upper
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "A"
# DEBUG:   [1]: "B"
# DEBUG:   [2]: "C"
# DEBUG: ---
# DEBUG: Applying operation 3: Join { sep: "-" }
# DEBUG: Result: String("A-B-C")
# DEBUG: ---
# A-B-C
```

---

## Testing

Run the test suite:

```sh
cargo test
```

---

## Contributing

Contributions and suggestions are welcome!
Please open issues or pull requests on [GitHub](https://github.com/lalvarezt/string_pipeline).

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**Enjoy fast, composable string transformations!**
