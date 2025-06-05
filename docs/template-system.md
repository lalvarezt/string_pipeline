# 📝 Template System Documentation

A powerful string processing template system with support for splitting, transforming, filtering, and joining operations.

## 📋 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [🏗️ Template Syntax](#️-template-syntax)
- [📊 Operations Reference](#-operations-reference)
- [🎯 Range Specifications](#-range-specifications)
- [🔤 Escaping Rules](#-escaping-rules)
- [🗺️ Map Operations](#️-map-operations)
- [🐛 Debug Mode](#-debug-mode)
- [💡 Examples](#-examples)
- [⚠️ Troubleshooting](#️-troubleshooting)

## 🚀 Quick Start

Templates are enclosed in curly braces `{}` and can contain one or more operations separated by pipes `|`:

```text
{operation1|operation2|operation3}
```

**Basic example:**

```text
Input: "hello,world,test"
Template: "{split:,:..|map:{upper}|join:-}"
Output: "HELLO-WORLD-TEST"
```

## 🏗️ Template Syntax

### Basic Structure

```text
{[!][operation[|operation...]*]}
```

| Component | Required | Description |
|-----------|----------|-------------|
| `{` `}`   | ✅       | Template delimiters |
| `!`       | ❌       | Debug flag (optional, only one time after the left brace) |
| Operations| ❌       | One or more operations separated by `\|` |

### Operation Chaining

Operations are processed left-to-right, with each operation receiving the output of the previous one:

```text
{trim|split: :..|map:{upper}|join:_}
```

1. `trim` - Remove whitespace from both ends
2. `split: :..` - Split on spaces, take all parts
3. `map:{upper}` - Convert each part to uppercase
4. `join:_` - Join with underscores

## 📊 Operations Reference

### 🔪 Splitting Operations

#### Split

Splits input into a list using a separator.

**Syntax:** `split:SEPARATOR:RANGE`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| SEPARATOR | string | ✅ | Character(s) to split on |
| RANGE     | range  | ❌ | Which parts to keep (default: all) |

**Examples:**

```text
{split:,:..}           # Split on comma, keep all
{split: :0..2}         # Split on space, keep items 0,1
{split: :0..=2}        # Split on space, keep items 0,1,2
{split:\n:-1}          # Split on newline, keep last item
```

#### Slice

Extracts a range of items from a list.

**Syntax:** `slice:RANGE`

**Examples:**

```text
{split:,:..|slice:0..2}   # Take items 0,1 after splitting
{split: :..|slice:-3..}   # Take last 3 items
```

### 🔗 Joining Operations

#### Join

Combines list items into a single string with a separator.

**Syntax:** `join:SEPARATOR`

**Examples:**

```text
{split:,:..|join:-}       # Join with hyphen
{split: :..|join:\n}      # Join with newlines
{split:,:..|join:}        # Join with no separator
```

### ✂️ String Manipulation

#### Substring

Extracts characters from a string using range notation.

**Syntax:** `substring:RANGE`

**Examples:**

```text
{substring:0..2}     # Characters 0,1
{substring:-3..}     # Last 3 characters
{substring:..5}      # First 5 characters
{substring:2}        # Single character at index 2
```

#### Trim

Removes specified characters from the beginning and end of strings.

**Syntax:** `trim[:CHARACTERS][:DIRECTION]`

| Direction | Description |
|-----------|-------------|
| `both` (default) | Remove from both ends |
| `left` | Remove from start only |
| `right` | Remove from end only |

**Examples:**

```text
{trim}           # Remove whitespace from both ends
{trim:left}      # Remove from start only
{trim:right}     # Remove from end only
{trim:xy}        # Remove 'x' and 'y' from ends
{trim:*-+}       # Remove *, -, + from ends
{trim:\t\n}      # Remove tabs and newlines
```

#### Pad

Adds padding characters to reach a specified width.

**Syntax:** `pad:WIDTH[:CHAR[:DIRECTION]]`

| Parameter | Default | Options |
|-----------|---------|---------|
| WIDTH | - | Number of total characters |
| CHAR | space | Any single character |
| DIRECTION | `right` | `left`, `right`, `both` |

**Examples:**

```text
{pad:10}             # Pad to 10 chars with spaces (right)
{pad:5:0:left}       # Pad to 5 chars with zeros (left)
{pad:8:*:both}       # Pad to 8 chars with * (center)
```

### 🔄 Case Operations

#### Upper

Converts text to uppercase.

**Syntax:** `upper`

#### Lower

Converts text to lowercase.

**Syntax:** `lower`

### 🔧 Text Processing

#### Append

Adds text to the end of each string.

**Syntax:** `append:TEXT`

#### Prepend

Adds text to the beginning of each string.

**Syntax:** `prepend:TEXT`

#### Replace

Performs regex-based find and replace using sed-like syntax.

**Syntax:** `replace:s/PATTERN/REPLACEMENT/FLAGS`

| Flag | Description |
|------|-------------|
| `g` | Replace all occurrences (global) |
| `i` | Case-insensitive matching |
| `m` | Multiline mode |
| `s` | Dot matches newlines |

**Examples:**

```text
{replace:s/hello/hi/}        # Replace first "hello" with "hi"
{replace:s/\d+/NUM/g}        # Replace all numbers with "NUM"
{replace:s/world/WORLD/gi}   # Case-insensitive global replace
{replace:s/(.+)/[$1]/}       # Wrap in brackets using capture group
```

#### Regex Extract

Extracts text matching a regex pattern.

**Syntax:** `regex_extract:PATTERN[:GROUP]`

| Parameter | Description |
|-----------|-------------|
| PATTERN | Regular expression |
| GROUP | Capture group number (0 = whole match) |

**Examples:**

```text
{regex_extract:\d+}          # Extract first number
{regex_extract:@(.+):1}      # Extract domain from email
{regex_extract:\w+}          # Extract first word
```

### 📝 List Operations

#### Sort

Sorts list items alphabetically.

**Syntax:** `sort[:DIRECTION]`

| Direction | Description |
|-----------|-------------|
| `asc` (default) | Ascending order |
| `desc` | Descending order |

#### Reverse

Reverses the order of list items (see `map` section for examples) or characters in a string

**Syntax:** `reverse`

#### Unique

Removes duplicate items from a list, preserving order.

**Syntax:** `unique`

### 🔍 Filtering Operations

#### Filter

Keeps only items matching a regex pattern.

**Syntax:** `filter:PATTERN`

**Examples:**

```text
{split:,:..|filter:\d+}      # Keep items containing numbers
{split:,:..|filter:^test}    # Keep items starting with "test"
{split:,:..|filter:\.txt$}   # Keep .txt files
```

#### Filter Not

Removes items matching a regex pattern.

**Syntax:** `filter_not:PATTERN`

**Examples:**

```text
{split:,:..|filter_not:^#}   # Remove comments (lines starting with #)
{split:,:..|filter_not:^$}   # Remove empty lines
```

### 🎨 Formatting Operations

#### Strip ANSI

Removes ANSI escape sequences (colors, formatting) from text.

**Syntax:** `strip_ansi`

## 🎯 Range Specifications

Ranges are used with `split`, `substring`, `slice` operations.

### Syntax Summary

| Syntax | Description | Example |
|--------|-------------|---------|
| `N` | Single index | `5` (6th item, 0-indexed) |
| `N..M` | Range exclusive | `1..3` (items 1,2) |
| `N..=M` | Range inclusive | `1..=3` (items 1,2,3) |
| `N..` | From N to end | `2..` (from 3rd item) |
| `..M` | From start to M-1 | `..3` (first 3 items) |
| `..=M` | From start to M | `..=2` (first 3 items) |
| `..` | All items | `..` (everything) |

### Negative Indexing

Negative numbers count from the end:

| Index | Position |
|-------|----------|
| `-1` | Last item |
| `-2` | Second to last |
| `-3` | Third to last |

**Examples:**

```text
{split:,:-1}               # Last item after split
{substring:-3..}           # Last 3 characters
{split: :..|slice:-2..-1}  # Second to last item
```

## 🔤 Escaping Rules

### When is Escaping Required?

Different argument types have different escaping requirements:

#### Simple Arguments (append, prepend, join, etc.)

| Character | Escape | Reason                |
|-----------|--------|----------------------|
| `:`       | `\:`   | Separates arguments  |
| `\|`       | `\\|`   | Separates operations |
| `}`       | `\}`   | Ends template        |
| `{`       | `\{`   | Starts template      |
| `\`       | `\\`   | Escape character     |

#### Regex Arguments (filter, regex_extract)

Regex patterns can contain most characters naturally.

#### Split Arguments

Split separators can contain most characters. Only escape:

| Character | Escape | Reason |
|-----------|--------|--------|
| `:` | `\:` | Visual helper |

### Special Sequences

| Sequence | Result | Description |
|----------|--------|-------------|
| `\n` | newline | Line break |
| `\t` | tab | Tab character |
| `\r` | carriage return | Windows line ending |
| `\\` | `\` | Literal backslash |

### Example

```text
{append:\:value}         # Append ":value"
{prepend:\|}             # Prepend "|"
{split:\:\::..|join:-}   # Split on "::" and join with "-"
{split::::..|join:-}     # Does the same but is much harder to read
{replace:s/\//\-/g}      # Replace "/" with "-"
{filter:\\.txt$}         # Filter .txt files
```

## 🗺️ Map Operations

Map operations apply a sequence of operations to each item in a list individually.

### Syntax

```text
map:{operation1|operation2|...}
```

### Supported Operations in Map

✅ **Allowed:**

- `upper`, `lower`
- `trim`
- `append`, `prepend`
- `substring`
- `replace`
- `regex_extract`
- `pad`
- `reverse`
- `strip_ansi`

❌ **Not Allowed:**

- `split`, `join`
- `sort`, `unique`
- `filter`, `filter_not`
- `slice`
- Nested `map`

### Examples

```text
# Convert each item to uppercase
{split:,:..|map:{upper}}

# Trim and add prefix to each item
{split:,:..|map:{trim|prepend:item_}}

# Extract numbers from each item
{split:,:..|map:{regex_extract:\d+}}

# Complex processing per item
{split:,:..|map:{trim|upper|append:!|pad:10: :both}}
```

## 🐛 Debug Mode

Enable detailed logging by adding `!` at the start of the template.

### Syntax

```text
{!operation1|operation2|...}
```

### Debug Output

Shows:

- Initial input value
- Each operation being applied
- Intermediate results
- Final output
- For map operations: per-item processing details

Regular output goes to `stdout` debug information goes to `stderr`

### Example

```bash
Input: "hello,world"
Template: "{!split:,:..|map:{upper}|join:-}"

Debug Output:
DEBUG: Initial value: Str("hello,world")
DEBUG: Applying operation 1: Split { sep: ",", range: Range(None, None, false) }
DEBUG: Result: List with 2 items:
DEBUG:   [0]: "hello"
DEBUG:   [1]: "world"
DEBUG: ---
DEBUG: Applying operation 2: Map { operations: [Upper] }
DEBUG: Map operation starting with 2 items
DEBUG: Map operations to apply: 1 steps
DEBUG:   Step 1: Upper
DEBUG: Processing item 1 of 2: "hello"
DEBUG:   Item 1/2 initial value: Str("hello")
DEBUG:   Item 1/2 applying step 1: Upper
DEBUG:   Item 1/2 step 1 result: String("HELLO")
DEBUG: Processing item 2 of 2: "world"
DEBUG:   Item 2/2 initial value: Str("world")
DEBUG:   Item 2/2 applying step 1: Upper
DEBUG:   Item 2/2 step 1 result: String("WORLD")
DEBUG: Map operation completed. Results:
DEBUG:   Item 1: "HELLO"
DEBUG:   Item 2: "WORLD"
DEBUG: Result: List with 2 items:
DEBUG:   [0]: "HELLO"
DEBUG:   [1]: "WORLD"
DEBUG: ---
DEBUG: Applying operation 3: Join { sep: "-" }
DEBUG: Result: String("HELLO-WORLD")
DEBUG: ---
HELLO-WORLD
```

## 💡 Examples

### 📄 Data Processing

#### CSV Column Extraction

```bash
# Extract and format names from CSV
Input: "John Doe,30,Engineer"
Template: "{split:,:0|upper}"
Output: "JOHN DOE"
```

#### Log Analysis

```bash
# Extract timestamps from log lines
Input: "2023-01-01 10:30:00 ERROR Failed to connect"
Template: "{regex_extract:\\d{4}-\\d{2}-\\d{2}}"
Output: "2023-01-01"
```

#### File Processing

```bash
# Get file extensions and convert to uppercase
Input: "file1.txt,image.png,doc.pdf"
Template: "{split:,:..|map:{regex_extract:\.\w+$|upper}}"
Output: ".TXT,.PNG,.PDF"
```

### 🔄 Text Transformation

#### Path Manipulation

```bash
# Convert Unix path to Windows path
Input: "/home/user/documents/file.txt"
Template: "{replace:s/\//\\\\/g}"
Output: "\\home\\user\\documents\\file.txt"
```

#### Case Conversion with Formatting

```bash
# Uppercase with underscores
Input: "hello world test"
Template: "{split: :..|map:{upper}|join:_}"
Output: "HELLO_WORLD_TEST"
```

#### Cleaning Messy Data

```bash
# Clean and format user input
Input: "  John123  ,  Jane456  ,  Bob789  "
Template: "{split:,:..|map:{trim|regex_extract:[A-Za-z]+|lower|prepend:user_}}"
Output: "user_john,user_jane,user_bob"
```

### 📋 List Processing

#### Filtering and Sorting

```bash
# Filter files and sort
Input: "readme.md,script.py,data.json,test.py,config.yaml"
Template: "{split:,:..|filter:\.py$|sort}"
Output: "script.py,test.py"
```

#### Deduplication

```bash
# Remove duplicates and sort
Input: "apple,banana,apple,cherry,banana"
Template: "{split:,:..|unique|sort}"
Output: "apple,banana,cherry"
```

#### Padding for Alignment

```bash
# Create aligned output
Input: "1,22,333"
Template: "{split:,:..|map:{pad:4:0:left}}"
Output: "0001,0022,0333"
```

### 🎨 Formatting

#### Creating Tables

```bash
# Format as table row
Input: "a,b,c"
Template: "{split:,:..|map:{pad:15: :both}|join:\||append:\||prepend:\|}"
Output: "|       a       |       b       |       c       |"
```

#### Adding Decorations

```bash
# Add bullets and formatting
Input: "First item,Second item,Third item"
Template: "{split:,:..|map:{prepend:• |append: ✓}}"
Output: "• First item ✓,• Second item ✓,• Third item ✓"
```

## ⚠️ Troubleshooting

### Common Errors

#### ❌ Parse Errors

**Problem:** `Parse error: Expected operation`

```bash
# Wrong:
{split:,|invalid_op}

# Correct:
{split:,:..|upper}
```

**Problem:** `Parse error: Expected '}'`

```bash
# Wrong:
{split:,:..

# Correct:
{split:,:..}
```

#### ❌ Operation Errors

**Problem:** `Operation can only be applied to lists`

```bash
# Wrong: Trying to join a string
Input: "hello"
Template: "{join:-}"

# Correct: Split first
Template: "{split: :..|join:-}"
```

**Problem:** `Invalid regex`

```bash
# Wrong: Unescaped regex metacharacters
{filter:[}

# Correct: Proper regex
{filter:\\[}
```

#### ❌ Range Errors

**Problem:** `Invalid range specification`

```bash
# Wrong:
{split:,:abc}

# Correct:
{split:,:1..3}
```

### Best Practices

#### ✅ Do's

1. **Use debug mode** when developing complex templates:

   ```bash
   {!split:,:..|map:{upper}|join:-}
   ```

2. **Start simple** and build complexity gradually:

   ```bash
   # Start with:
   {split:,:..}

   # Then add:
   {split:,:..|map:{upper}}

   # Finally:
   {split:,:..|map:{trim|upper}|join:-}
   ```

3. **Test ranges** with simple data first:

   ```bash
   # Test with: "a,b,c,d,e"
   {split:,:1..3}  # Should output: "b,c"
   ```

4. **Escape when in doubt**:

   ```bash
   {append:\:value}  # Safe
   ```

#### ❌ Don'ts

1. **Don't use map operations that work on lists:**

   ```bash
   # Wrong:
   {split:,:..|map:{sort}}

   # Correct:
   {split:,:..|sort}
   ```

2. **Don't forget to split before list operations:**

   ```bash
   # Wrong:
   "hello,world" -> {slice:1..}

   # Correct:
   "hello,world" -> {split:,:..|slice:1..}

   # Even better
   "hello,world" -> {split:,:1..}
   ```

3. **Don't over-escape in regex patterns:**

   ```bash
   # Usually okay:
   {filter:\.txt$}

   # Over-escaped:
   {filter:\\.txt$}  # This looks for literal backslash + .txt
   ```

### Performance Tips

1. **Filter early** in the pipeline to reduce data:

   ```bash
   # Good:
   {split:,:..|filter:important|map:{upper}|sort}

   # Less efficient:
   {split:,:..|map:{upper}|sort|filter:IMPORTANT}
   ```

2. **Use specific ranges** instead of processing everything:

   ```bash
   # Better:
   {split:,:0..10|map:{upper}}

   # Potentially slower with large input:
   {split:,:..|map:{upper}|slice:0..10}
   ```

3. **Combine operations** when possible:

   ```bash
   # More efficient:
   {split: :..|map:{trim|upper}}

   # Less efficient:
   {split: :..|map:{trim}|map:{upper}}
   ```

---

💡 **Need more help?** Try using debug mode (`{!...}`) to see exactly how your template is being processed!
