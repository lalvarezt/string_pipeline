# 📝 Template System Documentation

A powerful string processing template system with support for splitting, transforming, filtering, and joining operations.

## 📋 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [🏗️ Template Syntax](#️-template-syntax)
  - [Basic Structure](#basic-structure)
  - [Operation Chaining](#operation-chaining)
  - [List Rendering Behavior](#list-rendering-behavior)
- [📊 Operations Reference](#-operations-reference)
  - [🔪 Split](#-split) - Split text into parts
  - [🍰 Slice](#-slice) - Extract range of items
  - [🔗 Join](#-join) - Combine items with separator
  - [✂️ Substring](#️-substring) - Extract characters from string
  - [✨ Trim](#-trim) - Remove characters from ends
  - [📐 Pad](#-pad) - Add padding to reach width
  - [🔠 Upper](#-upper) - Convert to uppercase
  - [🔡 Lower](#-lower) - Convert to lowercase
  - [➡️ Append](#️-append) - Add text to end
  - [⬅️ Prepend](#️-prepend) - Add text to beginning
  - [⚡ Replace](#-replace) - Find and replace with regex
  - [🎯 Regex Extract](#-regex-extract) - Extract with regex pattern
  - [🗂️ Sort](#️-sort) - Sort items alphabetically
  - [🪞 Reverse](#-reverse) - Reverse order or characters
  - [⭐ Unique](#-unique) - Remove duplicates
  - [🧪 Filter](#-filter) - Keep items matching pattern
  - [🚫 Filter Not](#-filter-not) - Remove items matching pattern
  - [🧹 Strip ANSI](#-strip-ansi) - Remove color codes
- [🎯 Range Specifications](#-range-specifications)
  - [Syntax Summary](#syntax-summary)
  - [Negative Indexing](#negative-indexing)
  - [Edge Case Handling](#edge-case-handling)
- [🛡️ Escaping Rules](#️-escaping-rules)
  - [Simple Arguments](#simple-arguments-append-prepend-join-etc)
  - [Regex Arguments](#regex-arguments-filter-regex_extract)
  - [Split Arguments](#split-arguments)
  - [Special Sequences](#special-sequences)
- [🗺️ Map Operations](#️-map-operations)
- [🐛 Debug Mode](#-debug-mode)
- [💡 Examples](#-examples)
- [⚠️ Troubleshooting](#️-troubleshooting)
  - [Common Errors](#common-errors)
  - [Best Practices](#best-practices)
    - [✅ Do's](#-dos)
    - [❌ Don'ts](#-donts)
    - [Performance Tips](#performance-tips)

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

### List Rendering Behavior

When a template produces a list as the final result, the system automatically renders it as a string. The separator used for this automatic rendering is determined by the **last operation that used a separator** in the processing chain.

**Examples:**

```text
# Last separator was comma in split - list renders with commas
{split:,:..}                    # "a,b,c" → outputs: "a,b,c"

# Last separator was space in split - list renders with spaces
{split: :..}                    # "a b c" → outputs: "a b c"

# Explicit join overrides automatic behavior
{split:,:..|join:-}             # "a,b,c" → outputs: "a-b-c"

# Operations after split don't change the separator
{split:,:..|sort}               # "c,a,b" → outputs: "a,b,c" (comma separator preserved)
```

**Separator Change Example:**

```text
Input: "apple|banana|cherry"

{split:\|:..}                   # Split on | → outputs: "apple|banana|cherry"
{split:\|:..|split:a:..}        # Split on |, then on 'a' → outputs: "apple a banana a cherry"
```

In this example:

1. First `split:\|:..` uses `|` as separator
2. Second `split:a:..` uses `a` as separator (this becomes the **last separator**)
3. Final output uses `a` to join the list, not the original `|`

> 💡 **Note:** To have full control over the output format, always use an explicit `join` operation as the final step.
> 🐛 **Debug Tip:** Use [Debug Mode](#-debug-mode) (`{!...}`) to see exactly which separator is being tracked and used for final rendering. This helps identify when and how the separator changes during processing.

## 📊 Operations Reference

### 🔪 Split

Splits input into a list using a separator.

**Syntax:** `split:SEPARATOR:RANGE`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| SEPARATOR | string | ✅ | Character(s) to split on |
| RANGE     | range  | ❌ | Which parts to keep (default: all) |

> 💡 **Note:** For detailed range syntax and examples, see [🎯 Range Specifications](#-range-specifications).

**Behavior on Different Input Types:**

- **String:** Splits the string by the separator into a list of parts
- **List:** Splits each item in the list by the separator, then flattens all results into a single list

**Example of List Behavior:**

```bash
# First, create a list where each item contains commas
string-pipeline '{split: :..|map:{append:,data,more}}' 'hello world'
# Creates: ["hello,data,more", "world,data,more"]

# Then split each item by comma - this flattens all results
string-pipeline '{split: :..|map:{append:,data,more}|split:,:..|join:-}' 'hello world'
# Output: "hello-data-more-world-data-more"
```

**Examples:**

```text
{split:,:..}           # Split on comma, keep all
{split: :0..2}         # Split on space, keep items 0,1
{split: :0..=2}        # Split on space, keep items 0,1,2
{split:\n:-1}          # Split on newline, keep last item
```

### 🍰 Slice

Extracts a range of items from a list.

**Syntax:** `slice:RANGE`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| RANGE     | range  | ✅ | Which items to extract from the list |

> 💡 **Note:** For detailed range syntax and examples, see [🎯 Range Specifications](#-range-specifications).

**Examples:**

```text
{split:,:..|slice:0..2}   # Take items 0,1 after splitting
{split: :..|slice:-3..}   # Take last 3 items
```

### 🔗 Join

Combines list items into a single string with a separator.

**Syntax:** `join:SEPARATOR`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| SEPARATOR | string | ❌ | Character(s) to place between items (default: empty) |

**Behavior on Different Input Types:**

- **List:** Joins items with the separator in their current order (no sorting applied)
- **String:** Returns the string unchanged (treats as single-item list)

> 💡 **Note:** If you don't use `join`, lists are automatically rendered using the separator from the last operation that used one. See [List Rendering Behavior](#list-rendering-behavior) for details.

**Examples:**

```text
{split:,:..|join:-}       # Join with hyphen
{split: :..|join:\n}      # Join with newlines
{split:,:..|join:}        # Join with no separator
```

### ✂️ Substring

Extracts characters from a string using range notation.

**Syntax:** `substring:RANGE`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| RANGE     | range  | ✅ | Which characters to extract from the string |

> 💡 **Note:** For detailed range syntax and examples, see [🎯 Range Specifications](#-range-specifications).

**Examples:**

```text
{substring:0..2}     # Characters 0,1
{substring:-3..}     # Last 3 characters
{substring:..5}      # First 5 characters
{substring:2}        # Single character at index 2
```

### ✨ Trim

Removes specified characters from the beginning and end of strings.

**Syntax:** `trim[:CHARACTERS][:DIRECTION]`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| CHARACTERS | string | ❌ | Characters to remove (default: whitespace) |
| DIRECTION | enum | ❌ | Where to trim: `both`, `left`, `right` (default: both) |

**Whitespace Characters:** When no characters are specified, removes standard whitespace: spaces, tabs (`\t`), newlines (`\n`), and carriage returns (`\r`).

**Examples:**

```text
{trim}           # Remove whitespace from both ends
{trim:left}      # Remove from start only
{trim:right}     # Remove from end only
{trim:xy}        # Remove 'x' and 'y' from ends
{trim:*-+}       # Remove *, -, + from ends
{trim:\t\n}      # Remove tabs and newlines
```

### 📐 Pad

Adds padding characters to reach a specified width.

**Syntax:** `pad:WIDTH[:CHAR[:DIRECTION]]`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| WIDTH     | number | ✅ | Total width to pad to |
| CHAR      | string | ❌ | Character to use for padding (default: space) |
| DIRECTION | enum | ❌ | Padding direction: `left`, `right`, `both` (default: right) |

**Examples:**

```text
{pad:10}             # Pad to 10 chars with spaces (right)
{pad:5:0:left}       # Pad to 5 chars with zeros (left)
{pad:8:*:both}       # Pad to 8 chars with * (center)
```

### 🔠 Upper

Converts text to uppercase.

**Syntax:** `upper`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | - | - | No parameters required |

**Examples:**

```text
{upper}                      # "hello world" → "HELLO WORLD"
{split:,:..|map:{upper}}     # "a,b,c" → "A,B,C"
{split: :..|map:{upper}|join:_}  # "hello world" → "HELLO_WORLD"
```

### 🔡 Lower

Converts text to lowercase.

**Syntax:** `lower`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | - | - | No parameters required |

**Examples:**

```text
{lower}                      # "HELLO WORLD" → "hello world"
{split:,:..|map:{lower}}     # "A,B,C" → "a,b,c"
```

### ➡️ Append

Adds text to the end of each string.

**Syntax:** `append:TEXT`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| TEXT      | string | ✅ | Text to add to the end of each string |

**Examples:**

```text
{append:.txt}                    # "file" → "file.txt"
{split:,:..|map:{append:!}}      # "a,b,c" → "a!,b!,c!"
```

### ⬅️ Prepend

Adds text to the beginning of each string.

**Syntax:** `prepend:TEXT`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| TEXT      | string | ✅ | Text to add to the beginning of each string |

**Examples:**

```text
{prepend:/home/user/}            # "file.txt" → "/home/user/file.txt"
{split:,:..|map:{prepend:- }}    # "a,b,c" → "- a,- b,- c"
```

### ⚡ Replace

Performs regex-based find and replace using sed-like syntax.

**Syntax:** `replace:s/PATTERN/REPLACEMENT/FLAGS`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| PATTERN   | regex | ✅ | Regular expression to find |
| REPLACEMENT | string | ✅ | Text to replace matches with |
| FLAGS     | string | ❌ | Modifiers: `g` (global), `i` (case-insensitive), `m` (multiline), `s` (dot-all) |

**Performance Optimization:** For simple string patterns without regex metacharacters and without global flag, a fast string replacement is used instead of regex compilation.

**Examples:**

```text
{replace:s/hello/hi/}        # Replace first "hello" with "hi"
{replace:s/\d+/NUM/g}        # Replace all numbers with "NUM"
{replace:s/world/WORLD/gi}   # Case-insensitive global replace
{replace:s/(.+)/[$1]/}       # Wrap in brackets using capture group
```

### 🎯 Regex Extract

Extracts text matching a regex pattern.

**Syntax:** `regex_extract:PATTERN[:GROUP]`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| PATTERN   | regex | ✅ | Regular expression to match |
| GROUP     | number | ❌ | Capture group number (default: 0 = whole match) |

**No Match Behavior:** Returns empty string when pattern doesn't match.

**Examples:**

```text
{regex_extract:\d+}          # Extract first number
{regex_extract:@(.+):1}      # Extract domain from email, get 1st group
{regex_extract:\w+}          # Extract first word
```

### 🗂️ Sort

Sorts list items alphabetically.

**Syntax:** `sort[:DIRECTION]`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| DIRECTION | enum | ❌ | Sort order: `asc` (ascending, default), `desc` (descending) |

**Examples:**

```text
{split:,:..|sort}                # "c,a,b" → "a,b,c"
{split:,:..|sort:desc}           # "a,b,c" → "c,b,a"
{split:,:..|unique|sort}         # "c,a,b,a,c" → "a,b,c"
```

### 🪞 Reverse

Reverses the order of list items or characters in a string.

**Syntax:** `reverse`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | - | - | No parameters required |

**Behavior on Different Input Types:**

- **String:** Reverses character order
- **List:** Reverses item order

**Examples:**

```text
{reverse}                        # "hello" → "olleh"
{split:,:..|reverse}             # "a,b,c" → "c,b,a"
{split:,:..|map:{reverse}}       # "abc,def" → "cba,fed"
```

### ⭐ Unique

Removes duplicate items from a list, preserving order.

**Syntax:** `unique`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | - | - | No parameters required |

**Order Preservation:** The first occurrence of each item is kept, maintaining the original order.

**Examples:**

```text
{split:,:..|unique}              # "a,b,a,c,b" → "a,b,c"
{split: :..|unique|sort}         # "cat dog cat bird" → "bird cat dog"
{split:,:..|unique|join:-}       # "x,y,x,z,y" → "x-y-z"
{split:\n:..|unique}             # Remove duplicate lines
```

### 🧪 Filter

Keeps only items matching a regex pattern.

**Syntax:** `filter:PATTERN`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| PATTERN   | regex | ✅ | Regular expression to match items against |

**Behavior on Different Input Types:**

- **List:** Keeps items that match the pattern
- **String:** Returns the string if it matches, empty string otherwise

**Examples:**

```text
{split:,:..|filter:\d+}      # Keep items containing numbers
{split:,:..|filter:^test}    # Keep items starting with "test"
{split:,:..|filter:\.txt$}   # Keep .txt files
```

### 🚫 Filter Not

Removes items matching a regex pattern.

**Syntax:** `filter_not:PATTERN`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| PATTERN   | regex | ✅ | Regular expression to match items for removal |

**Behavior on Different Input Types:**

- **List:** Removes items that match the pattern
- **String:** Returns empty string if it matches, original string otherwise

**Examples:**

```text
{split:,:..|filter_not:^#}   # Remove comments (lines starting with #)
{split:,:..|filter_not:^$}   # Remove empty lines
```

### 🧹 Strip ANSI

Removes ANSI escape sequences (colors, formatting) from text.

**Syntax:** `strip_ansi`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | - | - | No parameters required |

**Sequence Types Removed:** Color codes, cursor movement, text formatting, and other ANSI escape sequences.

**Examples:**

```text
{strip_ansi}                     # "\e[31mRed Text\e[0m" → "Red Text"
{split:\n:..|map:{strip_ansi}}   # Clean colored log lines
```

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

### Edge Case Handling

The range system includes robust edge case handling:

**Out of Bounds:**

- **Single Index:** If index is beyond bounds, returns the last valid item
- **Range:** Automatically clamps to valid boundaries, returns empty if no valid range

**Empty Input:**

- Returns empty result for any range operation on empty input

**Invalid Ranges:**

- When start index >= end index, returns empty result
- Negative ranges are resolved relative to length before validation

**Examples:**

```text
{split:,:-1}               # Last item after split
{substring:-3..}           # Last 3 characters
{split: :..|slice:-2..-1}  # Second to last item
{substring:100}            # If string has 5 chars, returns last char
{split:,:..|slice:10..15}  # If list has 3 items, returns empty
```

## 🛡️ Escaping Rules

### When is Escaping Required?

Different argument types have different escaping requirements:

### Simple Arguments (append, prepend, join, etc.)

| Character | Escape | Reason                |
|-----------|--------|----------------------|
| `:`       | `\:`   | Separates arguments  |
| `\|`       | `\\|`   | Separates operations |
| `}`       | `\}`   | Ends template        |
| `{`       | `\{`   | Starts template      |
| `\`       | `\\`   | Escape character     |

### Regex Arguments (filter, regex_extract)

Regex patterns can contain most characters naturally.

### Split Arguments

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
| `\/` | `/` | Forward slash (for sed patterns) |
| `\\` | `\` | Literal backslash |

**Fallback Behavior:** Any escape sequence not listed above (`\X`) will result in the character `X` being inserted literally.

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

### CSV Column Extraction

```bash
# Extract and format names from CSV
string-pipeline '{split:,:0|map:{upper}}' 'John Doe,30,Engineer'
# Output: "JOHN DOE"
```

### Log Analysis

```bash
# Extract timestamps from log lines
string-pipeline '{regex_extract:\d{4}-\d{2}-\d{2}}' '2023-01-01 10:30:00 ERROR Failed to connect'
# Output: "2023-01-01"
```

### File Processing

```bash
# Get file extensions and convert to uppercase
string-pipeline '{split:,:..|map:{regex_extract:\.\w+$|upper}}' 'file1.txt,image.png,doc.pdf'
# Output: ".TXT,.PNG,.PDF"
```

### 🔄 Text Transformation

### Path Manipulation

```bash
# Convert Unix path to Windows path
string-pipeline '{replace:s/\//\\\\/g}' '/home/user/documents/file.txt'
# Output: "\\home\\user\\documents\\file.txt"
```

### Case Conversion with Formatting

```bash
# Uppercase with underscores
string-pipeline '{split: :..|map:{upper}}' 'hello world test'
# Output: "HELLO_WORLD_TEST"
```

### Cleaning Messy Data

```bash
# Clean and format user input
string-pipeline '{split:,:..|map:{trim|regex_extract:[A-Za-z]+|lower|prepend:user_}}' '  John123  ,  Jane456  ,  Bob789  '
# Output: "user_john,user_jane,user_bob"
```

### 📋 List Processing

### Filtering and Sorting

```bash
# Filter files and sort
string-pipeline '{split:,:..|filter:\.py$|sort}' 'readme.md,script.py,data.json,test.py,config.yaml'
# Output: "script.py,test.py"
```

### Deduplication

```bash
# Remove duplicates and sort
string-pipeline '{split:,:..|unique|sort}' 'apple,banana,apple,cherry,banana'
# Output: "apple,banana,cherry"
```

### Padding for Alignment

```bash
# Create aligned output
string-pipeline '{split:,:..|map:{pad:4:0:left}}' '1,22,333'
# Output: "0001,0022,0333"
```

### 🎨 Formatting

### Creating Tables

```bash
# Format as table row
string-pipeline '{split:,:..|map:{pad:15: :both}|join:\||append:\||prepend:\|}' 'a,b,c'
# Output: "|       a       |       b       |       c       |"
```

### Adding Decorations

```bash
# Add bullets and formatting
string-pipeline '{split:,:..|map:{prepend:• |append: ✓}}' 'First item,Second item,Third item'
# Output: "• First item ✓,• Second item ✓,• Third item ✓"
```

## ⚠️ Troubleshooting

### Common Errors

### Parse Errors

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

### Operation Errors

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

### Range Errors

**Problem:** `Invalid range specification`

```bash
# Wrong:
{split:,:abc}

# Correct:
{split:,:1..3}
```

### Best Practices

### ✅ Do's

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

### ❌ Don'ts

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
