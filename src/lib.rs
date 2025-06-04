//! # string_pipeline
//!
//! A flexible, template-driven string transformation pipeline for Rust.
//!
//! This library provides a way to define a sequence of string operations using a concise template syntax,
//! allowing for dynamic string manipulation based on user-defined templates.
//!
//! # Quick start
//! ```rust
//! use string_pipeline::Template;
//!
//! // Define a template with operations
//! let template = Template::parse("{split:,:0..2|join: and }").unwrap();
//!
//! // Format a string using the template
//! let result = template.format("a,b,c,d").unwrap();
//!
//! assert_eq!(result, "a and b");
//! ```
//!
//! A more in-depth view of the template syntax can be found in the [Template::parse](Template::parse) method documentation.
//!
//! # More examples
//! Get the second item in a comma-separated list:
//! ```rust
//! use string_pipeline::Template;
//!
//! let template = Template::parse("{split:,:1}").unwrap();
//!
//! let result = template.format("a,b,c").unwrap();
//!
//! assert_eq!(result, "b");
//! ```
//!
//! Replace all spaces with underscores and uppercase:
//! ```rust
//! use string_pipeline::Template;
//!
//! let template = Template::parse("{replace:s/ /_/g|upper}").unwrap();
//!
//! let result = template.format("foo bar baz").unwrap();
//!
//! assert_eq!(result, "FOO_BAR_BAZ");
//! ```
//!
//! Trim, split and append a suffix to each resulting item:
//! ```rust
//! use string_pipeline::Template;
//!
//! let template = Template::parse("{split:,:..|map:{trim|append:!}}").unwrap();
//!
//! let result = template.format(" a, b,c , d , e ").unwrap();
//!
//! assert_eq!(result, "a!,b!,c!,d!,e!");
//! ```
//!
//! Strip ANSI escape codes:
//! ```rust
//! use string_pipeline::Template;
//!
//! let template = Template::parse("{strip_ansi}").unwrap();
//!
//! let result = template.format("\x1b[31mHello\x1b[0m").unwrap();
//!
//! assert_eq!(result, "Hello");
//! ```

mod pipeline;

pub use pipeline::Template;
