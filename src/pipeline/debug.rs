//! Debug context for pipeline operations.
//!
//! This module provides a comprehensive debugging and tracing system for pipeline execution,
//! with detailed logging, performance analysis, and data flow visualization.

use crate::pipeline::{REGEX_CACHE, RangeSpec, SPLIT_CACHE, StringOp, TrimDirection, Value};
use std::time::Instant;

/// Display mode for operation formatting
#[derive(Debug, Clone, Copy)]
pub enum OperationDisplayMode {
    /// Debug mode: Split(sep="," range=Range(...))
    Debug,
    /// Template mode: split:,:..
    Template,
}

/// Unified operation formatter
pub struct OperationFormatter;

impl OperationFormatter {
    /// Format an operation for display in the specified mode
    pub fn format_operation(op: &StringOp, mode: OperationDisplayMode) -> String {
        match mode {
            OperationDisplayMode::Debug => Self::format_debug(op),
            OperationDisplayMode::Template => Self::format_template(op),
        }
    }

    /// Format operation for debug output
    fn format_debug(op: &StringOp) -> String {
        match op {
            StringOp::Map { operations } => {
                if operations.len() <= 3 {
                    let ops_str: Vec<String> = operations.iter().map(Self::format_debug).collect();
                    format!("Map[{}]", ops_str.join(", "))
                } else {
                    format!("Map[{} ops]", operations.len())
                }
            }
            StringOp::Split { sep, range } => {
                format!("Split(sep={:?}, range={:?})", sep, range)
            }
            StringOp::Replace {
                pattern,
                replacement,
                flags,
            } => {
                format!("Replace(s/{}/{}/{})", pattern, replacement, flags)
            }
            StringOp::Filter { pattern } => format!("Filter({:?})", pattern),
            StringOp::FilterNot { pattern } => format!("FilterNot({:?})", pattern),
            StringOp::Trim { chars, direction } => {
                format!("Trim(chars={:?}, dir={:?})", chars, direction)
            }
            StringOp::Pad {
                width,
                char,
                direction,
            } => {
                format!("Pad(width={}, char={:?}, dir={:?})", width, char, direction)
            }
            StringOp::RegexExtract { pattern, group } => {
                format!("RegexExtract(pat={:?}, group={:?})", pattern, group)
            }
            StringOp::Substring { range } => format!("Substring({:?})", range),
            StringOp::Append { suffix } => format!("Append({:?})", suffix),
            StringOp::Prepend { prefix } => format!("Prepend({:?})", prefix),
            StringOp::Join { sep } => format!("Join({:?})", sep),
            StringOp::Sort { direction } => format!("Sort({:?})", direction),
            StringOp::Slice { range } => format!("Slice({:?})", range),
            op => format!("{:?}", op), // Fallback for simple operations
        }
    }

    /// Format operation for template syntax display
    fn format_template(op: &StringOp) -> String {
        match op {
            StringOp::Split { sep, range } => {
                if sep == " " {
                    // Shorthand syntax for space separator
                    Self::format_range_spec(range)
                } else {
                    format!("split:{}:{}", sep, Self::format_range_spec(range))
                }
            }
            StringOp::Upper => "upper".to_string(),
            StringOp::Lower => "lower".to_string(),
            StringOp::Append { suffix } => format!("append:{}", suffix),
            StringOp::Prepend { prefix } => format!("prepend:{}", prefix),
            StringOp::Replace {
                pattern,
                replacement,
                flags,
            } => {
                if flags.is_empty() {
                    format!("replace:s/{}/{}/", pattern, replacement)
                } else {
                    format!("replace:s/{}/{}/{}", pattern, replacement, flags)
                }
            }
            StringOp::Join { sep } => format!("join:{}", sep),
            StringOp::Map { operations } => {
                let inner_ops = operations
                    .iter()
                    .map(Self::format_compact)
                    .collect::<Vec<_>>()
                    .join("|");
                format!("map:{{{}}}", inner_ops)
            }
            StringOp::Trim { chars, direction } => Self::format_trim_operation(chars, direction),
            _ => format!("{:?}", op).to_lowercase(),
        }
    }

    /// Format operation for compact display (used in maps)
    fn format_compact(op: &StringOp) -> String {
        match op {
            StringOp::Upper => "upper".to_string(),
            StringOp::Lower => "lower".to_string(),
            StringOp::Trim { chars, direction } => Self::format_trim_operation(chars, direction),
            _ => format!("{:?}", op).to_lowercase(),
        }
    }

