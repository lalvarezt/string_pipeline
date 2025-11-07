//! Debug context for pipeline operations.
//!
//! This module contains the debug context implementation that provides
//! detailed logging and tracing capabilities for pipeline execution.

use crate::pipeline::{REGEX_CACHE, SPLIT_CACHE, StringOp, Value};
use std::time::Duration;

/// Debug tracer that provides hierarchical execution logging for pipeline operations.
///
/// The `DebugTracer` outputs detailed information about pipeline execution including
/// operation timing, input/output values, cache statistics, and hierarchical structure
/// visualization. It supports both main pipeline and sub-pipeline tracing with
/// appropriate indentation levels.
#[derive(Clone)]
pub struct DebugTracer {
    enabled: bool,
    is_sub_pipeline: bool,
}

impl DebugTracer {
    /// Creates a new debug tracer.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether debug output should be generated
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            is_sub_pipeline: false,
        }
    }

    /// Creates a debug tracer for sub-pipeline operations.
    ///
    /// Sub-pipeline tracers use deeper indentation levels and different
    /// visual markers to distinguish nested operations from main pipeline operations.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether debug output should be generated
    pub fn sub_pipeline(enabled: bool) -> Self {
        Self {
            enabled,
            is_sub_pipeline: true,
        }
    }

    /// Logs the start of a template processing session.
    ///
    /// This marks the beginning of a complete processing session, showing
    /// the session type, template string, input data, and optional additional information.
    ///
    /// # Arguments
    ///
    /// * `session_type` - Label for the session type in debug output (e.g., "MULTI-TEMPLATE")
    /// * `template` - The template string being processed
    /// * `input` - The input data to be processed
    /// * `info` - Optional additional information to display
    pub fn session_start(
        &self,
        session_type: &str,
        template: &str,
        input: &str,
        info: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }

        self.line(format!("📂 {session_type}"));
        self.line_with_prefix(format!("🏁 {session_type} START"), 1);
        self.line_with_prefix(format!("Template: {template:?}"), 1);
        self.line_with_prefix(format!("➡️ Input: {input:?}"), 1);
        if let Some(info) = info {
            self.line_with_prefix(info.to_string(), 1);
        }
        self.separator();
    }

    /// Logs the end of a template processing session with results and timing information.
    ///
    /// This marks the completion of a processing session, showing the final result,
    /// execution time, and cache statistics.
    ///
    /// # Arguments
    ///
    /// * `session_type` - Label for the session type in debug output (e.g., "MULTI-TEMPLATE")
    /// * `result` - The final processed result
    /// * `elapsed` - Total execution time for the session
    pub fn session_end(&self, session_type: &str, result: &str, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        self.line_with_prefix(format!("🏁 ✅ {session_type} COMPLETE"), 1);
        self.line_with_prefix(format!("🎯 Final result: {result:?}"), 1);
        self.line_with_prefix(format!("Total execution time: {elapsed:?}"), 1);

        self.line_with_ending_prefix(
            format!(
                "Cache stats: {} regex patterns, {} split operations cached",
                REGEX_CACHE.len(),
                SPLIT_CACHE.len()
            ),
            1,
        );
    }

    /// Logs the start of pipeline execution.
    ///
    /// This shows the beginning of a pipeline (main or sub-pipeline) with the
    /// operations to be executed and the initial input value.
    ///
    /// # Arguments
    ///
    /// * `ops` - The sequence of operations to be executed
    /// * `input` - The initial input value for the pipeline
    pub fn pipeline_start(&self, ops: &[StringOp], input: &Value) {
        if !self.enabled {
            return;
        }

        let depth = if self.is_sub_pipeline { 4 } else { 1 };
        let icon = if self.is_sub_pipeline { "🔧" } else { "🚀" };
        let label = if self.is_sub_pipeline {
            "SUB-PIPELINE"
        } else {
            "PIPELINE"
        };

        self.line_with_prefix(
            format!(
                "📂 {}",
                if self.is_sub_pipeline {
                    "Sub-Pipeline"
                } else {
                    "Main Pipeline"
                }
            ),
            depth,
        );
        self.line_with_prefix(
            format!("{} {} START: {} operations", icon, label, ops.len()),
            depth + 1,
        );
        self.line_with_prefix(
            format!("➡️ Input: {}", Self::format_value(input)),
            depth + 1,
        );

        if ops.len() > 1 {
            for (i, op) in ops.iter().enumerate() {
                self.line_with_prefix(
                    format!("{}. {}", i + 1, Self::format_operation(op)),
                    depth + 1,
                );
            }
        }
    }

    /// Logs the end of pipeline execution with results and timing.
    ///
    /// This shows the completion of a pipeline with the final result and execution time.
    ///
    /// # Arguments
    ///
    /// * `result` - The final result value from the pipeline
    /// * `elapsed` - Total execution time for the pipeline
    pub fn pipeline_end(&self, result: &Value, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        let depth = if self.is_sub_pipeline { 4 } else { 1 };
        let label = if self.is_sub_pipeline {
            "SUB-PIPELINE"
        } else {
            "PIPELINE"
        };

        self.line_with_prefix(format!("✅ {label} COMPLETE"), depth + 1);
        self.line_with_prefix(
            format!("🎯 Result: {}", Self::format_value(result)),
            depth + 1,
        );
        self.line_with_ending_prefix(format!("Time: {elapsed:?}"), depth + 1);

        if !self.is_sub_pipeline {
            self.separator();
        }
    }

    /// Logs an individual operation step with input, output, and timing.
    ///
    /// This provides detailed information about each operation in the pipeline,
    /// including the operation type, input value, result value, and execution time.
    ///
    /// # Arguments
    ///
    /// * `step` - The current step number (1-based)
    /// * `_total` - The total number of steps (currently unused)
    /// * `op` - The operation being executed
    /// * `input` - The input value for this operation
    /// * `result` - The result value from this operation
    /// * `elapsed` - Execution time for this operation
    pub fn operation_step(
        &self,
        step: usize,
        _total: usize,
        op: &StringOp,
        input: &Value,
        result: &Value,
        elapsed: Duration,
    ) {
        if !self.enabled {
            return;
        }

        let depth = if self.is_sub_pipeline { 5 } else { 2 };

        self.line_with_prefix(
            format!("⚙️ Step {}: {}", step, Self::format_operation_name(op)),
            depth,
        );
        self.line_with_prefix(
            format!("➡️ Input: {}", Self::format_value(input)),
            depth + 1,
        );
        self.line_with_prefix(
            format!("🎯 Result: {}", Self::format_value(result)),
            depth + 1,
        );
        self.line_with_ending_prefix(format!("Time: {elapsed:?}"), depth + 1);
    }

    /// Logs the start of processing a map operation item.
    ///
    /// This shows when a map operation begins processing an individual item,
    /// including the item index and input value.
    ///
    /// # Arguments
    ///
    /// * `item_idx` - The current item index (1-based)
    /// * `total_items` - The total number of items being processed
    /// * `input` - The input string for this item
    pub fn map_item_start(&self, item_idx: usize, total_items: usize, input: &str) {
        if !self.enabled {
            return;
        }

        self.line_with_prefix(format!("🗂️ Item {item_idx}/{total_items}"), 3);
        self.line_with_prefix(format!("➡️ Input: {input:?}"), 4);
    }

    /// Logs the end of processing a map operation item.
    ///
    /// This shows the completion of processing an individual item in a map operation,
    /// with either the successful result or error information.
    ///
    /// # Arguments
    ///
    /// * `output` - The result of processing the item, either success or error
    pub fn map_item_end(&self, output: Result<&str, &str>) {
        if !self.enabled {
            return;
        }

        match output {
            Ok(result) => self.line_with_ending_prefix(format!("Output: {result:?}"), 4),
            Err(error) => self.line_with_ending_prefix(format!("❌ ERROR: {error}"), 4),
        }
    }

    /// Logs the completion of a map operation with item counts.
    ///
    /// This shows the final statistics for a map operation, including how many
    /// items were processed and how many results were produced.
    ///
    /// # Arguments
    ///
    /// * `input_count` - Number of input items processed
    /// * `output_count` - Number of output items produced
    pub fn map_complete(&self, input_count: usize, output_count: usize) {
        if !self.enabled {
            return;
        }

        self.line_with_ending_prefix(
            format!("📦 MAP COMPLETED: {input_count} → {output_count} items"),
            3,
        );
    }

    /// Logs cache-related operations.
    ///
    /// This provides information about cache hits, misses, and other cache-related
    /// operations that occur during pipeline execution.
    ///
    /// # Arguments
    ///
    /// * `operation` - The type of cache operation (e.g., "HIT", "MISS", "STORE")
    /// * `details` - Additional details about the cache operation
    pub fn cache_operation(&self, operation: &str, details: &str) {
        if !self.enabled {
            return;
        }

        self.line_with_prefix(format!("💾 {operation} {details}"), 1);
        self.separator();
    }

    /// Logs section processing information for multi-template operations.
    ///
    /// This shows progress through different sections of a multi-template,
    /// including section type and content information.
    ///
    /// # Arguments
    ///
    /// * `section_num` - Current section number (1-based)
    /// * `total_sections` - Total number of sections
    /// * `section_type` - Type of section ("LITERAL" or "TEMPLATE")
    /// * `content` - Content or description of the section
    pub fn section(
        &self,
        section_num: usize,
        total_sections: usize,
        section_type: &str,
        content: &str,
    ) {
        if !self.enabled {
            return;
        }

        self.line_with_prefix(
            format!(
                "📊 SECTION {section_num}/{total_sections}: [{section_type}{}]",
                if content.is_empty() {
                    String::new()
                } else {
                    format!(": {content}")
                }
            ),
            1,
        );
    }

    // PRIVATE HELPERS

    /// Outputs a debug line without indentation prefix.
    fn line(&self, msg: String) {
        eprintln!("DEBUG: {msg}");
    }

    /// Outputs a debug line with hierarchical indentation prefix.
    ///
    /// Uses tree-like prefixes to show the hierarchical structure of operations.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to output
    /// * `depth` - The indentation depth level
    fn line_with_prefix(&self, msg: String, depth: usize) {
        let prefix = match depth {
            1 => "├── ".to_string(),
            2 => "│   ├── ".to_string(),
            3 => "│   │   ├── ".to_string(),
            4 => "│   │   │   ├── ".to_string(),
            5 => "│   │   │   │   ├── ".to_string(),
            6 => "│   │   │   │   │   ├── ".to_string(),
            _ => "│   ".repeat(depth.saturating_sub(1)) + "├── ",
        };
        eprintln!("DEBUG: {prefix}{msg}");
    }

    /// Outputs a debug line with ending hierarchical prefix.
    ///
    /// Uses terminal tree prefixes (`└──`) to indicate the end of a section.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to output
    /// * `depth` - The indentation depth level
    fn line_with_ending_prefix(&self, msg: String, depth: usize) {
        let prefix = match depth {
            1 => "└── ".to_string(),
            2 => "│   └── ".to_string(),
            3 => "│   │   └── ".to_string(),
            4 => "│   │   │   └── ".to_string(),
            5 => "│   │   │   │   └── ".to_string(),
            6 => "│   │   │   │   │   └── ".to_string(),
            _ => "│   ".repeat(depth.saturating_sub(1)) + "└── ",
        };
        eprintln!("DEBUG: {prefix}{msg}");
    }

    /// Outputs a visual separator line.
    pub fn separator(&self) {
        eprintln!("DEBUG: │");
    }

    /// Formats a value for display in debug output.
    ///
    /// Provides compact, readable representations of values with length limits
    /// for large strings and lists.
    ///
    /// # Arguments
    ///
    /// * `val` - The value to format
    ///
    /// # Returns
    ///
    /// A formatted string representation of the value
    fn format_value(val: &Value) -> String {
        match val {
            Value::Str(s) => {
                if s.len() > 40 {
                    format!("String({}..)", &s[..40])
                } else {
                    format!("String({s})")
                }
            }
            Value::List(list) => {
                if list.is_empty() {
                    "List(empty)".to_string()
                } else if list.len() <= 3 {
                    format!("List{list:?}")
                } else {
                    format!("List[{}, {}, ...+{}]", list[0], list[1], list.len() - 2)
                }
            }
        }
    }

    /// Formats a string operation for display with key parameters.
    ///
    /// Provides informative representations of operations including important
    /// parameters like separators and operation counts.
    fn format_operation(op: &StringOp) -> String {
        match op {
            StringOp::Split { sep, .. } => format!("Split('{sep}')"),
            StringOp::Join { sep } => format!("Join('{sep}')"),
            StringOp::Map { operations } => format!("Map({})", operations.len()),
            _ => Self::format_operation_name(op),
        }
    }

    /// Returns the simple name of a string operation without parameters.
    fn format_operation_name(op: &StringOp) -> String {
        match op {
            StringOp::Split { .. } => "Split".to_string(),
            StringOp::Join { .. } => "Join".to_string(),
            StringOp::Map { .. } => "Map".to_string(),
            StringOp::Upper => "Upper".to_string(),
            StringOp::Lower => "Lower".to_string(),
            StringOp::Trim { .. } => "Trim".to_string(),
            StringOp::Replace { .. } => "Replace".to_string(),
            StringOp::Filter { .. } => "Filter".to_string(),
            StringOp::FilterNot { .. } => "FilterNot".to_string(),
            StringOp::Sort { .. } => "Sort".to_string(),
            StringOp::Reverse => "Reverse".to_string(),
            StringOp::Unique => "Unique".to_string(),
            StringOp::Substring { .. } => "Substring".to_string(),
            StringOp::Append { .. } => "Append".to_string(),
            StringOp::Prepend { .. } => "Prepend".to_string(),
            StringOp::Surround { .. } => "Surround".to_string(),
            StringOp::Pad { .. } => "Pad".to_string(),
            StringOp::RegexExtract { .. } => "RegexExtract".to_string(),
            StringOp::Slice { .. } => "Slice".to_string(),
            StringOp::StripAnsi => "StripAnsi".to_string(),
        }
    }
}
