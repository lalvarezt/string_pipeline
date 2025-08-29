# 🔗 String Pipeline - Command Line Interface

_NOTE: what follows has mostly been assembled using AI as an experiment and as a basis for further improvements._

A comprehensive CLI tool for powerful string transformations with flexible input/output options and advanced debugging capabilities.

## 📋 Table of Contents

- [🏗️ Basic Usage](#️-basic-usage)
- [📥 Input Sources](#-input-sources)
- [📝 Template Input](#-template-input)
- [📤 Output Control](#-output-control)
- [🐛 Debug & Validation](#-debug--validation)
- [📚 Help & Information](#-help--information)
- [🎯 Common Usage Patterns](#-common-usage-patterns)
- [🔄 Advanced Workflows](#-advanced-workflows)
- [⚠️ Error Handling](#️-error-handling)
- [💡 Best Practices](#-best-practices)
- [⚡ Performance & Benchmarking](#-performance--benchmarking)
- [🔧 Troubleshooting](#-troubleshooting)

## 🏗️ Basic Usage

### Command Structure

```bash
string-pipeline [OPTIONS] [TEMPLATE] [INPUT]
```

| Component | Required | Description |
|-----------|----------|-------------|
| `TEMPLATE` | ✅* | Template string to apply |
| `INPUT` | ❌ | Input string to process |
| `OPTIONS` | ❌ | Configuration flags and parameters |

> 💡 **Note:** `TEMPLATE` is optional only when using `--template-file` option.

### 🔄 Input Priority Order

The CLI processes input sources in this priority order:

1. **🎯 Command Line Argument** - `INPUT` parameter
2. **📁 File Input** - `--input-file` option
3. **⌨️ Standard Input** - Default fallback (pipes, redirects)

**Quick Examples:**

```bash
# 🎯 Direct arguments
string-pipeline '{upper}' 'hello world'
# Output: HELLO WORLD

# ⌨️ From stdin (pipe)
echo "hello world" | string-pipeline '{upper}'
# Output: HELLO WORLD

# 📁 From file
echo "hello world" > input.txt
string-pipeline '{upper}' -f input.txt
# Output: HELLO WORLD
```

## 📥 Input Sources

### 🌊 Standard Input (Stdin)

The most flexible input method - works with pipes, redirects, and interactive input.

| Method | Syntax | Description | Use Case |
|--------|--------|-------------|----------|
| **🔗 Pipe** | `command \| string-pipeline` | Chain with other tools | Data processing pipelines |
| **📂 Redirect** | `string-pipeline < file.txt` | Read from file | Batch file processing |
| **📝 Heredoc** | `string-pipeline <<< "text"` | Inline text input | Quick testing |
| **⌨️ Interactive** | `string-pipeline '{template}'` | Type input manually | Development/testing |

**Real-world Examples:**

```bash
# 🔗 Process command output
echo "apple banana cherry" | string-pipeline '{split: :..|map:{upper}}'

# 📂 Process log files
echo -e "ERROR: failed\nINFO: success" | string-pipeline '{split:\n:..|filter:ERROR}'

# 📝 Quick data transformation
string-pipeline '{split:,:..|map:{upper}|join:-}' <<< "apple,banana,cherry"
# Output: APPLE-BANANA-CHERRY

# ⌨️ Interactive development
string-pipeline '{split: :..|map:{upper}}'
# Type: hello world test
# Output: HELLO,WORLD,TEST
```

### 🎯 Command Line Arguments

Perfect for scripting and automation when input is known.

```bash
# ✨ Simple transformation
string-pipeline '{upper}' "hello world"

# 🔄 Complex processing
string-pipeline '{split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}|sort}' "  apple  , banana , cat ,  elephant  "
# Output: APPLE,BANANA,ELEPHANT
```

### 📁 File Input

Ideal for processing existing files and batch operations.

**Syntax:** `--input-file FILE` or `-f FILE`

```bash
# 📊 Process CSV data
echo "Name,Age,City\nJohn,30,NYC\nJane,25,LA" > data.csv
string-pipeline '{split:\n:..|slice:1..|map:{split:,:0|upper}}' -f data.csv
# Output: JOHN,JANE

# 📋 Process log files
echo "2023-01-01 ERROR Failed\n2023-01-02 INFO Success" > app.log
string-pipeline '{split:\n:..|filter:ERROR|map:{regex_extract:\d{4}-\d{2}-\d{2}}}' -f app.log
# Output: 2023-01-01

# 🧹 Clean data files
string-pipeline '{split:\n:..|map:{trim}|filter:^.+$|unique}' -f messy_data.txt
```

## 📝 Template Input

### 🎯 Inline Templates

Default method - template provided as command argument.

```bash
# 🔤 Basic operations
string-pipeline '{upper}' 'hello world'
string-pipeline '{split:,:..|join:-}' 'a,b,c'

# 🔧 Complex transformations
string-pipeline '{split: :..|map:{substring:0..1|upper}|join:}' 'hello world test'
# Output: HWT
```

### 📄 Template Files

Store complex templates in files for reuse and better organization.

**Syntax:** `--template-file FILE` or `-t FILE`

**Creating Template Files:**

```bash
# 📝 Create reusable templates
echo '{split:,:..|map:{trim|upper}|sort|join: | }' > format_list.template
echo '{regex_extract:\d{4}-\d{2}-\d{2}}' > extract_date.template
echo '{split:\n:..|filter:ERROR|map:{regex_extract:\d{2}:\d{2}:\d{2}}}' > extract_error_times.template
```

**Using Template Files:**

```bash
# 📊 Format data consistently
string-pipeline -t format_list.template 'apple, banana, cherry'
# Output: APPLE | BANANA | CHERRY

# 📅 Extract dates from logs
echo "2023-01-01 ERROR Failed" > app.log
string-pipeline -t extract_date.template -f app.log

# 🔄 Combine template and input files
echo "apple,banana,cherry" > data.txt
string-pipeline -t format_list.template -f data.txt
```

**Template File Benefits:**

| Benefit | Description | Example Use Case |
|---------|-------------|------------------|
| **♻️ Reusability** | Use same template with different inputs | Standard data formatting |
| **📖 Readability** | Complex templates are easier to read | Multi-step transformations |
| **🔧 Maintainability** | Update logic in one place | Production data processing |
| **📋 Documentation** | Self-documenting with comments | Team workflows |

## 📤 Output Control

### 🎨 Output Formats

Control how results are presented with the `--output` or `-o` option.

| Format | Description | Best For | Example |
|--------|-------------|----------|---------|
| **📄 `raw`** | Output as-is (default) | Single values, custom formatting | `A,B,C` |
| **📋 `lines`** | Split comma-separated into lines | Lists, easy reading | `A`<br/>`B`<br/>`C` |
| **🔗 `json`** | JSON format | API integration, structured data | `["A","B","C"]` |

**Format Examples:**

```bash
# 📄 Raw format (default)
string-pipeline '{split:,:..|map:{upper}}' 'a,b,c'
# Output: A,B,C

# 📋 Lines format - great for readability
string-pipeline -o lines '{split:,:..|map:{upper}}' 'a,b,c'
# Output:
# A
# B
# C

# 🔗 JSON format - perfect for APIs
string-pipeline -o json '{split:,:..|map:{upper}}' 'a,b,c'
# Output: ["A","B","C"]

# 🔗 JSON with single value
string-pipeline -o json '{upper}' 'hello'
# Output: "HELLO"
```

### 🤫 Quiet Mode

Suppress debug output and validation messages with `--quiet` or `-q`.

```bash
# 🔊 Normal debug mode (shows detailed step-by-step processing)
string-pipeline -d '{split:,:..|map:{upper}}' "hello,world"
# [Detailed debug output - see Debug System Guide for complete examples]
# HELLO,WORLD

# 🤫 Quiet debug mode (result only)
string-pipeline -d -q '{split:,:..|map:{upper}}' 'a,b,c'
# A,B,C

# 🤫 Quiet validation (silent success)
string-pipeline -q --validate '{split:,:..|upper}'
# (no output if template is valid)
```

## 🐛 Debug & Validation

> 🔍 **For comprehensive debugging coverage**, see the [🐛 Debug System Guide](debug-system.md) which provides in-depth documentation on advanced debugging techniques, performance analysis, error diagnosis, and real-world troubleshooting scenarios.

### 🔍 Debug Mode

Enable step-by-step processing visualization.

**Syntax:** `--debug` or `-d`

**What Debug Mode Shows:**

- 🎯 Initial input value
- 🔄 Each operation being applied
- 📊 Intermediate results after each step
- 🗺️ Detailed map operation processing
- ⏱️ Performance timing for each step
- 📊 Cache statistics
- ✅ Final output

**Debug Examples:**

```bash
# 🔍 Basic debugging
string-pipeline -d '{split:,:..|map:{upper}}' 'hello,world'
# [Shows detailed step-by-step processing - see Debug System Guide]
# HELLO,WORLD

# 🤫 Quiet debugging (result only)
string-pipeline -d -q '{split:,:..|map:{upper}}' 'hello,world'
# HELLO,WORLD
```

### ✅ Template Validation

Validate template syntax without processing data.

**Syntax:** `--validate`

```bash
# ✅ Valid template
string-pipeline --validate '{split:,:..|map:{upper}|join:-}'
# Template syntax is valid

# ❌ Invalid template
string-pipeline --validate '{split:,:..|invalid_op}'
# Error parsing template: Unknown operation: invalid_op

# 🤫 Quiet validation (scripting)
if string-pipeline -q --validate '{template}'; then
    echo "Template is valid"
else
    echo "Template has errors"
fi
```

### 🔄 Inline Debug vs CLI Debug

| Method | Syntax | When to Use |
|--------|--------|-------------|
| **🔍 Inline Debug** | `{!operations...}` | Template development, one-off debugging |
| **🛠️ CLI Debug** | `--debug` flag | Script debugging, systematic testing |
| **🤫 Quiet Debug** | `--debug --quiet` | Production debugging, clean output |

```bash
# 🔍 Inline debug (template syntax)
string-pipeline '{!split:,:..|map:{upper}}' 'a,b,c'

# 🛠️ CLI debug (flag)
string-pipeline --debug '{split:,:..|map:{upper}}' 'a,b,c'

# 🤫 Combined: inline debug with quiet flag
string-pipeline -q '{!split:,:..|map:{upper}}' 'a,b,c'
# A,B,C (debug info suppressed by -q)
```

## 📚 Help & Information

### 📖 Getting Help

| Option | Description | Example |
|--------|-------------|---------|
| `--help`, `-h` | Show command help | `string-pipeline --help` |
| `--version`, `-V` | Show version info | `string-pipeline --version` |
| `--list-operations` | List all available operations | `string-pipeline --list-operations` |
| `--syntax-help` | Show template syntax guide | `string-pipeline --syntax-help` |

**Information Examples:**

```bash
# 📖 Basic help
string-pipeline --help

# 🏷️ Version information
string-pipeline --version
# string-pipeline 0.12.0

# 📋 List all operations
string-pipeline --list-operations
# Available operations:
# split - Split text into parts using separator
# join - Combine list items with separator
# upper - Convert to uppercase
# lower - Convert to lowercase
# ... (all operations listed)

# 📝 Syntax guide
string-pipeline --syntax-help
# Template Syntax Guide:
# Basic structure: {operation1|operation2|...}
# Debug mode: {!operation1|operation2|...}
# ... (detailed syntax examples)
```

## 🎯 Common Usage Patterns

### 🛠️ Development Workflow

**1. 🧪 Template Development:**

```bash
# ✅ Start with validation
string-pipeline --validate '{split:,:..|map:{upper}}'

# 🔍 Add debugging
string-pipeline -d '{split:,:..|map:{upper}}' 'test,data'

# 🤫 Clean up output
string-pipeline -d -q '{split:,:..|map:{upper}}' 'test,data'

# 💾 Save successful template
echo '{split:,:..|map:{upper}|join:-}' > uppercase_list.template
```

**2. 📊 Data Processing Pipeline:**

```bash
# 🔄 Multi-step data processing
cat raw_data.csv | \
    string-pipeline '{split:\n:..|slice:1..|filter:^[^#]}' | \
    string-pipeline '{split:,:..|map:{trim}}' | \
    string-pipeline -o json '{split:\n:..|unique|sort}'
```

### 🏭 Production Usage

**1. 📋 Batch File Processing:**

```bash
# 🔄 Process multiple files
for file in data_*.txt; do
    echo "Processing $file..."
    string-pipeline -t transform.template -f "$file" > "processed_$file"
done
```

**2. 🔗 Integration with Other Tools:**

```bash
# 📊 Extract and analyze
grep "ERROR" app.log | \
    string-pipeline '{regex_extract:\d{4}-\d{2}-\d{2}}' | \
    sort | uniq -c | sort -nr

# 🔄 Data transformation pipeline
echo '{"items":["apple,banana","cherry,date"]}' | \
    jq -r '.items[]' | \
    string-pipeline '{split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}}' | \
    string-pipeline -o lines '{split:,:..|sort}'
```

## 🔄 Advanced Workflows

### 🧪 Template Testing

```bash
# 🎯 Create test data
echo "apple,banana,cherry,date" > test_data.txt

# 🔍 Test templates incrementally
string-pipeline '{split:,:..}' -f test_data.txt
string-pipeline '{split:,:..|map:{upper}}' -f test_data.txt
string-pipeline '{split:,:..|map:{upper}|sort}' -f test_data.txt
string-pipeline '{split:,:..|map:{upper}|sort|join: \| }' -f test_data.txt

# ✅ Final validation
string-pipeline --validate '{split:,:..|map:{upper}|sort|join: \| }'
```

### 🔧 Complex Data Processing

```bash
# 📋 Multi-format output generation
DATA="john.doe@company.com,jane.smith@example.org,bob.wilson@test.net"

# 📄 Raw format
string-pipeline '{split:,:..|map:{regex_extract:@(.+):1|upper}}' "$DATA"
# Output: COMPANY.COM,EXAMPLE.ORG,TEST.NET

# 📋 Lines format
string-pipeline -o lines '{split:,:..|map:{regex_extract:@(.+):1|upper}}' "$DATA"
# Output:
# COMPANY.COM
# EXAMPLE.ORG
# TEST.NET

# 🔗 JSON format for APIs
string-pipeline -o json '{split:,:..|map:{regex_extract:@(.+):1|upper}}' "$DATA"
# Output: ["COMPANY.COM","EXAMPLE.ORG","TEST.NET"]
```

### 🎨 Custom Formatting

```bash
# 📊 Create formatted reports
USERS="Alice,Bob,Charlie,Diana"

# 📋 Bullet list
string-pipeline '{split:,:..|map:{prepend:• |append: ✓}}' "$USERS"
# Output: • Alice ✓,• Bob ✓,• Charlie ✓,• Diana ✓

# 🔢 Numbered list
string-pipeline -o lines '{split:,:..|map:{prepend:1. }}' "$USERS" | \
    awk '{gsub(/1\./, NR"."); print}'
# Output:
# 1. Alice
# 2. Bob
# 3. Charlie
# 4. Diana

# 📊 Table format
string-pipeline '{split:,:..|map:{pad:15: :both}|join:\|}' "$USERS"
# Output: '     Alice     |      Bob      |   Charlie     |     Diana     '
```

## ⚠️ Error Handling

### 🚨 Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| **0** | ✅ Success | Operation completed successfully |
| **1** | ❌ Error | Template parsing, I/O, or processing error |

### 🔍 Common Error Types

#### 📝 Template Errors

```bash
# ❌ Invalid operation
string-pipeline '{invalid_op}' 'input'
# Error parsing template: Unknown operation: invalid_op

# ❌ Syntax error
string-pipeline '{split:,}' 'input'  # Missing range
# Error parsing template: Expected range specification after ':'

# ❌ Unclosed template
string-pipeline '{split:,:.. ' 'input'
# Error parsing template: Expected '}'
```

#### 📁 Input/Output Errors

```bash
# ❌ File not found
string-pipeline '{upper}' -f nonexistent.txt
# Error reading input file: Failed to read file 'nonexistent.txt': No such file or directory

# ❌ Template file missing
string-pipeline -t missing.template 'input'
# Error reading template file: Failed to read file 'missing.template': No such file or directory

# ❌ Input conflict
string-pipeline '{upper}' -f input.txt 'also_input'
# Error: Cannot specify both input argument and input file
```

#### 🔧 Processing Errors

```bash
# ❌ Invalid regex
string-pipeline '{filter:[}' 'input'
# Error: Invalid regex pattern: missing closing bracket

# ❌ Invalid range
string-pipeline '{split:,:abc}' 'input'
# Error: Invalid range specification: 'abc'

# ❌ Operation type mismatch
string-pipeline '{join:-}' 'plain_string'
# Error: join operation can only be applied to lists
```

### 🛡️ Error Prevention

**✅ Best Practices:**

```bash
# 1. ✅ Validate templates first
string-pipeline --validate '{template}' && \
string-pipeline '{template}' 'input'

# 2. 🔍 Use debug mode during development
string-pipeline -d '{template}' 'test_input'

# 3. 🧪 Test with simple data first
string-pipeline '{complex_template}' 'a,b,c'

# 4. 📁 Check file existence
[ -f input.txt ] && string-pipeline '{template}' -f input.txt

# 5. 🔄 Handle errors in scripts
if ! string-pipeline '{template}' 'input' > output.txt; then
    echo "Processing failed" >&2
    exit 1
fi
```

## 💡 Best Practices

### 🎯 Template Development

#### ✅ Do's

1. **🧪 Start Simple and Build Up:**

```bash
# ✅ Incremental development
string-pipeline '{split:,:..}' 'a,b,c'                    # Test split
string-pipeline '{split:,:..|map:{upper}}' 'a,b,c'        # Add transformation
string-pipeline '{split:,:..|map:{upper}|sort}' 'c,a,b'   # Add sorting
```

1. **🔍 Use Debug Mode Liberally:**

```bash
# ✅ Debug complex templates
string-pipeline -d '{split:,:..|map:{trim|upper}|filter:^[A-Z]{3,}}' '  apple  , hi , banana  '
```

1. **📁 Organize Complex Templates:**

```bash
# ✅ Save reusable templates
echo '{split:,:..|map:{trim|upper}|sort|unique}' > clean_sort_list.template
string-pipeline -t clean_sort_list.template 'data'
```

1. **✅ Validate Before Processing:**

```bash
# ✅ Safe template execution
string-pipeline --validate '{template}' && \
string-pipeline '{template}' -f large_file.txt
```

#### ❌ Don'ts

1. **❌ Don't Skip Validation:**

```bash
# ❌ Risk processing large data with broken template
string-pipeline '{broken_template}' -f huge_file.txt

# ✅ Validate first
string-pipeline --validate '{template}' && \
string-pipeline '{template}' -f huge_file.txt
```

1. **❌ Don't Ignore Debug Output:**

```bash
# ❌ Assuming template works without testing
string-pipeline '{complex_template}' 'production_data'

# ✅ Test and debug first
string-pipeline -d '{complex_template}' 'test_data'
```

### ⚡ Performance Optimization

1. **🎯 Filter Early:**

```bash
# ✅ Filter before expensive operations
'{split:,:..|filter:important|map:{complex_operation}}'

# ❌ Process everything then filter
'{split:,:..|map:{complex_operation}|filter:IMPORTANT}'
```

1. **📏 Use Specific Ranges:**

```bash
# ✅ Process only what you need
'{split:,:0..10|map:{upper}}'

# ❌ Process everything then slice
'{split:,:..|map:{upper}|slice:0..10}'
```

1. **🔄 Combine Operations:**

```bash
# ✅ Single map with multiple operations
'{split:,:..|map:{trim|upper|append:!}}'

# ❌ Multiple separate maps
'{split:,:..|map:{trim}|map:{upper}|map:{append:!}}'
```

### 🏭 Production Usage

1. **🔒 Error Handling:**

```bash
# ✅ Robust script with error handling
#!/bin/bash
set -euo pipefail

if ! string-pipeline --validate "${TEMPLATE}"; then
    echo "Invalid template" >&2
    exit 1
fi

if ! string-pipeline "${TEMPLATE}" -f "${INPUT_FILE}" > "${OUTPUT_FILE}"; then
    echo "Processing failed" >&2
    exit 1
fi
```

1. **📊 Logging and Monitoring:**

```bash
# ✅ Production processing with logging
{
    echo "Starting processing at $(date)"
    time string-pipeline -t transform.template -f data.txt
    echo "Processing completed at $(date)"
} 2>&1 | tee process.log
```

## ⚡ Performance & Benchmarking

String Pipeline includes built-in performance measurement tools accessible via the CLI.

### 🔬 Built-in Benchmarking Tool

Run comprehensive performance benchmarks:

```bash
# Build and run benchmarks
cargo build --release --bin bench
./target/release/bench

# Quick performance check
./target/release/bench --iterations 100

# JSON output for automation
./target/release/bench --format json > results.json
```

### 🔍 Real-Time Performance Monitoring

Use debug mode to see timing information for your specific templates:

```bash
# Get per-operation timing with debug mode
string-pipeline -d '{split:,:..|map:{upper}|sort}' 'your,data,here'
```

Debug output includes step-by-step timing:

```text
DEBUG: Step completed in 342.7µs
DEBUG: Total execution time: 18.7456ms
```

### 🚀 Quick Optimization Tips

**Template Performance Best Practices:**

```bash
# ✅ Filter early to reduce data
'{split:,:..|filter:important|map:{expensive_operation}}'

# ✅ Use direct ranges instead of slice
'{split:,:0..10}'

# ✅ Combine operations in single map
'{split:,:..|map:{trim|upper|append:!}}'
```

> 📊 **Comprehensive Guide:** For detailed benchmarking methodology, performance analysis, automation scripts, and optimization strategies, see the [🏆 Performance Benchmarking Guide](benchmarking.md).

## 🔧 Troubleshooting

### 🐛 Common Issues and Solutions

#### 🔍 "No Output" Problems

**Problem:** Template runs but produces no output.

```bash
# 🔍 Diagnose with debug mode
string-pipeline -d '{template}' 'input'

# 🔍 Check if input is being processed
string-pipeline -d '{upper}' 'test'  # Use a simple template first
```

**Common Causes:**

- Filter operations removing all items
- Range operations selecting empty ranges
- Input not matching expected format

**Solutions:**

```bash
# ✅ Step-by-step debugging
string-pipeline -d '{split:,:..}' 'input'        # Check split result
string-pipeline -d '{split:,:..|filter:pattern}' 'input'  # Check filter
```

#### 📁 File Processing Issues

**Problem:** File input not working as expected.

```bash
# 🔍 Verify file contents and encoding
file input.txt                    # Check file type
head -5 input.txt                 # Check first few lines
wc -l input.txt                   # Check line count

# 🔍 Test with simple template first
string-pipeline '{upper}' -f input.txt
```

#### 🔤 Character Encoding Problems

**Problem:** Special characters not displaying correctly.

```bash
# 🔍 Check file encoding
file -i input.txt

# 🔍 Convert if necessary
iconv -f ISO-8859-1 -t UTF-8 input.txt > input_utf8.txt
string-pipeline '{template}' -f input_utf8.txt
```

#### 🔧 Template Complexity Issues

**Problem:** Complex template not working as expected.

```bash
# ✅ Break down complex templates
# Instead of:
string-pipeline '{split:,:..|map:{trim|upper|filter:^[A-Z]{3,}}|sort|unique}'

# Do this:
string-pipeline '{split:,:..}' 'input'                    # Step 1
string-pipeline '{split:,:..|map:{trim}}' 'input'         # Step 2
string-pipeline '{split:,:..|map:{trim|upper}}' 'input'   # Step 3
# ... continue building step by step
```

### 🆘 Getting Help

1. **📖 Check Documentation:**
   - Use `--syntax-help` for template syntax
   - Use `--list-operations` for available operations
   - Review template examples in documentation

2. **🔍 Enable Debug Mode:**

   ```bash
   string-pipeline -d '{your_template}' 'test_data'
   ```

3. **🧪 Test with Simple Data:**

   ```bash
   # Test with predictable input
   string-pipeline '{your_template}' 'a,b,c'
   ```

4. **✅ Validate Templates:**

   ```bash
   string-pipeline --validate '{your_template}'
   ```

---

🎉 **You're now equipped to master the String Pipeline CLI!**

💡 **Pro Tip:** Combine the power of templates from the [📖 Template System Documentation](template-system.md) with these CLI features for maximum productivity!

🐛 **Debug Like a Pro:** Master the [🔍 Debug System Guide](debug-system.md) to troubleshoot complex pipelines and optimize performance!

🚀 **Ready to transform your data processing workflows? Start with simple examples and build up to complex transformations!**
