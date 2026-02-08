# Command Line Interface

This document covers the `string-pipeline` CLI behavior and supported flags.

## Contents

- [Command Format](#command-format)
- [Template Input](#template-input)
- [Data Input](#data-input)
- [Debug and Validation](#debug-and-validation)
- [Help Commands](#help-commands)
- [Common Patterns](#common-patterns)
- [Exit Behavior](#exit-behavior)
- [Troubleshooting](#troubleshooting)

## Command Format

```bash
string-pipeline [OPTIONS] [TEMPLATE] [INPUT]
```

Arguments:

- `TEMPLATE`: template string
- `INPUT`: input string (optional when using `--validate`; otherwise read from argument, file, or `stdin`)

## Template Input

Choose one template source:

- positional `TEMPLATE`
- `--template-file FILE` (`-t FILE`)

If both are provided, the command fails.

Examples:

```bash
# Inline template
string-pipeline '{upper}' 'hello world'

# Template from file
printf '{split:,:..|map:{upper}|join:-}' > transform.template
printf 'a,b,c\n' | string-pipeline -t transform.template
```

## Data Input

Input source priority:

1. positional `INPUT`
2. `--input-file FILE` (`-f FILE`)
3. `stdin`

If both `input` and `--input-file` are provided, the command fails.

Examples:

```bash
# Positional input
string-pipeline '{upper}' 'hello world'

# Input file
printf 'hello world\n' > input.txt
string-pipeline '{upper}' -f input.txt

# stdin
printf 'hello world\n' | string-pipeline '{upper}'
```

## Debug and Validation

### Debug mode

Enable debug output with:

- inline debug flag: `{!...}`
- CLI flag: `--debug` (`-d`)

Behavior:

- Debug logs go to `stderr`.
- Final result goes to `stdout`.
- `--quiet` (`-q`) suppresses debug logs.

Examples:

```bash
# Inline debug
string-pipeline '{!split:,:..|map:{upper}}' 'hello,world'

# CLI debug
string-pipeline -d '{split:,:..|map:{upper}}' 'hello,world'

# Debug requested, logs suppressed by quiet mode
string-pipeline -d -q '{split:,:..|map:{upper}}' 'hello,world'
```

### Template validation

`--validate` checks template syntax without processing input.

Examples:

```bash
# Validation success
string-pipeline --validate '{split:,:..|map:{upper}|join:-}'

# Quiet validation (no output on success)
string-pipeline --validate -q '{split:,:..|map:{upper}|join:-}'
```

## Help Commands

Supported informational flags:

- `--help`, `-h`
- `--version`, `-V`
- `--list-operations`
- `--syntax-help`

Examples:

```bash
string-pipeline --help
string-pipeline --version
string-pipeline --list-operations
string-pipeline --syntax-help
```

## Common Patterns

### Build templates incrementally

```bash
string-pipeline '{split:,:..}' 'a,b,c'
string-pipeline '{split:,:..|map:{upper}}' 'a,b,c'
string-pipeline '{split:,:..|map:{upper}|join:-}' 'a,b,c'
```

### Validate before running

```bash
printf '{split:,:..|map:{trim|upper}|unique|sort}' > normalize.template
if string-pipeline --validate --template-file normalize.template; then
  string-pipeline --template-file normalize.template 'a,b,c'
fi
```


## Exit Behavior

- Exit code `0`: success
- Exit code `1`: parse error, I/O error, validation failure, or runtime processing error

Behavior notes:

- `--validate` does not require input.
- If no template is provided and `stdin` is not available, the CLI prints help.

## Troubleshooting

### Parse errors

- Check missing braces and separators.
- Run `--validate` first.

```bash
string-pipeline --validate '{split:,:..|map:{upper}|join:-}'
```

### Type errors

- `map` requires list input.
- String-only operations on lists should be inside `map:{...}`.

```bash
# Error: upper on list
string-pipeline '{split:,:..|upper}' 'a,b,c'

# Correct
string-pipeline '{split:,:..|map:{upper}}' 'a,b,c'
```

### Debugging command behavior

```bash
string-pipeline -d '{split:,:..|map:{upper}|join:-}' 'a,b,c'
```

Related documentation:

- `docs/template-system.md`
- `docs/debug-system.md`
- `docs/benchmarking.md`
