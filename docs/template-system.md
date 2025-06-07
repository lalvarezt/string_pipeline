# 📝 Template System Documentation

A powerful string processing template system with support for splitting, transforming, filtering, and joining operations.

## 📋 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [🏗️ Template Syntax](#️-template-syntax)
  - [Basic Structure](#basic-structure)
  - [Operation Chaining](#operation-chaining)
  - [List Rendering Behavior](#list-rendering-behavior)
- [📊 Operations Reference](#-operations-reference)
  - [🎯 Operation Type System](#-operation-type-system) - Understanding input/output types
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
- [⚡ Performance Characteristics](#-performance-characteristics)

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

### 🎯 Operation Type System

Understanding how operations handle different input types is crucial for building effective templates. The String Pipeline system has a well-designed type system that ensures predictable behavior and clear error messages.

#### 📋 Complete Type Matrix

| Operation | Accepts String | Accepts List | Returns String | Returns List | Notes |
|-----------|----------------|--------------|----------------|--------------|-------|
| **Split** | ✅ | ✅ | ✅* | ✅* | *Single index → String, Range → List |
| **Join** | ✅ | ✅ | ✅ | ❌ | String input passes through unchanged |
| **Replace** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Upper** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Lower** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Trim** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Substring** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Append** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Prepend** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **StripAnsi** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Pad** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **RegexExtract** | ✅ | ❌ | ✅ | ❌ | String-only operation |
| **Filter** | ✅ | ✅ | ✅ | ✅ | Type-preserving operation |
| **FilterNot** | ✅ | ✅ | ✅ | ✅ | Type-preserving operation |
| **Reverse** | ✅ | ✅ | ✅ | ✅ | Type-preserving operation |
| **Slice** | ❌ | ✅ | ❌ | ✅ | List-only operation |
| **Sort** | ❌ | ✅ | ❌ | ✅ | List-only operation |
| **Unique** | ❌ | ✅ | ❌ | ✅ | List-only operation |
| **Map** | ❌ | ✅ | ❌ | ✅ | List-only operation |

#### 🏗️ Type Categories

**🔤 String-to-String Operations** (10 operations)
Work exclusively with strings, provide clear error messages when applied to lists:

- `replace`, `upper`, `lower`, `trim`, `substring`
- `append`, `prepend`, `strip_ansi`, `pad`, `regex_extract`

```text
# ✅ Correct usage
{upper}                    # "hello" → "HELLO"
{split:,:..|map:{upper}}   # "a,b,c" → "A,B,C"

# ❌ Will error with helpful message
{upper}  # Applied to list → "upper operation can only be applied to strings. Use map:{upper} for lists."
```

**📋 List-to-List Operations** (4 operations)
Work exclusively with lists, provide clear guidance for string inputs:

- `slice`, `sort`, `unique`, `map`

```text
# ✅ Correct usage
{split:,:..|sort}          # "c,a,b" → "a,b,c"
{split:,:..|map:{upper}}   # "a,b,c" → "A,B,C"

# ❌ Will error with helpful message
{sort}  # Applied to string → "sort operation can only be applied to lists. Use split first."
```

**🔄 Type-Preserving Operations** (3 operations)
Accept both types and maintain the input type:

- `filter`, `filter_not`, `reverse`

```text
# ✅ String input → String output
{filter:hello}             # "hello world" → "hello world" (matches)
{reverse}                  # "hello" → "olleh"

# ✅ List input → List output
{split:,:..|filter:^a}     # "apple,banana,cherry" → "apple"
{split:,:..|reverse}       # "a,b,c" → "c,b,a"
```

**🔀 Type-Converting Operations** (2 operations)
Can change types based on parameters:

- `split` - String/List → String (single index) or List (range)
- `join` - List → String (String passes through unchanged)

```text
# Split examples
{split:,:0}                # "a,b,c" → "a" (String)
{split:,:..}               # "a,b,c" → ["a","b","c"] (List)
{split:,:1..3}             # "a,b,c,d" → ["b","c"] (List)

# Join examples
{split:,:..|join:-}        # "a,b,c" → "a-b-c" (List → String)
{join:-}                   # "hello" → "hello" (String passthrough)
```

#### ✅ Design Principles

**🎯 Predictable Behavior**
Every operation has consistent, well-defined input/output behavior:

- **Clear Error Messages**: When operations receive wrong types, they provide helpful suggestions
- **Type Safety**: No unexpected type conversions or silent failures
- **Explicit Control**: Use `map` to apply string operations to lists explicitly

**🔗 Composability**
Operations chain naturally with predictable data flow:

```text
{split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}|sort|join: | }
```

1. `split` - String → List
2. `map` - List → List (applies string operations per item)
3. `filter` - List → List (preserves type)
4. `sort` - List → List
5. `join` - List → String

**🛡️ Error Prevention**
The type system helps prevent common mistakes:

```text
# ❌ This would error clearly
{split:,:..|upper}         # "Cannot apply upper to list"

# ✅ Correct approach is obvious
{split:,:..|map:{upper}}   # Apply upper to each item
```

#### 💡 Practical Guidelines

**🚀 When Building Templates:**

1. **Start with your data type** - String or List?
2. **Plan your transformations** - What type does each operation expect?
3. **Use Map for string operations on lists** - Explicit and clear
4. **Let the system guide you** - Error messages suggest corrections

**🔍 Type Flow Examples:**

```text
# 📊 Data processing pipeline
"john,jane,bob"           # String input
{split:,:..}              # → List ["john","jane","bob"]
{map:{upper}}             # → List ["JOHN","JANE","BOB"]
{filter:^J}               # → List ["JOHN","JANE"]
{sort}                    # → List ["JANE","JOHN"]
{join: and }              # → String "JANE and JOHN"

# 🧹 Text cleaning pipeline
"  hello world  "         # String input
{trim}                    # → String "hello world"
{split: :..}              # → List ["hello","world"]
{map:{upper}}             # → List ["HELLO","WORLD"]
{join:_}                  # → String "HELLO_WORLD"
```

> 💡 **Pro Tip:** Use [Debug Mode](#-debug-mode) (`{!...}`) to see exactly how types flow through your template. This is invaluable for understanding and troubleshooting complex transformations! For comprehensive debugging techniques, see the [🐛 Debug System Guide](debug-system.md).

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
- **List:** Splits each item in the list by the separator, then **flattens all results into a single list**

> 💡 **List Processing Detail:** When applied to a list, Split processes each item individually and combines all split results. For example: `["a,b", "c,d"]` with `split:,` becomes `["a", "b", "c", "d"]`.

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

**Unicode Handling:** Substring correctly handles both ASCII and Unicode strings. For ASCII strings, it uses byte-level operations for performance. For Unicode strings, it operates on character boundaries to preserve multi-byte characters.

**Examples:**

```text
{substring:0..2}     # Characters 0,1
{substring:-3..}     # Last 3 characters
{substring:..5}      # First 5 characters
{substring:2}        # Single character at index 2
{substring:0..1}     # "🔥hello" → "🔥" (Unicode safe)
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

**Performance Optimization:** For simple string patterns without regex metacharacters and without global flag, a fast string replacement is used instead of regex compilation. Additionally, if the pattern doesn't exist in the input string, the operation returns immediately without processing.

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

Map operations apply a sequence of operations to each item in a list individually, enabling powerful per-item transformations.

### 📖 Concept

The `map` operation takes a list and applies a sequence of operations to each item separately, then combines the results back into a list.

```text
# Basic concept
["item1", "item2", "item3"] → map:{operation} → [result1, result2, result3]
```

### 🔧 Syntax

```text
map:{operation1|operation2|...}
```

**Key Rules:**

- Can only be applied to lists (use `split` first for strings)
- Operations inside map are applied to each item individually
- Nested `map` operations are not allowed

### 🎯 Operation Categories

#### ✅ **String Operations**

Apply to each item individually (item treated as string):

- **🔤 Case:** `upper`, `lower`
- **✂️ Modify:** `trim`, `append`, `prepend`, `substring`, `pad`
- **🔍 Extract/Replace:** `replace`, `regex_extract`
- **🎨 Format:** `reverse`, `strip_ansi`

#### ✅ **List Operations**

Process each item's content as a sub-list:

- **🔪 Parse:** `split` - Split each item and flatten results
- **🔗 Combine:** `join` - Join sub-lists within each item
- **📏 Select:** `slice` - Extract ranges from each item's content
- **📊 Transform:** `sort`, `unique` - Process each item's sub-elements
- **🧪 Filter:** `filter`, `filter_not` - Filter each item's content

#### ❌ **Not Allowed**

- Nested `map` operations
- Operations that change the fundamental list structure in unexpected ways

### 💡 Basic Examples

#### 🔤 String Operations

```text
# Convert each item to uppercase
{split:,:..|map:{upper}}
# "hello,world" → "HELLO,WORLD"

# Trim and add prefix to each item
{split:,:..|map:{trim|prepend:• }}
# "  apple  , banana " → "• apple,• banana"

# Extract numbers from each item
{split:,:..|map:{regex_extract:\d+}}
# "item1,thing22,stuff333" → "1,22,333"

# Chain multiple string operations
{split:,:..|map:{trim|upper|append:!|pad:10: :left}}
# " hello , world " → "    HELLO!,    WORLD!"
```

#### 📋 List Operations

```text
# Split each item further and sort words
{split:,:..|map:{split: :..|sort|join:_}}
# "c a,b d" → "a_c,b_d"

# Filter words within each item
{split:,:..|map:{split: :..|filter:^[A-Z]|join: }}
# "apple Banana,cherry Date" → "Banana,Date"

# Process CSV-like nested data
{split:\n:..|map:{split:,:..|slice:1..3|join:-}}
# "name,age,city\njohn,30,nyc\njane,25,la" → "age-city,30-nyc,25-la"
```

### 🔄 Automatic String Conversion

**Critical Behavior:** When map operations produce lists without explicit `join`, the system automatically converts them to strings using intelligent separator inheritance.

#### 📋 How It Works

1. **🎯 Item Processing:** Each map item's result is auto-joined using the separator from the last split within that map item
2. **🔗 Final Assembly:** The final list is auto-joined using the separator from the last split in the main pipeline
3. **📏 Flexible Lengths:** Different length sublists are handled gracefully - each joins independently

#### 💡 Step-by-Step Example

```text
# Input: "hello world,foo bar,test orange"
# Template: {split:,:..|map:{split: :..|filter:o}}

# Step 1: Split by comma
["hello world", "foo bar", "test orange"]

# Step 2: Map processes each item
#   "hello world" → split: ["hello", "world"] → filter:o → ["hello", "world"] → auto-join: "hello world"
#   "foo bar"     → split: ["foo", "bar"]     → filter:o → ["foo"]           → auto-join: "foo"
#   "test orange" → split: ["test", "orange"] → filter:o → ["orange"]        → auto-join: "orange"

# Step 3: Final result
["hello world", "foo", "orange"] → auto-join with comma → "hello world,foo,orange"
```

#### 🎛️ Controlling Output Format

```text
# 🔄 Automatic behavior (inherits separators)
{split:,:..|map:{split: :..}}                    # → "hello world,foo bar"

# 🎯 Explicit inner join (custom word separator)
{split:,:..|map:{split: :..|join:-}}             # → "hello-world,foo-bar"
{split:,:..|map:{split: :..|join:}}              # → "helloworld,foobar"
{split:,:..|map:{split: :..|join: | }}           # → "hello | world,foo | bar"

# 🔗 Explicit outer join (custom item separator)
{split:,:..|map:{split: :..}|join: ; }           # → "hello world ; foo bar"

# 🎨 Both explicit (full control)
{split:,:..|map:{split: :..|join:-}|join: | }    # → "hello-world | foo-bar"
```

#### ✅ Design Benefits

- **🔄 No Data Loss:** Sublists of different lengths are preserved correctly
- **🎯 Intuitive:** Output format matches input format by default
- **🎛️ Explicit Control:** Override with explicit `join` when needed
- **📏 Separator Inheritance:** Maintains consistent formatting automatically
- **🔍 Predictable:** Debug mode shows exactly what's happening at each step

## 🐛 Debug Mode

Enable detailed logging by adding `!` at the start of the template.

> 🔍 **For comprehensive debugging documentation**, see the [🐛 Debug System Guide](debug-system.md) which covers advanced debugging techniques, performance analysis, error diagnosis, and real-world troubleshooting scenarios.

### Syntax

```text
{!operation1|operation2|...}
```

### Debug Output

Shows:

- **🎯 Initial input value** - Starting data and type
- **🔄 Each operation** - Step-by-step execution
- **📊 Intermediate results** - Data transformation at each step
- **⚡ Performance metrics** - Timing and memory usage
- **🗺️ Map operation details** - Per-item processing visualization
- **✅ Final output** - Complete result with type information

Regular output goes to `stdout`, debug information goes to `stderr`.

### Quick Example

```bash
Input: "hello,world"
Template: "{!split:,:..|map:{upper}|join:-}"

Debug Output:
DEBUG: ═══════════════════════════════════════════════
DEBUG: PIPELINE START: 3 operations to apply
DEBUG: Initial input: Str("hello,world")
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 1/3: Applying Split { sep: ",", range: Range(None, None, false) }
DEBUG: Input: Str("hello,world")
DEBUG: Result: List(2 items: ["hello", "world"])
DEBUG: Step completed in 548.4µs
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 2/3: Applying Map { operations: [Upper] }
DEBUG: MAP OPERATION: Processing 2 items
DEBUG: ┌─ Processing item 1 of 2 ─────────────
DEBUG: │  Input: "hello" → Output: "HELLO"
DEBUG: └────────────────────────────────────────────
DEBUG: ┌─ Processing item 2 of 2 ─────────────
DEBUG: │  Input: "world" → Output: "WORLD"
DEBUG: └────────────────────────────────────────────
DEBUG: Result: List(2 items: ["HELLO", "WORLD"])
DEBUG: Step completed in 20.0277ms
DEBUG: ───────────────────────────────────────────────
DEBUG: STEP 3/3: Applying Join { sep: "-" }
DEBUG: Result: String("HELLO-WORLD")
DEBUG: Total execution time: 23.0989ms
DEBUG: ═══════════════════════════════════════════════
HELLO-WORLD
```

> 💡 **Need more?** The [🐛 Debug System Guide](debug-system.md) provides detailed coverage of:
>
> - **Complex pipeline debugging** with map operations
> - **Performance analysis** and bottleneck identification
> - **Error debugging** with type mismatch diagnosis
> - **Advanced techniques** for production debugging
> - **Real-world examples** and optimization case studies

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

> 🐛 **For comprehensive debugging and troubleshooting**, see the [🔍 Debug System Guide](debug-system.md) which covers advanced error diagnosis, performance debugging, and real-world troubleshooting scenarios with detailed examples.

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

**Problem:** `Operation can only be applied to strings`

```bash
# Wrong: Trying to apply string operation to list
Input: "a,b,c"
Template: "{split:,:..|upper}"

# Correct: Use map for string operations on lists
Template: "{split:,:..|map:{upper}}"
```

> 💡 **Type System Reference:** See the [🎯 Operation Type System](#-operation-type-system) section for complete details on which operations accept which input types and their expected outputs.

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

1. **Understand the type system** before building complex templates:

   ```bash
   # Know what each operation accepts and returns
   # See the Operation Type System section for complete details
   ```

2. **Use debug mode** when developing complex templates:

   ```bash
   {!split:,:..|map:{upper}|join:-}
   ```

3. **Start simple** and build complexity gradually:

   ```bash
   # Start with:
   {split:,:..}

   # Then add:
   {split:,:..|map:{upper}}

   # Finally:
   {split:,:..|map:{trim|upper}|join:-}
   ```

4. **Test ranges** with simple data first:

   ```bash
   # Test with: "a,b,c,d,e"
   {split:,:1..3}  # Should output: "b,c"
   ```

5. **Escape when in doubt**:

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

4. **Use replace optimization**:

   ```bash
   # Faster for simple patterns:
   {replace:s/hello/hi/}  # No regex compilation

   # Slower for simple patterns:
   {replace:s/h.*o/hi/}   # Requires regex engine
   ```

---

💡 **Need more help?**

🔍 **Try debug mode** (`{!...}`) to see exactly how your template is being processed!

🐛 **For advanced debugging**, check out the [Debug System Guide](debug-system.md) for comprehensive troubleshooting techniques, performance analysis, and real-world debugging examples!

## ⚡ Performance Characteristics

Understanding operation performance helps optimize templates for production use and identifies potential bottlenecks in complex pipelines.

### 🏆 Performance Categories

**🚀 Ultra-Fast Operations (100-200ns)**

- `upper`, `lower`, `trim` - Basic string transformations with minimal overhead
- Perfect for high-frequency processing

**⚡ Fast Operations (500ns-5μs)**

- `reverse`, `append`, `prepend` - Simple transformations and concatenations
- `split`, `join`, `sort` - Core list operations with optimized algorithms

**🔄 Moderate Operations (5-20μs)**

- `unique`, `filter`, `slice` - Operations requiring data examination
- Good for medium-complexity workflows

**🔍 Complex Operations (20-100μs)**

- `replace`, `regex_extract` - Pattern matching with regex compilation
- `map` operations - Per-item processing overhead
- Consider caching compiled patterns for repeated use

### 📊 Real-World Performance Data

Based on simple benchmarks with 1000 iterations on test data:

| Operation Category | Average Time | Best Use Case |
|-------------------|--------------|---------------|
| **String Basics** | 130-150ns | Real-time processing, hot paths |
| **List Processing** | 3-6μs | Data pipelines, batch processing |
| **Map Operations** | 8-15μs | Item-by-item transformations |
| **Complex Patterns** | 50-100μs | Advanced text processing |

### 🎯 Optimization Strategies

**For High-Performance Applications:**

1. **Minimize Map Operations**

   ```bash
   # Faster: Single map with multiple operations
   {split:,:..|map:{trim|upper|append:!}}

   # Slower: Multiple separate map operations
   {split:,:..|map:{trim}|map:{upper}|map:{append:!}}
   ```

2. **Filter Early in Pipeline**

   ```bash
   # Efficient: Reduce data size early
   {split:,:..|filter:important|map:{complex_processing}}

   # Inefficient: Process all data then filter
   {split:,:..|map:{complex_processing}|filter:IMPORTANT}
   ```

3. **Use Specific Ranges**

   ```bash
   # Direct: Split with range (faster)
   {split:,:1..10}

   # Indirect: Split all then slice (slower for large inputs)
   {split:,:..|slice:1..10}
   ```

### 🔬 Testing Your Templates

Use the built-in benchmarking tool to measure your specific templates:

```bash
# Build the benchmark tool
cargo build --release --bin bench

# Run performance tests
./target/release/bench --iterations 1000

# Get machine-readable results
./target/release/bench --format json > performance.json
```

**Custom Template Testing:**

For performance-sensitive applications, consider creating custom tests for your specific templates and data patterns. The debug system also provides per-operation timing:

```bash
# Debug with timing information
string-pipeline '{!your_complex_template}' "your_data"
# Look for "Step completed in XXXμs" in debug output
```

### 🏭 Production Considerations

**Release Builds:** Always use `--release` builds in production - they're 3-10x faster than debug builds.

**Memory Usage:** Most operations have minimal memory overhead, but be aware that:

- `unique` maintains a hash set of seen values
- `sort` creates temporary collections
- Large `split` operations create many string allocations

**Scaling Patterns:**

- Operations scale linearly with input size
- Map operations scale with both input size and sub-template complexity
- Regex operations have compilation overhead but efficient matching

> 📊 **Performance Details:** See the [Performance Benchmarking Guide](benchmarking.md) for timing data, automation tips, and optimization ideas.
