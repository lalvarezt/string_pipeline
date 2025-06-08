//! Debug context for pipeline operations.
//!
//! This module contains the debug context implementation that provides
//! detailed logging and tracing capabilities for pipeline execution.

use crate::pipeline::Value;

/// Debug context for tracking pipeline execution.
///
/// Provides detailed logging and tracing capabilities when debug mode is enabled,
/// including operation tracking, timing information, and data flow visualization.
#[derive(Debug, Clone)]
pub struct DebugContext {
    enabled: bool,
    template_raw: Option<String>,
    is_sub_pipeline: bool,
}

impl DebugContext {
    /// Create a new debug context for a template
    pub fn new_template(enabled: bool, template_raw: String) -> Self {
        Self {
            enabled,
            template_raw: Some(template_raw),
            is_sub_pipeline: false,
        }
    }

    /// Create a new debug context for map item processing
    pub fn new_map_item(enabled: bool, _item_num: usize, _total_items: usize) -> Self {
        Self {
            enabled,
            template_raw: None,
            is_sub_pipeline: true,
        }
    }

    /// Create a context with a specific depth
    pub fn with_depth(&self, _depth: usize) -> Self {
        Self {
            enabled: self.enabled,
            template_raw: self.template_raw.clone(),
            is_sub_pipeline: self.is_sub_pipeline,
        }
    }

    /// Create a context with an operation added to the path
    pub fn with_operation(&self, _op_name: &str) -> Self {
        Self {
            enabled: self.enabled,
            template_raw: self.template_raw.clone(),
            is_sub_pipeline: self.is_sub_pipeline,
        }
    }

    /// Check if this context is for a sub-pipeline
    pub fn is_sub_pipeline(&self) -> bool {
        self.is_sub_pipeline
    }

    /// Print template start header
    pub fn print_template_header(&self, title: &str, input: &str, additional_info: Option<&str>) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: ═══════════════════════════════════════════════");
        eprintln!("DEBUG: {} START", title);
        if let Some(template) = &self.template_raw {
            eprintln!("DEBUG: Template: {:?}", template);
        }
        eprintln!("DEBUG: Input: {:?}", input);
        if let Some(info) = additional_info {
            eprintln!("DEBUG: {}", info);
        }
        eprintln!("DEBUG: ───────────────────────────────────────────────");
    }

    /// Print template completion footer
    pub fn print_template_footer(&self, title: &str, result: &str, cache_info: Option<&str>) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: ✅ {} COMPLETE", title);
        eprintln!("DEBUG: 🎯 Final result: {:?}", result);
        if let Some(info) = cache_info {
            eprintln!("DEBUG: {}", info);
        }
        eprintln!("DEBUG: ═══════════════════════════════════════════════");
    }

    /// Print a debug message with simple formatting
    pub fn print(&self, message: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: {}", message);
    }

    /// Print section information
    pub fn print_section(
        &self,
        section_num: usize,
        total_sections: usize,
        section_type: &str,
        content: &str,
    ) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "DEBUG: SECTION {}/{}: [{}{}]",
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

    /// Print operation result
    pub fn print_result(&self, _prefix: &str, result: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: 🎯 Result: {:?}", result);
    }

    /// Print cache operation
    pub fn print_cache_operation(&self, operation_type: &str, details: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: {} {}", operation_type, details);
    }

    /// Print step information
    pub fn print_step(&self, step_info: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: {}", step_info);
    }

    /// Print operation details for pipeline steps
    pub fn print_operation(&self, op: &crate::pipeline::StringOp, step_num: usize) {
        if !self.enabled {
            return;
        }

        match op {
            crate::pipeline::StringOp::Map { operations } => {
                if operations.len() <= 3 {
                    let ops_str: Vec<String> =
                        operations.iter().map(|op| format!("{:?}", op)).collect();
                    eprintln!(
                        "DEBUG:   {}. Map {{ operations: [{}] }}",
                        step_num,
                        ops_str.join(", ")
                    );
                } else {
                    eprintln!("DEBUG:   {}. Map {{ operations: [", step_num);
                    for (i, map_op) in operations.iter().enumerate() {
                        eprintln!("DEBUG:       {}: {:?}", i + 1, map_op);
                    }
                    eprintln!("DEBUG:     ] }}");
                }
            }
            _ => eprintln!("DEBUG:   {}. {:?}", step_num, op),
        }
    }

    /// Format and print values with simple output
    pub fn print_value(&self, val: &Value, prefix_text: &str) {
        if !self.enabled {
            return;
        }

        match val {
            Value::Str(s) => {
                if s.len() > 100 {
                    eprintln!(
                        "DEBUG: {}String(len={}, preview={:?}...)",
                        prefix_text,
                        s.len(),
                        &s[..50]
                    );
                } else {
                    eprintln!("DEBUG: {}String({:?})", prefix_text, s);
                }
            }
            Value::List(list) => {
                if list.is_empty() {
                    eprintln!("DEBUG: {}List(empty)", prefix_text);
                } else if list.len() == 1 {
                    eprintln!("DEBUG: {}List(1 item: {:?})", prefix_text, list[0]);
                } else if list.len() <= 5 {
                    eprintln!("DEBUG: {}List({} items: [", prefix_text, list.len());
                    for item in list {
                        eprintln!("DEBUG:   {:?}", item);
                    }
                    eprintln!("DEBUG: ])");
                } else {
                    eprintln!("DEBUG: {}List({} items: [", prefix_text, list.len());
                    for item in &list[..3] {
                        eprintln!("DEBUG:   {:?}", item);
                    }
                    eprintln!("DEBUG:   ...");
                    for item in &list[list.len() - 2..] {
                        eprintln!("DEBUG:   {:?}", item);
                    }
                    eprintln!("DEBUG: ])");
                }
            }
        }
    }

    /// Format value as string (for inline display)
    pub fn format_value(&self, val: &Value) -> String {
        match val {
            Value::Str(s) => {
                if s.len() > 100 {
                    format!("String(len={}, preview={:?}...)", s.len(), &s[..50])
                } else {
                    format!("String({:?})", s)
                }
            }
            Value::List(list) => {
                if list.is_empty() {
                    "List(empty)".to_string()
                } else if list.len() == 1 {
                    format!("List(1 item: {:?})", list[0])
                } else if list.len() <= 5 {
                    format!("List({} items: {:?})", list.len(), list)
                } else {
                    format!(
                        "List({} items: {:?}...{:?})",
                        list.len(),
                        &list[..3],
                        &list[list.len() - 2..]
                    )
                }
            }
        }
    }

    /// Print map item processing start header
    pub fn print_map_item_start(&self, item_idx: usize, total_items: usize) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: Processing item {} of {}", item_idx, total_items);
    }

    /// Print map item input
    pub fn print_map_item_input(&self, input: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: Map item input: {:?}", input);
    }

    /// Print map item output
    pub fn print_map_item_output(&self, output: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: Map item output: {:?}", output);
    }

    /// Print map item error
    pub fn print_map_item_error(&self, error: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("DEBUG: ❌ Map item ERROR: {}", error);
    }
}
