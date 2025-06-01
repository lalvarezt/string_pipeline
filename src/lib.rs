//! # string_pipeline
//!
//! A flexible, template-driven string transformation pipeline for Rust.
//!
//! ## Example
//!
//! ```rust
//! use string_pipeline::process;
//! let result = process("a,b,c", "{split:,:..:join:\\n}").unwrap();
//! assert_eq!(result, "a\nb\nc");
//! ```

mod pipeline;

pub use pipeline::apply_ops;
pub use pipeline::parse_template;
pub use pipeline::process;
pub use pipeline::{RangeSpec, StringOp, Value};
