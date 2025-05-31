# String Template Processor

A flexible, composable string transformation CLI tool and library for Rust originally created as a parser for [television](https://github.com/alexpasmantier/television). It allows you to chain operations like split, join, slice, replace, case conversion, trim, and more, using a concise template syntax.

## Features

- **Composable operations**: Chain multiple string operations in a single template.
- **Split and join**: Extract and reassemble parts of strings.
- **Slice and range**: Select substrings or sublists by index or range.
- **Replace**: Regex-based search and replace, with sed-like syntax.
- **Case conversion**: Uppercase and lowercase.
- **Trim and strip**: Remove whitespace or custom characters.
- **Append and prepend**: Add text before or after.
- **Escaping**: Use `\:` and `\\` to include literal colons and backslashes in arguments.
- **Negative indices**: Support for negative indices (like Python) in ranges and slices.
- **Stdin support**: Read input from stdin when no input argument is provided.
- **Tested**: Comprehensive test suite.

## Usage

### As a CLI

```sh
# With input argument
cargo run -- "{template}" "input string"

# With stdin input
echo "input string" | cargo run -- "{template}"
```

**Examples:**

```sh
# Get the second item in a comma-separated list
cargo run -- "{split:,:1}" "a,b,c"
# Output: b

# Using stdin
echo "a,b,c" | cargo run -- "{split:,:1}"
# Output: b

# Replace all spaces with underscores and uppercase
cargo run -- "{replace:s/ /_/g:upper}" "foo bar baz"
# Output: FOO_BAR_BAZ

# Trim, split, and append
cargo run -- "{split:,:..:trim:append:!}" " a, b,c , d , e "
# Output: a!,b!,c!,d!,e!

# Using stdin for processing file content
cat data.txt | cargo run -- "{split:\n:..:trim:prepend:- }"

# Pipeline processing
echo "hello,world,test" | cargo run -- "{split:,:0..2:join: | :upper}"
# Output: HELLO | WORLD
```

### Template Syntax

Templates are enclosed in `{}` and consist of a chain of operations separated by `:`.  
Arguments to operations are separated by `:` as well.  
To include a literal `:` or `\` in an argument, escape it as `\:` or `\\`.

**Supported operations:**

- `split:<sep>:<range>` — Split by separator, select by index or range.
- `join:<sep>` — Join a list with separator.
- `slice:<range>` — Slice string or list elements by range.
- `replace:s/<pattern>/<replacement>/<flags>` — Regex replace (sed-like).
- `upper` — Uppercase.
- `lower` — Lowercase.
- `trim` — Trim whitespace.
- `strip:<chars>` — Trim custom characters.
- `append:<suffix>` — Append text.
- `prepend:<prefix>` — Prepend text.

**Range syntax:**

- Single index: `2` or `-1`
- Range: `1..3`, `..2`, `2..`, `1..=3`, `-3..-1`, etc.

**Examples:**

```sh
# Get the last item
cargo run -- "{split:,:-1}" "a,b,c"
# Output: c

# Get a range of items
cargo run -- "{split:,:1..=3}" "a,b,c,d,e"
# Output: b,c,d

# Replace 'foo' with 'bar' globally
cargo run -- "{replace:s/foo/bar/g}" "foo foo"
# Output: bar bar

# Chain operations: uppercase, then append
cargo run -- "{upper:append:!}" "hello"
# Output: HELLO!

# Prepend with a colon (escaped)
cargo run -- "{prepend:\:foo}" "bar"
# Output: :foobar

# Complex chaining: split, select range, join, replace, uppercase
cargo run -- "{split:,:0..2:join:-:replace:s/a/X/:upper}" "a,b,c"
# Output: X-B

# Slice string characters
cargo run -- "{slice:1..=3}" "hello"
# Output: ell

# Split, trim each item, then prepend
echo " a , b , c " | cargo run -- "{split:,:..:trim:prepend:item_}"
# Output: item_a,item_b,item_c

# Strip custom characters
cargo run -- "{strip:xy}" "xyhelloxy"
# Output: hello
```

### Advanced Examples

```sh
# Process CSV-like data
echo "name,age,city" | cargo run -- "{split:,:1..}" 
# Output: age,city

# Format file paths
echo "/home/user/documents/file.txt" | cargo run -- "{split:/:-1:prepend:dir }"
# Output: dir file.txt

# Extract file extension
echo "document.pdf" | cargo run -- "{split:.:-1:upper}"
# Output: PDF

# Process log entries with timestamps
echo "2023-01-01 ERROR Failed to connect" | cargo run -- "{split: :1..:join: :lower}"
# Output: error failed to connect
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
clap = "4"
regex = "1"
```

Use in your code:

```rust
use your_crate::process;

let result = process("foo,bar,baz", "{split:,:1:upper}").unwrap();
assert_eq!(result, "BAR");

// Chain multiple operations
let result = process("  hello world  ", "{trim:split: :join:_:upper}").unwrap();
assert_eq!(result, "HELLO_WORLD");

// Work with ranges
let result = process("a,b,c,d,e", "{split:,:1..=3:join:-}").unwrap();
assert_eq!(result, "b-c-d");
```

## Error Handling

The tool provides helpful error messages for common issues:

```sh
# Invalid template format
cargo run -- "split:,:0" "test"
# Error: Template must start with '{' and end with '}'

# Invalid range
cargo run -- "{split:,:abc}" "a,b,c"  
# Error: Invalid index

# Invalid regex
cargo run -- "{replace:s/[/replacement/}" "test"
# Error: regex parse error
```

## Running Tests

```sh
cargo test
```

## Use Cases

- **Data extraction**: Parse CSV, logs, or structured text
- **Text transformation**: Clean and format strings in pipelines
- **File processing**: Extract parts of filenames or paths
- **Configuration parsing**: Process environment variables or config files
- **Shell scripting**: Quick text manipulation in scripts

## License

MIT

---

**Enjoy fast, composable string transformations!**  
Contributions and suggestions welcome.
