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
- [More Examples](#more-examples)
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
- **Map**: Apply a sub-pipeline to each list item.
- **Sort, reverse, unique**: List operations for sorting, reversing, deduplication.
- **Pad**: Pad strings or list items to a given width.
- **Regex extract**: Extract regex matches or groups.
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
string_pipeline = "0.8.1"
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

# Using map to uppercase each item
string-pipeline "{split:,:..|map:{upper}}" "a,b,c"
# Output: A,B,C

# Sort and join
string-pipeline "{split:,:..|sort:desc|join:-}" "b,a,c"
# Output: c-b-a

# Pad each item to width 3 with '*'
string-pipeline "{split:,:..|map:{pad:3:*:both}}" "a,bb,c"
# Output: *a*,bb*,*c*

# Extract numbers from each item
string-pipeline "{split:,:..|map:{regex_extract:\d+}}" "a1,b22,c333"
# Output: 1,22,333
```

### As a Library

```rust
use string_pipeline::Template;

fn main() {
    let template = Template::parse("{split:,:..|join:\\n}").unwrap();
    let result = template.format("a,b,c").unwrap();
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
  - `trim[:left|right|both]`
  - `strip:<chars>`
  - `append:<suffix>`
  - `prepend:<prefix>`
  - `strip_ansi`
  - `filter:<regex_pattern>`
  - `filter_not:<regex_pattern>`
  - `slice:<range>`
  - `map:{<operation_list>}`
  - `sort[:asc|desc]`
  - `reverse`
  - `unique`
  - `pad:<width>[:<char>][:left|right|both]`
  - `regex_extract:<pattern>[:<group>]`

#### Supported Operations

| Operation    | Syntax                                      | Description                                         |
| ------------ | ------------------------------------------- | --------------------------------------------------- |
| Split        | `split:<sep>:<range>`                       | Split by separator, select by index/range           |
| Join         | `join:<sep>`                                | Join a list with separator                          |
| Substring    | `substring:<range>`                         | Extract substring(s) by character index/range       |
| Replace      | `replace:s/<pattern>/<replacement>/<flags>` | Regex replace (sed-like, supports flags)            |
| Uppercase    | `upper`                                     | Convert to uppercase                                |
| Lowercase    | `lower`                                     | Convert to lowercase                                |
| Trim         | `trim[:left\|right\|both]`                  | Trim whitespace (or side-specific)                  |
| Strip        | `strip:<chars>`                             | Strip custom characters from both ends              |
| Append       | `append:<suffix>`                           | Append text                                         |
| Prepend      | `prepend:<prefix>`                          | Prepend text                                        |
| StripAnsi    | `strip_ansi`                                | Remove ANSI escape sequences                        |
| Filter       | `filter:<regex_pattern>`                    | Keep only items matching regex pattern              |
| FilterNot    | `filter_not:<regex_pattern>`                | Remove items matching regex pattern                 |
| Slice        | `slice:<range>`                             | Select elements from a list by index/range          |
| Map          | `map:{<operation_list>}`                    | Apply a sub-pipeline to each list item              |
| Sort         | `sort[:asc\|desc]`                          | Sort list ascending/descending                      |
| Reverse      | `reverse`                                   | Reverse string or list                              |
| Unique       | `unique`                                    | Remove duplicate items from a list                  |
| Pad          | `pad:<width>[:<char>][:left\|right\|both]`  | Pad string/list items to width with char/side       |
| RegexExtract | `regex_extract:<pattern>[:<group>]`         | Extract first match or group from string/list items |

#### Range Specifications

Ranges use Rust-like syntax and support negative indices like Python:

| Range | Description               | Example                            |
| ----- | ------------------------- | ---------------------------------- |
| `N`     | Single index              | `{split:,:1}`     → second element   |
| `N..M`  | Exclusive range           | `{split:,:1..3}`  → elements 1,2     |
| `N..=M` | Inclusive range           | `{split:,:1..=3}` → elements 1,2,3   |
| `N..`   | From N to end             | `{split:,:2..}`   → from 2nd to end  |
| `..N`   | From start to N           | `{split:,:..3}`   → first 3 elements |
| `..=N`  | From start to N inclusive | `{split:,:..=2}`  → first 3 elements |
| `..`    | All elements              | `{split:,:..}`    → all elements     |

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

## More examples

```sh
# Get the last item
string-pipeline "{split:,:-1}" "a,b,c"
# Output: c

# Get the second word using the short form (only works with space separator)
string-pipeline "{1}" "foo bar baz"
# Output: bar

# Get a range of items
string-pipeline "{split:,:1..=3}" "a,b,c,d,e"
# Output: b,c,d

# Get a range of words using the short form (only works with space separator)
string-pipeline "{1..3}" "foo bar baz qux"
# Output: bar baz

# Get a range of items, another way
string-pipeline "{split:,:..|slice:1..=3}" "a,b,c,d,e"
# Output: b,c,d

# Substring string characters
string-pipeline "{substring:1..=3}" "hello"
# Output: ell

# Replace 'foo' with 'bar' globally
string-pipeline "{replace:s/foo/bar/g}" "foo foo"
# Output: bar bar

# Chain operations: uppercase, then append
string-pipeline "{upper|append:!}" "hello"
# Output: HELLO!

# Prepend with a colon (escaped)
string-pipeline "{prepend:\:foo}" "bar"
# Output: :foobar

# Map: uppercase each item
string-pipeline "{split:,:..|map:{upper}}" "a,b,c"
# Output: A,B,C

# Sort, reverse, unique, pad, regex_extract
string-pipeline "{split:,:..|sort:desc|join:-}" "b,a,c"
# Output: c-b-a

string-pipeline "{split:,:..|reverse}" "a,b,c"
# Output: c,b,a

string-pipeline "{split:,:..|unique}" "a,b,a,c"
# Output: a,b,c

string-pipeline "{split:,:..|map:{pad:3:*:both}}" "a,bb,c"
# Output: *a*,bb*,*c*

string-pipeline "{split:,:..|map:{regex_extract:\\d+}}" "a1,b22,c333"
# Output: 1,22,333
```

### Debug Mode

```sh
# Print debug info for each operation
string-pipeline "{!split:,:..|map:{trim|upper}}" "user123,   admin456   ,guest789"
# DEBUG: Initial value: Str("user123,   admin456   ,guest789")
# DEBUG: Applying operation 1: Split { sep: ",", range: Range(None, None, false) }
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "user123"
# DEBUG:   [1]: "   admin456   "
# DEBUG:   [2]: "guest789"
# DEBUG: ---
# DEBUG: Applying operation 2: Map { operations: [Trim { direction: Both }, Upper] }
# DEBUG: Map operation starting with 3 items
# DEBUG: Map operations to apply: 2 steps
# DEBUG:   Step 1: Trim { direction: Both }
# DEBUG:   Step 2: Upper
# DEBUG: Processing item 1 of 3: "user123"
# DEBUG:   Item 1/3 initial value: Str("user123")
# DEBUG:   Item 1/3 applying step 1: Trim { direction: Both }
# DEBUG:   Item 1/3 step 1 result: String("user123")
# DEBUG:   Item 1/3 applying step 2: Upper
# DEBUG:   Item 1/3 step 2 result: String("USER123")
# DEBUG: Processing item 2 of 3: "   admin456   "
# DEBUG:   Item 2/3 initial value: Str("   admin456   ")
# DEBUG:   Item 2/3 applying step 1: Trim { direction: Both }
# DEBUG:   Item 2/3 step 1 result: String("admin456")
# DEBUG:   Item 2/3 applying step 2: Upper
# DEBUG:   Item 2/3 step 2 result: String("ADMIN456")
# DEBUG: Processing item 3 of 3: "guest789"
# DEBUG:   Item 3/3 initial value: Str("guest789")
# DEBUG:   Item 3/3 applying step 1: Trim { direction: Both }
# DEBUG:   Item 3/3 step 1 result: String("guest789")
# DEBUG:   Item 3/3 applying step 2: Upper
# DEBUG:   Item 3/3 step 2 result: String("GUEST789")
# DEBUG: Map operation completed. Results:
# DEBUG:   Item 1: "USER123"
# DEBUG:   Item 2: "ADMIN456"
# DEBUG:   Item 3: "GUEST789"
# DEBUG: Result: List with 3 items:
# DEBUG:   [0]: "USER123"
# DEBUG:   [1]: "ADMIN456"
# DEBUG:   [2]: "GUEST789"
# DEBUG: ---
# USER123,ADMIN456,GUEST789
```

---

## Testing

Run the test suite:

```sh
cargo test
```

---

## Benchmarks

Run the bench suite:

```sh
cargo bench
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
