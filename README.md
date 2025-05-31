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
- **Tested**: Comprehensive test suite.

## Usage

### As a CLI

```sh
cargo run -- "input string" "{template}"
```

**Examples:**

```sh
# Get the second item in a comma-separated list
cargo run -- "a,b,c" "{split:,:1}"
# Output: b

# Replace all spaces with underscores and uppercase
cargo run -- "foo bar baz" "{replace:s/ /_/g:upper}"
# Output: FOO_BAR_BAZ

# Trim, split, and append
cargo run -- " a, b,c , d , e " "{split:,:..:trim:append:!}"
# Output: a!,b!,c!,d!,e!
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
cargo run -- "a,b,c" "{split:,:-1}"
# Output: c

# Replace 'foo' with 'bar'
cargo run -- "foo foo" "{replace:s/foo/bar/g}"
# Output: bar bar

# Uppercase, then append
cargo run -- "hello" "{upper:append:!}"
# Output: HELLO!

# Prepend with a colon (escaped)
cargo run -- "bar" "{prepend:\:foo}"
# Output: :foobar
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
```

## Running Tests

```sh
cargo test
```

## License

MIT

---

**Enjoy fast, composable string transformations!**  
Contributions and suggestions welcome.