    /// Format a range specification for display
    fn format_range_spec(range: &RangeSpec) -> String {
        match range {
            RangeSpec::Index(i) => i.to_string(),
            RangeSpec::Range(start, end, inclusive) => match (start, end) {
                (None, None) => "..".to_string(),
                (Some(s), None) => format!("{}...", s),
                (None, Some(e)) => {
                    if *inclusive {
                        format!("..={}", e)
                    } else {
                        format!("..{}", e)
                    }
                }
                (Some(s), Some(e)) => {
                    let op = if *inclusive { "..=" } else { ".." };
                    format!("{}{}{}", s, op, e)
                }
            },
        }
    }

    /// Format a trim operation
    fn format_trim_operation(chars: &str, direction: &TrimDirection) -> String {
        let direction_str = match direction {
            TrimDirection::Both => "",
            TrimDirection::Left => ":left",
            TrimDirection::Right => ":right",
        };

        if chars.is_empty() {
            format!("trim{}", direction_str)
        } else if direction_str.is_empty() {
            format!("trim:{}", chars)
        } else {
            format!("trim:{}{}", chars, direction_str)
        }
    }

    /// Format multiple operations as a summary
    pub fn format_operations_summary(ops: &[StringOp], mode: OperationDisplayMode) -> String {
        ops.iter()
            .map(|op| Self::format_operation(op, mode))
            .collect::<Vec<_>>()
            .join("|")
    }
}

/// Utility for building debug prefixes consistently
pub struct DebugPrefix;

impl DebugPrefix {
    /// Build debug prefix for the specified indent level
    pub fn build(indent_level: usize) -> String {
        if indent_level == 0 {
            "DEBUG: ".to_string()
        } else {
            format!("DEBUG: {}", "  ".repeat(indent_level))
        }
    }

    /// Build debug prefix for child level (one level deeper)
    pub fn build_child(indent_level: usize) -> String {
        format!("DEBUG: {}", "  ".repeat(indent_level + 1))
    }
}

/// Context for pipeline execution debugging
#[derive(Debug, Clone)]
pub enum DebugContext {
    /// Single template execution
    SingleTemplate,
    /// Sub-pipeline execution (from map operations)
    SubPipeline { item_num: usize, total_items: usize },
}

/// A session-scoped debug tracker that manages the entire debug lifecycle
/// for a template execution, providing centralized timing, caching, and output.
pub struct DebugSession {
    enabled: bool,
    template_raw: String,
    session_type: SessionType,
    start_time: Instant,
    step_timings: Vec<StepTiming>,
    cache_stats_start: CacheStats,
    indent_level: usize,
}

/// Type of debug session being tracked
#[derive(Debug, Clone)]
pub enum SessionType {
    SingleTemplate,
    MultiTemplate { sections_count: usize },
    SubPipeline { item_num: usize, total_items: usize },
}

/// Timing information for individual pipeline steps
#[derive(Debug)]
struct StepTiming {
    step_num: usize,
    operation: String,
    duration: std::time::Duration,
}

/// Cache statistics snapshot
#[derive(Debug, Clone)]
struct CacheStats {
    regex_count: usize,
    split_count: usize,
}

impl CacheStats {
    fn current() -> Self {
        let regex_cache = REGEX_CACHE.lock().unwrap();
        let split_cache = SPLIT_CACHE.lock().unwrap();
        Self {
            regex_count: regex_cache.len(),
            split_count: split_cache.len(),
        }
    }

    fn diff(&self, other: &CacheStats) -> (usize, usize) {
        (
            other.regex_count.saturating_sub(self.regex_count),
            other.split_count.saturating_sub(self.split_count),
        )
    }
}

impl DebugSession {
    /// Create a new debug session for a single template
    pub fn new_single_template(enabled: bool, template_raw: String) -> Self {
        Self {
            enabled,
            template_raw,
            session_type: SessionType::SingleTemplate,
            start_time: Instant::now(),
            step_timings: Vec::new(),
            cache_stats_start: CacheStats::current(),
            indent_level: 0,
        }
    }

    /// Create a new debug session for a multi-template
    pub fn new_multi_template(enabled: bool, template_raw: String, sections_count: usize) -> Self {
        Self {
            enabled,
            template_raw,
            session_type: SessionType::MultiTemplate { sections_count },
            start_time: Instant::now(),
            step_timings: Vec::new(),
            cache_stats_start: CacheStats::current(),
            indent_level: 0,
        }
    }

