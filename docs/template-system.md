# Template System

This document describes the template syntax used by `string_pipeline`.

## Contents

- [Quick Start](#quick-start)
- [Template Syntax](#template-syntax)
- [Evaluation Rules](#evaluation-rules)
- [Multi-template Strings](#multi-template-strings)
- [Operation Reference](#operation-reference)
- [Range Specifications](#range-specifications)
- [Escaping Rules](#escaping-rules)
- [Map Semantics](#map-semantics)
- [Debug Mode](#debug-mode)
- [Troubleshooting](#troubleshooting)

## Quick Start

Templates are enclosed in braces and operations are separated by `|`.

```text
{operation1|operation2|operation3}
```

Example:

```bash
string-pipeline "{split:,:..|map:{upper}|join:-}" "hello,world,test"
# HELLO-WORLD-TEST
```

## Template Syntax

```text
{[!][operation[|operation...]]}
```

| Component      | Required | Description                                |
|----------------|----------|--------------------------------------------|
| `{` `}`        | yes      | Template delimiters                        |
| `!`            | no       | Debug flag, immediately after `{`          |
| operation list | no       | One or more operations separated by a pipe |

Notes:

- `{}` is valid and returns the input unchanged.
- Operations are evaluated from left to right.
- A template can contain either only a template block (`{...}`) or literal text with one or more blocks (multi-template
mode).

### Shorthand syntax

Shorthand index/range syntax is equivalent to splitting on a space separator.

```text
{1}      == {split: :1}
{1..3}   == {split: :1..3}
{..}     == {split: :..}
```

## Evaluation Rules

The pipeline works with two runtime value types:

- `String`
- `List<String>`

Operations are type-checked during execution.

### Type categories

| Category         | Operations                                                                                                                       |
|------------------|----------------------------------------------------------------------------------------------------------------------------------|
| string -> string | `replace`, `upper`, `lower`, `trim`, `substring`, `append`, `prepend`, `surround`, `quote`, `strip_ansi`, `pad`, `regex_extract` |
| list -> list     | `slice`, `sort`, `unique`, `map`                                                                                                 |
| type-preserving  | `filter`, `filter_not`, `reverse`                                                                                                |
| type-converting  | `split`, `join`                                                                                                                  |

### Final list rendering

If a pipeline ends with a list and no explicit `join`, the list is rendered as a string using the separator from the
most recent `split` or `join` operation in that pipeline.

```text
{split:,:..}                    # "a,b,c" -> "a,b,c"
{split:,:..|sort}               # "c,a,b" -> "a,b,c"
{split:,:..|join:-}             # "a,b,c" -> "a-b-c"
{split:\|:..|split:a:..}        # "apple|banana|cherry" -> "appleabananaacherry"
```

Use an explicit `join` as the final step when output format must be fixed.

## Multi-template Strings

A multi-template combines literal text and one or more template sections.

```text
User: {split:,:0} Score: {split:,:1}
```

Examples:

```bash
string-pipeline "Hello {upper}, welcome" "world"
# Hello WORLD, welcome

string-pipeline "Name: {split: :0} Age: {split: :1}" "John 25"
# Name: John Age: 25

string-pipeline "Host: {split: :0|split:=:1} Port: {split: :1|split:=:1} SSL: {split: :-1|split:=:1|upper}" "host=localhost port=8080 ssl=true"
# Host: localhost Port: 8080 SSL: TRUE
```

### Caching behavior

Within one `format()` call, repeated template sections with the same operation sequence and input are cached.

```bash
string-pipeline "First: {split:,:0} Again: {split:,:0}" "apple,banana,cherry"
# First: apple Again: apple
```

## Operation Reference

### split

- Syntax: `split:SEPARATOR:RANGE`
- Input: string or list
- Output: string (index range) or list (range)

Notes:

- `RANGE` is required; use `..` for all parts.
- For list input, each item is split and the results are flattened.

```text
{split:,:..}            # split all items by comma
{split: :0..2}          # keep first two parts
{split:\n:-1}          # keep last line

{split: :..|map:{append:,x}|split:,:..|join:-}
# "a b" -> "a-x-b-x"
```

### slice

- Syntax: `slice:RANGE`
- Input: list
- Output: list

```text
{split:,:..|slice:1..3}   # "a,b,c,d" -> "b,c"
```

### join

- Syntax: `join:SEPARATOR`
- Input: list or string
- Output: string

Behavior:

- On lists, joins items using `SEPARATOR`.
- On strings, returns the input unchanged.

```text
{split:,:..|join:-}       # "a,b,c" -> "a-b-c"
{split:,:..|join:}        # "a,b,c" -> "abc"
{join:-}                  # "hello" -> "hello"
```

### substring

- Syntax: `substring:RANGE`
- Input: string
- Output: string

```text
{substring:1..4}          # "hello" -> "ell"
{substring:-3..}          # "hello" -> "llo"
```

### trim

- Syntax: `trim[:CHARS][:DIRECTION]`
- Input: string
- Output: string
- `DIRECTION`: `both` (default), `left`, `right`

```text
{trim}                    # trim whitespace
{trim:left}               # trim from start only
{trim:xy}                 # trim x/y from both ends
{trim:*-+:right}          # trim from right only
```

### pad

- Syntax: `pad:WIDTH[:CHAR[:DIRECTION]]`
- Input: string
- Output: string
- `DIRECTION`: `left`, `right` (default), `both`

```text
{pad:5}                   # "hi" -> "hi   "
{pad:5:0:left}            # "42" -> "00042"
```

### upper

- Syntax: `upper`
- Input: string
- Output: string

```text
{upper}                   # "hello" -> "HELLO"
```

### lower

- Syntax: `lower`
- Input: string
- Output: string

```text
{lower}                   # "HELLO" -> "hello"
```

### append

- Syntax: `append:TEXT`
- Input: string
- Output: string

```text
{append:.txt}             # "file" -> "file.txt"
```

### prepend

- Syntax: `prepend:TEXT`
- Input: string
- Output: string

```text
{prepend:/tmp/}           # "file.txt" -> "/tmp/file.txt"
```

### surround

- Syntax: `surround:TEXT`
- Input: string
- Output: string

```text
{surround:"}             # "hello" -> "\"hello\""
{surround:**}             # "text" -> "**text**"
```

### quote

- Syntax: `quote:TEXT`
- Input: string
- Output: string

`quote` is an alias of `surround`.

```text
{quote:'}                 # "hello" -> "'hello'"
```

### replace

- Syntax: `replace:s/PATTERN/REPLACEMENT/FLAGS`
- Input: string
- Output: string
- Supported flags: `g`, `i`, `m`, `s`

```text
{replace:s/hello/hi/}     # first match
{replace:s/\d+/NUM/g}     # global replacement
{replace:s/(.+)/[$1]/}    # capture groups
```

### regex_extract

- Syntax: `regex_extract:PATTERN[:GROUP]`
- Input: string
- Output: string

If there is no match, returns an empty string.

```text
{regex_extract:\d+}        # first number
{regex_extract:@(.+):1}    # group extraction
```

### sort

- Syntax: `sort[:DIRECTION]`
- Input: list
- Output: list
- `DIRECTION`: `asc` (default), `desc`

```text
{split:,:..|sort}          # "c,a,b" -> "a,b,c"
{split:,:..|sort:desc}     # "a,b,c" -> "c,b,a"
```

### reverse

- Syntax: `reverse`
- Input: string or list
- Output: same type as input

```text
{reverse}                  # "hello" -> "olleh"
{split:,:..|reverse}       # "a,b,c" -> "c,b,a"
```

### unique

- Syntax: `unique`
- Input: list
- Output: list

Keeps first occurrence order.

```text
{split:,:..|unique}        # "a,b,a,c,b" -> "a,b,c"
```

### filter

- Syntax: `filter:PATTERN`
- Input: string or list
- Output: same type as input

```text
{split:,:..|filter:^test}  # keep list items starting with "test"
```

### filter_not

- Syntax: `filter_not:PATTERN`
- Input: string or list
- Output: same type as input

```text
{split:,:..|filter_not:^#} # remove items starting with "#"
```

### strip_ansi

- Syntax: `strip_ansi`
- Input: string
- Output: string

```text
{strip_ansi}               # remove ANSI escape sequences
```

### map

- Syntax: `map:{operation1|operation2|...}`
- Input: list
- Output: list

Notes:

- Nested `map` is not allowed.
- String operations and list operations are both available inside `map`.

```text
{split:,:..|map:{trim|upper}}                    # " a , b " -> "A,B"
{split:,:..|map:{split: :..|join:-}}             # "hello world,foo bar" -> "hello-world,foo-bar"
{split:,:..|map:{split: :..|filter:o}}           # "hello world,foo bar,test orange" -> "hello world,foo,orange"
```

### shorthand index and ranges

Shorthand forms operate as `split` with a space separator.

```text
{1}                      # "a b c d" -> "b"
{-1}                     # "a b c d" -> "d"
{1..3}                   # "a b c d" -> "b c"
{1..=3}                  # "a b c d" -> "b c d"
{..3}                    # "a b c d" -> "a b c"
{..}                     # "a b c d" -> "a b c d"
```

## Range Specifications

Ranges are used by `split`, `slice`, `substring`, and shorthand syntax.

| Syntax  | Description         |
|---------|---------------------|
| `N`     | single index        |
| `N..M`  | exclusive range     |
| `N..=M` | inclusive range     |
| `N..`   | from index to end   |
| `..M`   | from start to `M-1` |
| `..=M`  | from start to `M`   |
| `..`    | full range          |

Negative indexes count from the end (`-1` is last item).

Edge behavior:

- Single indexes are clamped to valid bounds (out-of-range resolves to nearest valid index).
- Ranges are clamped to valid bounds.
- If computed start is greater than or equal to end, the result is empty.
- Empty input always returns empty output.

## Escaping Rules

### Simple arguments

For operations such as `append`, `prepend`, `join`, `surround`, `quote`, and `trim` arguments, escape these characters when needed:

| Character | Escape |
|-----------|--------|
| `:`       | `\:`   |
| pipe      | `\|`   |
| `{`       | `\{`   |
| `}`       | `\}`   |
| `\`       | `\\`   |

### Special escape sequences

`process_arg` supports these sequences in simple/split arguments:

- `\n` newline
- `\t` tab
- `\r` carriage return
- `\:` literal colon
- `\|` literal pipe
- `\\` literal backslash
- `\/` literal slash
- `\{` literal `{`
- `\}` literal `}`

Any other `\X` sequence is treated as literal `X`.

### Regex arguments

For `filter`, `filter_not`, and `regex_extract`, the pattern is read as raw template content and passed to the regex engine.

Examples:

```text
{append:\:value}         # append ":value"
{split:\:\::..|join:-}   # split on "::"
{filter:\.txt$}          # regex pattern for .txt suffix
```

When calling from a shell, prefer single quotes around templates to reduce extra shell escaping.

## Map Semantics

`map` runs a sub-pipeline for each list item.

Important details:

- Each item is processed independently.
- If a map item ends as a list, it is auto-rendered to a string using that sub-pipeline's current separator.
- After mapping, the outer pipeline continues with a list of mapped strings.

```text
Input: "hello world,foo bar,test orange"
Template: {split:,:..|map:{split: :..|filter:o}}
Output: "hello world,foo,orange"
```

## Debug Mode

Enable debug mode with either:

- Inline template flag: `{!...}`
- CLI flag: `--debug` or `-d`

Debug output is written to `stderr`; final result is written to `stdout`.

`--quiet` suppresses debug output even when debug is enabled inline or via CLI.

```bash
string-pipeline "{!split:,:..|map:{upper}|join:-}" "hello,world"
```

For full debug output structure and examples, see `docs/debug-system.md`.

## Troubleshooting

Common issues:

- Parse errors: check missing braces, missing separators, or invalid operation names.
- Type errors: apply string-only operations through `map` when working with lists.
- Empty output: verify regex and range expressions; filter/range steps may remove all items.

Quick checks:

```bash
string-pipeline --validate '{split:,:..|map:{upper}|join:-}'
string-pipeline -d '{split:,:..|map:{upper}|join:-}' 'a,b,c'
```

Related documentation:

- `docs/command-line-options.md`
- `docs/debug-system.md`
- `docs/benchmarking.md`
