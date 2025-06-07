//! # string_pipeline
//!
//! A powerful string transformation CLI tool and Rust library that makes complex text processing simple.
//! Transform data using intuitive **template syntax** — chain operations like **split**, **join**, **replace**,
//! **filter**, and **20+ others** in a single readable expression.
//!
//! ## Features
//!
//! - **🔗 Chainable Operations**: Pipe operations together naturally
//! - **🎯 Precise Control**: Python-like ranges with Rust syntax (`-2..`, `1..=3`)
//! - **🗺️ Powerful Mapping**: Apply sub-pipelines to each list item
//! - **🔍 Regex Support**: sed-like patterns for complex transformations
//! - **🐛 Debug Mode**: Step-by-step operation visualization
//! - **📥 Flexible I/O**: CLI tool + embeddable Rust library
//! - **🦀 Performance optimized**: Zero-copy operations where possible, efficient memory usage
//! - **🌍 Unicode support**: Full UTF-8 and Unicode character handling
//! - **🛡️ Error handling**: Comprehensive error reporting for invalid operations
//!
//! ## Quick Start
//!
//! ```rust
//! use string_pipeline::Template;
//!
//! // Split by comma, take first 2 items, join with " and "
//! let template = Template::parse("{split:,:0..2|join: and }").unwrap();
//! let result = template.format("apple,banana,cherry,date").unwrap();
//! assert_eq!(result, "apple and banana");
//! ```
//!
//! ## Template Syntax Overview
//!
//! Templates are enclosed in `{}` and consist of operations separated by `|`:
//!
//! ```text
//! {operation1|operation2|operation3}
//! ```
//!
//! ### Core Operations (20+ Available)
//!
//! **🔪 Text Splitting & Joining**
//! - **`split:sep:range`** - Split text and optionally select range
//! - **`join:sep`** - Join list items with separator
//! - **`slice:range`** - Select list elements by range
//!
//! **✨ Text Transformation**
//! - **`upper`**, **`lower`** - Case conversion
//! - **`trim[:chars][:direction]`** - Remove whitespace or custom characters
//! - **`append:text`**, **`prepend:text`** - Add text to ends
//! - **`pad:width[:char][:direction]`** - Pad string to width
//! - **`substring:range`** - Extract characters from string
//!
//! **🔍 Pattern Matching & Replacement**
//! - **`replace:s/pattern/replacement/flags`** - Regex find/replace (sed-like)
//! - **`regex_extract:pattern[:group]`** - Extract with regex pattern
//! - **`filter:pattern`** - Keep items matching regex
//! - **`filter_not:pattern`** - Remove items matching regex
//!
//! **🗂️ List Processing**
//! - **`sort[:asc|desc]`** - Sort items alphabetically
//! - **`reverse`** - Reverse string or list order
//! - **`unique`** - Remove duplicate list items
//! - **`map:{operations}`** - Apply sub-pipeline to each list item
//!
//! **🧹 Utility Operations**
//! - **`strip_ansi`** - Remove ANSI escape sequences
//!
//! ### Range Syntax
//!
//! Supports Rust-like syntax with negative indexing:
//!
//! - **`N`** - Single index (`1` = second item)
//! - **`N..M`** - Range exclusive (`1..3` = items 1,2)
//! - **`N..=M`** - Range inclusive (`1..=3` = items 1,2,3)
//! - **`N..`** - From N to end
//! - **`..M`** - From start to M-1
//! - **`..`** - All items
//!
//! Negative indices count from end (`-1` = last item).
//!
//! ### Debug Mode
//!
//! Add `!` after opening `{` to enable detailed operation tracing:
//!
//! ```rust
//! use string_pipeline::Template;
//!
//! let template = Template::parse("{!split:,:..}").unwrap();
//! // Outputs debug information to stderr during processing
//! let result = template.format("a,b,c").unwrap();
//! assert_eq!(result, "a,b,c");
//! ```
//!
//! ## Common Use Cases
//!
//! ### Basic Text Processing
//! ```rust
//! use string_pipeline::Template;
//!
//! // Clean and normalize text
//! let cleaner = Template::parse("{trim|replace:s/\\s+/ /g|lower}").unwrap();
//! let result = cleaner.format("  Hello    WORLD  ").unwrap();
//! assert_eq!(result, "hello world");
//! ```
//!
//! ### Data Extraction
//! ```rust
//! use string_pipeline::Template;
//!
//! // Extract second field from space-separated data
//! let extractor = Template::parse("{split: :1}").unwrap();
//! let result = extractor.format("user 1234 active").unwrap();
//! assert_eq!(result, "1234");
//! ```
//!
//! ### List Processing with Map
//! ```rust
//! use string_pipeline::Template;
//!
//! // Process each item in a list
//! let processor = Template::parse("{split:,:..|map:{trim|upper}|join:\\|}").unwrap();
//! let result = processor.format(" apple, banana , cherry ").unwrap();
//! assert_eq!(result, "APPLE|BANANA|CHERRY");
//! ```
//!
//! ### Advanced Data Processing
//! ```rust
//! use string_pipeline::Template;
//!
//! // Extract domains from URLs
//! let domain_extractor = Template::parse("{split:,:..|map:{regex_extract://([^/]+):1|upper}}").unwrap();
//! let result = domain_extractor.format("https://github.com,https://google.com").unwrap();
//! assert_eq!(result, "GITHUB.COM,GOOGLE.COM");
//! ```
//!
//! ### Log Processing
//! ```rust
//! use string_pipeline::Template;
//!
//! // Extract timestamps from log entries
//! let log_parser = Template::parse(r"{split:\n:..|map:{regex_extract:\d\d\d\d-\d\d-\d\d}|filter_not:^$|join:\n}").unwrap();
//! let logs = "2023-12-01 ERROR: Failed\n2023-12-02 INFO: Success\nInvalid line";
//! let result = log_parser.format(logs).unwrap();
//! assert_eq!(result, "2023-12-01\n2023-12-02");
//! ```
//!
//! ### Filter Operations
//! ```rust
//! use string_pipeline::Template;
//!
//! // Filter files by extension
//! let py_filter = Template::parse("{split:,:..|filter:\\.py$|sort|join:\\n}").unwrap();
//! let files = "app.py,readme.md,test.py,data.json";
//! let result = py_filter.format(files).unwrap();
//! assert_eq!(result, "app.py\ntest.py");
//! ```
//!
//! ## Type System
//!
//! The pipeline system has a clear type system that distinguishes between:
//! - **String operations**: Work only on strings (e.g., `upper`, `lower`, `trim`, `replace`)
//! - **List operations**: Work only on lists (e.g., `sort`, `unique`, `slice`)
//! - **Type-preserving operations**: Accept both types (e.g., `filter`, `reverse`)
//! - **Type-converting operations**: Change between types (e.g., `split` converts string→list, `join` converts list→string)
//!
//! Use `map:{operation}` to apply string operations to each item in a list.
//!
//! ## Error Handling
//!
//! All operations return `Result<String, String>` for comprehensive error handling:
//!
//! ```rust
//! use string_pipeline::Template;
//!
//! // Invalid template syntax
//! let result = Template::parse("{split:}");
//! assert!(result.is_err());
//!
//! // Type mismatch errors are clear and helpful
//! let template = Template::parse("{sort}").unwrap();
//! let result = template.format("not_a_list");
//! assert!(result.is_err());
//! // Error: "sort operation can only be applied to lists. Use split first."
//! ```
//!
//! ## Performance Notes
//!
//! - Templates are compiled once and can be reused efficiently
//! - Operations use zero-copy techniques where possible
//! - Large datasets are processed with optimized algorithms
//! - Regex patterns are compiled and cached internally
//! - Memory allocation is minimized for common operations
//! - Early exit optimizations for string operations when patterns don't match
//!
//! For high-throughput applications, compile templates once and reuse them:
//!
//! ```rust
//! use string_pipeline::Template;
//!
//! // Compile once
//! let template = Template::parse("{split:,:0}").unwrap();
//!
//! // Reuse many times
//! for input in &["a,b,c", "x,y,z", "1,2,3"] {
//!     let result = template.format(input).unwrap();
//!     println!("{}", result);
//! }
//! ```
//!
//! For complete documentation including all operations, advanced features, and debugging techniques,
//! see the [`Template`] documentation and the comprehensive guides in the `docs/` directory.

mod pipeline;

pub use pipeline::Template;