    /// Create a new debug session for a sub-pipeline (map operation)
    pub fn new_sub_pipeline(enabled: bool, item_num: usize, total_items: usize) -> Self {
        Self {
            enabled,
            template_raw: String::new(),
            session_type: SessionType::SubPipeline {
                item_num,
                total_items,
            },
            start_time: Instant::now(),
            step_timings: Vec::new(),
            cache_stats_start: CacheStats::current(),
            indent_level: 1, // Sub-pipelines start at indent level 1
        }
    }

    /// Start the debug session with header information
    pub fn start(&self, input: &str, ops: &[StringOp]) -> Option<PipelineDebugger> {
        if !self.enabled {
            return None;
        }

        self.print_session_header(input, ops);
        Some(PipelineDebugger::new(self.enabled, self.indent_level))
    }

    /// Print the session header with context information
    fn print_session_header(&self, input: &str, ops: &[StringOp]) {
        if !self.enabled {
            return;
        }

        match &self.session_type {
            SessionType::SingleTemplate => {
                // Only show the separator for top-level single templates
                if self.indent_level == 0 {
                    eprintln!("DEBUG: ═══════════════════════════════════════════════");
                }
                eprintln!("{}SINGLE TEMPLATE START", self.build_debug_prefix());
                if !self.template_raw.is_empty() {
                    eprintln!(
                        "{}Template: {:?}",
                        self.build_child_debug_prefix(),
                        self.template_raw
                    );
                }
            }
            SessionType::MultiTemplate { sections_count } => {
                eprintln!("DEBUG: ═══════════════════════════════════════════════");
                eprintln!("{}MULTI-TEMPLATE START", self.build_debug_prefix());
                eprintln!(
                    "{}Template: {:?}",
                    self.build_child_debug_prefix(),
                    self.template_raw
                );
                eprintln!(
                    "{}Sections: {}",
                    self.build_child_debug_prefix(),
                    sections_count
                );
            }
            SessionType::SubPipeline {
                item_num,
                total_items,
            } => {
                eprintln!("{}🔧 SUB-PIPELINE START", self.build_debug_prefix());
                eprintln!(
                    "{}Processing item {} of {}",
                    self.build_child_debug_prefix(),
                    item_num,
                    total_items
                );
            }
        }

        if !matches!(self.session_type, SessionType::MultiTemplate { .. }) {
            eprintln!(
                "{}Input: {}",
                self.build_child_debug_prefix(),
                Self::format_value_for_display(&Value::Str(input.to_string()))
            );
            eprintln!(
                "{}Operations: {} to apply",
                self.build_child_debug_prefix(),
                ops.len()
            );
        }
    }

    /// End the debug session with summary information
    pub fn end(&mut self, final_result: &Value) {
        if !self.enabled {
            return;
        }

        let total_duration = self.start_time.elapsed();
        let cache_stats_end = CacheStats::current();
        let (new_regex, new_splits) = self.cache_stats_start.diff(&cache_stats_end);

        // Print completion header
        match &self.session_type {
            SessionType::SingleTemplate => {
                eprintln!("{}✅ SINGLE TEMPLATE COMPLETE", self.build_debug_prefix());
            }
            SessionType::MultiTemplate { .. } => {
                eprintln!("{}✅ MULTI-TEMPLATE COMPLETE", self.build_debug_prefix());
            }
            SessionType::SubPipeline { .. } => {
                eprintln!("{}✅ SUB-PIPELINE COMPLETE", self.build_debug_prefix());
            }
        }

        // Print final result
        eprintln!(
            "{}🎯 Final result: {}",
            self.build_child_debug_prefix(),
            Self::format_value_for_display(final_result)
        );
        eprintln!(
            "{}Total execution time: {:?}",
            self.build_child_debug_prefix(),
            total_duration
        );

        // Print performance summary if there were multiple steps
        if self.step_timings.len() > 1 {
            eprintln!("{}⏱️  Step timings:", self.build_child_debug_prefix());
            for timing in &self.step_timings {
                eprintln!(
                    "{}  Step {}: {} ({:?})",
                    self.build_child_debug_prefix(),
                    timing.step_num,
                    timing.operation,
                    timing.duration
                );
            }
        }

        // Print cache statistics only for main sessions
        if matches!(
            self.session_type,
            SessionType::SingleTemplate | SessionType::MultiTemplate { .. }
        ) && self.indent_level == 0
        {
            if new_regex > 0 || new_splits > 0 {
                eprintln!(
                    "{}📊 Cache activity: +{} regex patterns, +{} split operations",
                    self.build_child_debug_prefix(),
                    new_regex,
                    new_splits
                );
            }
            eprintln!(
                "{}📊 Total cache: {} regex patterns, {} split operations",
                self.build_child_debug_prefix(),
                cache_stats_end.regex_count,
                cache_stats_end.split_count
            );
        }

        if self.indent_level == 0 {
            eprintln!("DEBUG: ═══════════════════════════════════════════════");
        }
    }

