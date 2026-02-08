# 🔗 String Pipeline

[![Crates.io](https://img.shields.io/crates/v/string_pipeline.svg)](https://crates.io/crates/string_pipeline)
[![Docs.rs](https://docs.rs/string_pipeline/badge.svg)](https://docs.rs/string_pipeline)
[![CI](https://github.com/lalvarezt/string_pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/lalvarezt/string_pipeline/actions)
[![License](https://img.shields.io/crates/l/string_pipeline.svg)](https://github.com/lalvarezt/string_pipeline/blob/main/LICENSE)

`string_pipeline` is a Rust library for string transformation pipelines.

The `string-pipeline` CLI is a companion interface for exercising the same template engine outside your Rust code (quick
checks, validation, and debug tracing).

Templates chain operations such as split, map, filter, replace, and join:

```text
{split:,:..|map:{upper}|join:-}
```

## Contents

- [Why Not awk/sed/etc](#why-not-awksedetc)
- [Examples](#examples)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Development](#development)
- [License](#license)

## Why Not awk/sed/etc

`awk`, `sed`, and similar tools are strong choices and remain the right default for many shell tasks.

`string_pipeline` is intended for library-first workflows where text transformations live in Rust code and use an
explicit template format. The CLI exists to run that same engine externally during development and troubleshooting.

Use `string_pipeline` when you want:

- Templates checked into a Rust project and reused directly in application code
- One template format shared by Rust API and CLI checks
- Built-in template validation (`--validate`) and execution tracing (`--debug`)
- Structured operation chains (`split|map|filter|join`) instead of shell-specific one-liners
- Per-item sub-pipelines with `map:{...}` and explicit range handling

Use `awk`/`sed` when you want:

- Quick one-off line edits in shell scripts
- Full control of custom parsing/state logic in a single script
- Minimal dependency footprint on systems where those tools are already standard

Common library-side use cases:

- Normalize or reformat delimited text
- Extract fields with ranges or regex
- Apply per-item transformations with `map`
- Use mixed literal/template output (`"Name: {split: :0}"`)

## Examples

Examples below use the CLI for brevity. The same templates are parsed and executed by the Rust library.

```bash
# Extract and sort email domains
string-pipeline "{split:,:..|map:{regex_extract:@(.+):1}|sort}" "john.doe@email.com,jane.smith@company.org"
# company.org,email.com

# Normalize names
string-pipeline '{split:,:..|map:{trim|upper|append:!}}' "  john  , jane , bob  "
# JOHN!,JANE!,BOB!

# Keep Python files and print one per line
string-pipeline '{split:,:..|filter:\.py$|sort|map:{prepend:- }|join:\n}' 'app.py,readme.md,test.py,data.json'
# - app.py
# - test.py
```

## Installation

### Library (primary)

Add to `Cargo.toml`:

```toml
[dependencies]
string_pipeline = "0.13.0"
```

### CLI (companion)

Optional, for running templates outside your Rust program:

```bash
cargo install string_pipeline
```

Build from source:

```bash
git clone https://github.com/lalvarezt/string_pipeline.git
cd string_pipeline
cargo install --path .
```

## Quick Start

### Rust API (primary)

```rust
use string_pipeline::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::parse("{split:,:..|map:{upper}|join:-}")?;
    let result = template.format("hello,world,rust")?;
    assert_eq!(result, "HELLO-WORLD-RUST");
    Ok(())
}
```

### CLI (companion)

Use the CLI to test the same templates externally.

```bash
# Positional input
string-pipeline '{split:,:..|map:{upper}|join:-}' 'hello,world,rust'
# HELLO-WORLD-RUST

# stdin input
printf 'hello,world\n' | string-pipeline '{split:,:..|map:{upper}|join:-}'
# HELLO-WORLD

# Validate template
string-pipeline --validate '{split:,:..|map:{upper}|join:-}'
```

### Debug view (CLI)

```bash
string-pipeline '{!split:,:..|map:{upper}|join:-}' 'hello,world'
```

Example debug excerpt (`stderr`):

```text
DEBUG: 📂 MULTI-TEMPLATE
DEBUG: ├── 🏁 MULTI-TEMPLATE START
DEBUG: ├── Template: "{!split:,:..|map:{upper}|join:-}"
DEBUG: ├── ➡️ Input: "hello,world"
DEBUG: ├── 📊 SECTION 1/1: [template: split(',', ..) | map { operations: [upper] } | join { sep: "-" }]
DEBUG: ├── 📂 Main Pipeline
DEBUG: │   ├── 🚀 PIPELINE START: 3 operations
DEBUG: │   ├── 1. Split(',')
DEBUG: │   ├── 2. Map(1)
DEBUG: │   ├── 3. Join('-')
DEBUG: │   └── ... per-step results and timings ...
DEBUG: └── Cache stats: ...
```

Final result (`stdout`):

```text
HELLO-WORLD
```

## Documentation

- `docs/template-system.md`
- `docs/command-line-options.md`
- `docs/debug-system.md`
- `docs/benchmarking.md`

API docs: <https://docs.rs/string_pipeline>

## Development

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Build benchmark helper binary
cargo build --release --bin string-pipeline-bench
./target/release/string-pipeline-bench
```

## License

MIT. See `LICENSE`.
