//! # string_pipeline
//!
//! A flexible, template-driven string transformation pipeline for Rust.

mod pipeline;

pub use pipeline::apply_ops;
pub use pipeline::parse_template;
pub use pipeline::process;
pub use pipeline::{RangeSpec, StringOp, Value};