    /// Record timing for a completed step
    pub fn record_step_timing(
        &mut self,
        step_num: usize,
        operation: &StringOp,
        duration: std::time::Duration,
    ) {
        if !self.enabled {
            return;
        }

        self.step_timings.push(StepTiming {
            step_num,
            operation: OperationFormatter::format_operation(operation, OperationDisplayMode::Debug),
            duration,
        });
    }

    /// Format a value for display (consistent with documentation)
    fn format_value_for_display(val: &Value) -> String {
        match val {
            Value::Str(s) => {
                if s.len() > 50 {
                    format!("String(len={}, preview={:?}...)", s.len(), &s[..30])
                } else {
                    format!("String({:?})", s)
                }
            }
            Value::List(list) => {
                if list.is_empty() {
                    "List(empty)".to_string()
                } else if list.len() == 1 {
                    format!("List(1 item: {:?})", list[0])
                } else {
                    format!(
                        "List({} items: [\n{}\n{}])",
                        list.len(),
                        list.iter()
                            .map(|item| format!("DEBUG:     {:?}", item))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        "DEBUG:   "
                    )
                }
            }
        }
    }

    /// Print section information (for multi-templates)
    pub fn print_section(
        &self,
        section_num: usize,
        total_sections: usize,
        section_type: &str,
        content: &str,
        _is_last: bool,
    ) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "{}SECTION {}/{}: [{}{}]",
            self.build_debug_prefix(),
            section_num,
            total_sections,
            section_type,
            if content.is_empty() {
                String::new()
            } else {
                format!(": {}", content)
            }
        );
    }

    /// Build debug prefix for current level
    fn build_debug_prefix(&self) -> String {
        DebugPrefix::build(self.indent_level)
    }

    /// Build debug prefix for child level (one level deeper)
    fn build_child_debug_prefix(&self) -> String {
        DebugPrefix::build_child(self.indent_level)
    }

    /// Get the session type (for MultiTemplateDebugger)
    pub fn session_type(&self) -> &SessionType {
        &self.session_type
    }

    /// Get the template raw string (for MultiTemplateDebugger)
    pub fn template_raw(&self) -> &str {
        &self.template_raw
    }

    /// Print cache operation information
    pub fn print_cache_operation(&self, operation_type: &str, details: &str) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "{}📊 {} {}",
            self.build_child_debug_prefix(),
            operation_type,
            details
        );
    }

    /// Print literal content boundaries for multi-template debugging
    pub fn print_literal_boundaries(&self, content: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("{}LITERAL CONTENT", self.build_debug_prefix());
        eprintln!("{}Content: {}", self.build_child_debug_prefix(), content);
        eprintln!(
            "{}═══════════════════════════════════════════════",
            self.build_debug_prefix()
        );
    }

    /// Format literal content for debug display
    pub fn format_literal_content(text: &str) -> String {
        if text.trim().is_empty() && text.len() <= 2 {
            "whitespace".to_string()
        } else if text.len() <= 20 {
            format!("'{}'", text)
        } else {
            format!("'{}...' ({} chars)", &text[..15], text.len())
        }
    }
}

/// Active debugger for tracking individual pipeline steps
pub struct PipelineDebugger {
    enabled: bool,
    indent_level: usize,
}

impl PipelineDebugger {
    fn new(enabled: bool, indent_level: usize) -> Self {
        Self {
            enabled,
            indent_level,
        }
    }

    /// Start debugging a pipeline step
    pub fn start_step(
        &self,
        step_num: usize,
        total_steps: usize,
        operation: &StringOp,
        input: &Value,
        _is_last: bool,
    ) -> Option<StepDebugger> {
        if !self.enabled {
            return None;
        }

        let prefix = self.build_debug_prefix();
        let child_prefix = self.build_child_debug_prefix();

        eprintln!(
            "{}STEP {}/{}: Applying {}",
            prefix,
            step_num,
            total_steps,
            OperationFormatter::format_operation(operation, OperationDisplayMode::Debug)
        );

        eprintln!(
            "{}Input: {}",
            child_prefix,
            DebugSession::format_value_for_display(input)
        );

        Some(StepDebugger::new(Instant::now(), self.indent_level))
    }

    /// Debug map operation item processing start
    pub fn debug_map_item_start(&self, item_num: usize, total_items: usize, item: &str) {
        if !self.enabled {
            return;
        }

        let prefix = self.build_debug_prefix();

        eprintln!("{}Processing item {} of {}", prefix, item_num, total_items);
        eprintln!("{}Map item input: {:?}", prefix, item);
    }

    /// Debug map operation item processing end
    pub fn debug_map_item_end(
        &self,
        _item_num: usize,
        _total_items: usize,
        result: &Result<String, String>,
    ) {
        if !self.enabled {
            return;
        }

        let prefix = self.build_debug_prefix();

        match result {
            Ok(output) => eprintln!("{}Map item output: {:?}", prefix, output),
            Err(e) => eprintln!("{}❌ Map item ERROR: {}", prefix, e),
        }
    }

    /// Build debug prefix for current level
    fn build_debug_prefix(&self) -> String {
        DebugPrefix::build(self.indent_level)
    }

    /// Build debug prefix for child level
    fn build_child_debug_prefix(&self) -> String {
        DebugPrefix::build_child(self.indent_level)
    }
}

/// Individual step debugger for timing and result tracking
pub struct StepDebugger {
    start_time: Instant,
    indent_level: usize,
}

impl StepDebugger {
    fn new(start_time: Instant, indent_level: usize) -> Self {
        Self {
            start_time,
            indent_level,
        }
    }

    /// Complete the step debugging with result information
    pub fn complete(self, output: &Value) -> std::time::Duration {
        let duration = self.start_time.elapsed();
        let child_prefix = DebugPrefix::build_child(self.indent_level);

        eprintln!(
            "{}🎯 Result: {}",
            child_prefix,
            DebugSession::format_value_for_display(output)
        );
        eprintln!("{}Step completed in {:?}", child_prefix, duration);

        // Add memory usage info for large datasets
        match output {
            Value::List(list) if list.len() > 1000 => {
                let total_chars: usize = list.iter().map(|s| s.len()).sum();
                eprintln!(
                    "{}Memory: ~{} chars across {} items",
                    child_prefix,
                    total_chars,
                    list.len()
                );
            }
            Value::Str(s) if s.len() > 10000 => {
                eprintln!("{}Memory: ~{} chars in string", child_prefix, s.len());
            }
            _ => {}
        }

        duration
    }
}

/// Specialized debugger for MultiTemplate section processing
pub struct MultiTemplateDebugger {
    enabled: bool,
    debug_session: DebugSession,
}

impl MultiTemplateDebugger {
    pub fn new(enabled: bool, template_raw: String, sections_count: usize) -> Self {
        Self {
            enabled,
            debug_session: DebugSession::new_multi_template(enabled, template_raw, sections_count),
        }
    }

    pub fn start(&mut self, input: &str) -> Option<&mut DebugSession> {
        if !self.enabled {
            return None;
        }

        // Print the multi-template header
        eprintln!("DEBUG: ═══════════════════════════════════════════════");

        if let SessionType::MultiTemplate { sections_count } = self.debug_session.session_type() {
            eprintln!("DEBUG: MULTI-TEMPLATE START");
            eprintln!("DEBUG: Template: {:?}", self.debug_session.template_raw());
            eprintln!("DEBUG: Input: {:?}", input);
            eprintln!("DEBUG: Sections: {}", sections_count);
        }

        Some(&mut self.debug_session)
    }

    pub fn end(&mut self, final_result: &str, cache_stats: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: ✅ MULTI-TEMPLATE COMPLETE");
        eprintln!("DEBUG:   🎯 Final result: {:?}", final_result);
        eprintln!("DEBUG:   {}", cache_stats);
        eprintln!("DEBUG: ═══════════════════════════════════════════════");
    }
}
