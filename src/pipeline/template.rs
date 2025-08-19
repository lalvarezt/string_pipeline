//! Template engine for string transformation with mixed literal and operation sections.
//!
//! This module provides the `MultiTemplate` type, which supports templates containing
//! both literal text and transformation sections. Templates can mix static content
//! with dynamic string operations, enabling complex text processing patterns.
//!
//! # Template Syntax
//!
//! Templates consist of:
//! - **Literal sections**: Plain text that appears as-is in the output
//! - **Template sections**: `{operation|operation|...}` blocks that transform the input
//!
//! # Examples
//!
//! ```rust
//! use string_pipeline::Template;
//!
//! // Mixed literal and template sections
//! let template = Template::parse("Hello {upper}!").unwrap();
//! assert_eq!(template.format("world").unwrap(), "Hello WORLD!");
//!
//! // Multiple template sections
//! let template = Template::parse("Name: {split: :0} | Email: {split: :1}").unwrap();
//! assert_eq!(template.format("john doe john@example.com").unwrap(),
//!            "Name: john | Email: doe");
//!
//! // Complex transformations
//! let template = Template::parse("Files: {split:,:..|filter:\\.txt$|join:, }").unwrap();
//! assert_eq!(template.format("file1.txt,doc.pdf,file2.txt").unwrap(),
//!            "Files: file1.txt, file2.txt");
//! ```
//!
//! # Performance Features
//!
//! - **Operation Caching**: Template section results are cached per input to avoid recomputation
//! - **Fast Single Split**: Single split operations use an optimized code path
//! - **String Interning**: Common separators are interned to reduce memory allocations
//! - **Regex Caching**: Compiled regex patterns are cached globally for reuse
//!
//! # Debug Mode
//!
//! Templates support comprehensive debug output showing:
//! - Template parsing and section breakdown
//! - Operation execution steps with timing
//! - Cache hit/miss statistics
//! - Input/output values at each stage

use std::collections::{HashMap, hash_map::DefaultHasher};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::pipeline::get_cached_split;
use crate::pipeline::{DebugTracer, RangeSpec, StringOp, apply_ops_internal, apply_range, parser}; // ← use global split cache

/* ------------------------------------------------------------------------ */
/*  MultiTemplate – the single implementation                               */
/* ------------------------------------------------------------------------ */

/// A template engine supporting mixed literal text and string transformation operations.
///
/// `MultiTemplate` can contain any combination of literal text sections and template
/// sections that apply string operations to input data. This enables creating complex
/// text formatting patterns while maintaining high performance through intelligent caching.
///
/// # Template Structure
///
/// Templates are parsed into sections:
/// - **Literal sections**: Static text that appears unchanged in output
/// - **Template sections**: Operation sequences in `{op1|op2|...}` format
///
/// # Caching Strategy
///
/// The template engine employs multiple levels of caching:
/// - **Split cache**: Common string splitting operations are cached globally
/// - **Regex cache**: Compiled regex patterns are cached for reuse
/// - **Operation cache**: Results of template sections are cached per input hash
///
/// # Debug Support
///
/// When debug mode is enabled, templates provide detailed execution tracing:
/// - Section-by-section processing breakdown
/// - Operation timing and cache statistics
/// - Input/output transformations
///
/// # Examples
///
/// ## Basic Template Usage
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("Result: {upper|trim}").unwrap();
/// assert_eq!(template.format("  hello  ").unwrap(), "Result: HELLO");
/// ```
///
/// ## Multi-Section Templates
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("User: {split: :0} ({split: :1})").unwrap();
/// assert_eq!(template.format("john smith").unwrap(), "User: john (smith)");
/// ```
///
/// ## Debug Mode
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse_with_debug("{split:,:..|sort|join: \\| }", Some(true)).unwrap();
/// let result = template.format("c,a,b").unwrap(); // Prints debug info to stderr
/// assert_eq!(result, "a | b | c");
/// ```
#[derive(Debug, Clone)]
pub struct MultiTemplate {
    raw: String,
    sections: Vec<TemplateSection>,
    debug: bool,
}

/* ---------- helper enums ------------------------------------------------- */

/// Represents a section within a parsed template.
///
/// Templates are decomposed into alternating literal and template sections,
/// allowing for efficient processing and caching of the transformation parts.
#[derive(Debug, Clone)]
pub enum TemplateSection {
    /// A literal text section that appears unchanged in the output.
    Literal(String),
    /// A template section containing a sequence of string operations to apply.
    Template(Vec<StringOp>),
}

/// Type of template section for introspection and analysis.
///
/// Distinguishes between literal text sections and template operation sections
/// when examining template structure programmatically. Used by introspection
/// methods like [`MultiTemplate::get_section_info`] to provide detailed template analysis.
///
/// # Examples
///
/// ```rust
/// use string_pipeline::{Template, SectionType};
///
/// let template = Template::parse("Hello {upper} world!").unwrap();
/// let sections = template.get_section_info();
///
/// assert_eq!(sections[0].section_type, SectionType::Literal);  // "Hello "
/// assert_eq!(sections[1].section_type, SectionType::Template); // {upper}
/// assert_eq!(sections[2].section_type, SectionType::Literal);  // " world!"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    /// A literal text section that appears unchanged in template output.
    ///
    /// Literal sections contain static text that is copied directly to the
    /// output without any transformation or processing.
    Literal,
    /// A template section containing string operations.
    ///
    /// Template sections contain operation sequences like `{upper|trim}` that
    /// transform input data before including it in the output.
    Template,
}

/// Detailed information about a template section for introspection and debugging.
///
/// Provides comprehensive metadata about each section in a template, including
/// its type, position, and content. Used by [`MultiTemplate::get_section_info`]
/// to enable programmatic template analysis and debugging.
///
/// This struct contains all necessary information to understand both the structure
/// and content of template sections, making it useful for tools that need to
/// analyze or manipulate templates programmatically.
///
/// # Field Details
///
/// - **`section_type`**: Whether this is a literal text section or template operation section
/// - **`overall_position`**: Zero-based position among all sections in the template
/// - **`template_position`**: Zero-based position among template sections only (None for literals)
/// - **`content`**: The literal text content (populated only for literal sections)
/// - **`operations`**: The operation sequence (populated only for template sections)
///
/// # Examples
///
/// ```rust
/// use string_pipeline::{Template, SectionType};
///
/// let template = Template::parse("Name: {upper} | Age: {lower}").unwrap();
/// let sections = template.get_section_info();
///
/// // First section: "Name: "
/// assert_eq!(sections[0].section_type, SectionType::Literal);
/// assert_eq!(sections[0].overall_position, 0);
/// assert_eq!(sections[0].template_position, None);
/// assert_eq!(sections[0].content, Some("Name: ".to_string()));
/// assert!(sections[0].operations.is_none());
///
/// // Second section: {upper}
/// assert_eq!(sections[1].section_type, SectionType::Template);
/// assert_eq!(sections[1].overall_position, 1);
/// assert_eq!(sections[1].template_position, Some(0));
/// assert!(sections[1].content.is_none());
/// assert_eq!(sections[1].operations.as_ref().unwrap().len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// The type of this section (literal or template).
    pub section_type: SectionType,
    /// Position within all sections (both literal and template).
    pub overall_position: usize,
    /// Position among template sections only (None for literal sections).
    pub template_position: Option<usize>,
    /// Text content for literal sections (None for template sections).
    pub content: Option<String>,
    /// Operations for template sections (None for literal sections).
    pub operations: Option<Vec<StringOp>>,
}

/* ---------- per-format call cache (operation results only) -------------- */

/// Per-template-instance cache for operation results.
///
/// Caches the results of template section execution to avoid recomputing
/// identical operations on the same input data within a single format call.
struct TemplateCache {
    operations: HashMap<CacheKey, String>,
}

impl TemplateCache {
    fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }
}

/// Cache key combining input hash and operation signature.
///
/// This key uniquely identifies a specific input string and operation sequence
/// combination, enabling safe result caching across template section executions.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    input_hash: u64,
    ops_signature: String,
}

/* ------------------------------------------------------------------------ */
/*  impl MultiTemplate                                                      */
/* ------------------------------------------------------------------------ */

impl MultiTemplate {
    fn new(raw: String, sections: Vec<TemplateSection>, debug: bool) -> Self {
        Self {
            raw,
            sections,
            debug,
        }
    }

    /* -------- constructors ---------------------------------------------- */

    /// Parse a template string into a `MultiTemplate` instance.
    ///
    /// Parses template syntax containing literal text and `{operation}` blocks,
    /// with support for complex operation pipelines, debug information is suppressed.
    ///
    /// # Arguments
    ///
    /// * `template` - The template string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(MultiTemplate)` - Successfully parsed template
    /// * `Err(String)` - Parse error description
    ///
    /// # Template Syntax
    ///
    /// - Literal text appears as-is in output
    /// - `{operation}` blocks apply transformations to input
    /// - Multiple operations: `{op1|op2|op3}`
    /// - Debug markers: `{!debug}` are suppressed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Simple template
    /// let template = Template::parse("Hello {upper}!").unwrap();
    ///
    /// // Complex pipeline
    /// let template = Template::parse("{split:,:..|sort|join: - }").unwrap();
    /// ```
    pub fn parse(template: &str) -> Result<Self, String> {
        // Fast-path: if the input is a *single* template block (no outer-level
        // literal text) we can skip the multi-template scanner and directly
        // parse the operation list.
        if let Some(single) = Self::try_single_block(template)? {
            return Ok(single);
        }

        let (sections, _) = parser::parse_multi_template(template)?;
        Ok(Self::new(template.to_string(), sections, false))
    }

    /// Parse a template string into a `MultiTemplate` instance.
    ///
    /// Parses template syntax containing literal text and `{operation}` blocks,
    /// with support for complex operation pipelines and debug mode.
    ///
    /// # Arguments
    ///
    /// * `template` - The template string to parse
    /// * `debug` - Optional debug mode override (None uses template's debug markers)
    ///
    /// # Returns
    ///
    /// * `Ok(MultiTemplate)` - Successfully parsed template
    /// * `Err(String)` - Parse error description
    ///
    /// # Template Syntax
    ///
    /// - Literal text appears as-is in output
    /// - `{operation}` blocks apply transformations to input
    /// - Multiple operations: `{op1|op2|op3}`
    /// - Debug markers: `{!debug}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Simple template
    /// let template = Template::parse_with_debug("Hello {upper}!", None).unwrap();
    ///
    /// // Complex pipeline
    /// let template = Template::parse_with_debug("{split:,:..|sort|join: - }", None).unwrap();
    ///
    /// // Debug enabled
    /// let template = Template::parse_with_debug("{!upper|trim}", None).unwrap();
    ///
    /// // Debug override
    /// let template = Template::parse_with_debug("{upper}", Some(true)).unwrap();
    /// ```
    pub fn parse_with_debug(template: &str, debug: Option<bool>) -> Result<Self, String> {
        // Re-use the single-block shortcut when applicable.
        if let Some(mut single) = Self::try_single_block(template)? {
            if let Some(dbg_override) = debug {
                single.debug = dbg_override;
            }
            return Ok(single);
        }

        let (sections, inner_dbg) = parser::parse_multi_template(template)?;
        Ok(Self::new(
            template.to_string(),
            sections,
            debug.unwrap_or(inner_dbg),
        ))
    }

    /* -------- formatting ------------------------------------------------- */

    /// Apply the template to input data, producing formatted output.
    ///
    /// Processes each template section in sequence, applying literal text directly
    /// and executing operation pipelines on the input data. Results are cached
    /// per input to optimize repeated operations.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to transform
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The formatted result
    /// * `Err(String)` - Error description if processing fails
    ///
    /// # Performance
    ///
    /// - Template sections with identical operations and input are cached
    /// - Single split operations use a fast path
    /// - Common separators are interned to reduce allocations
    /// - ASCII-only operations use optimized algorithms where possible
    ///
    /// # Debug Output
    ///
    /// When debug mode is enabled, detailed execution information is printed to stderr:
    /// - Section-by-section processing
    /// - Operation timing and cache statistics
    /// - Input/output transformations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Name: {split: :0}, Age: {split: :1}").unwrap();
    /// let result = template.format("John 25").unwrap();
    /// assert_eq!(result, "Name: John, Age: 25");
    ///
    /// // List processing
    /// let template = Template::parse("Items: {split:,:..|sort|join: \\| }").unwrap();
    /// let result = template.format("apple,banana,cherry").unwrap();
    /// assert_eq!(result, "Items: apple | banana | cherry");
    /// ```
    pub fn format(&self, input: &str) -> Result<String, String> {
        use std::time::Instant;

        let mut cache = TemplateCache::new();
        let mut result = String::new();

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let input_hash = hasher.finish();

        /* -------- optional debug session -------------------------------- */

        let start_time = if self.debug {
            Some(Instant::now())
        } else {
            None
        };

        if self.debug {
            let tracer = DebugTracer::new(true);
            let info = format!(
                "{} sections (literal: {}, template: {})",
                self.sections.len(),
                self.sections.len() - self.template_section_count(),
                self.template_section_count()
            );
            tracer.session_start("MULTI-TEMPLATE", &self.raw, input, Some(&info));

            for (idx, section) in self.sections.iter().enumerate() {
                match section {
                    TemplateSection::Literal(text) => {
                        let preview = if text.trim().is_empty() && text.len() <= 2 {
                            "whitespace".to_string()
                        } else if text.len() <= 20 {
                            format!("'{text}'")
                        } else {
                            format!("'{}...' ({} chars)", &text[..15], text.len())
                        };
                        tracer.section(idx + 1, self.sections.len(), "literal", &preview);
                        result.push_str(text);
                        if idx + 1 < self.sections.len() {
                            tracer.separator();
                        }
                    }
                    TemplateSection::Template(ops) => {
                        let summary = Self::format_operations_summary(ops);
                        tracer.section(idx + 1, self.sections.len(), "template", &summary);
                        let out = self.apply_template_section(
                            input,
                            ops,
                            input_hash,
                            &mut cache,
                            &Some(&tracer),
                        )?;
                        result.push_str(&out);
                    }
                }
            }

            tracer.session_end("MULTI-TEMPLATE", &result, start_time.unwrap().elapsed());
        } else {
            for section in &self.sections {
                match section {
                    TemplateSection::Literal(text) => result.push_str(text),
                    TemplateSection::Template(ops) => {
                        let out =
                            self.apply_template_section(input, ops, input_hash, &mut cache, &None)?;
                        result.push_str(&out);
                    }
                }
            }
        }

        Ok(result)
    }

    /* -------- public helpers ------------------------------------------- */

    /// Get the original template string.
    ///
    /// Returns the raw template string that was used to create this instance,
    /// useful for debugging, serialization, or displaying template definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Hello {upper}!").unwrap();
    /// assert_eq!(template.template_string(), "Hello {upper}!");
    /// ```
    pub fn template_string(&self) -> &str {
        &self.raw
    }

    /// Get the total number of sections in the template.
    ///
    /// Returns the count of all sections (both literal and template sections)
    /// that make up this template.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Hello {upper} world!").unwrap();
    /// assert_eq!(template.section_count(), 3); // "Hello ", "{upper}", " world!"
    /// ```
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Get the number of template sections (excluding literals).
    ///
    /// Returns the count of sections that contain operations, excluding
    /// literal text sections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Hello {upper} world {lower}!").unwrap();
    /// assert_eq!(template.template_section_count(), 2); // {upper} and {lower}
    /// ```
    pub fn template_section_count(&self) -> usize {
        self.sections
            .iter()
            .filter(|s| matches!(s, TemplateSection::Template(_)))
            .count()
    }

    /// Check if debug mode is enabled.
    ///
    /// Returns `true` if this template will output debug information during
    /// formatting operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse_with_debug("{upper}", Some(true)).unwrap();
    /// assert!(template.is_debug());
    /// ```
    pub fn is_debug(&self) -> bool {
        self.debug
    }

    /// Create a new template instance with debug mode set.
    ///
    /// Returns a new template with the specified debug setting, leaving
    /// the original unchanged.
    ///
    /// # Arguments
    ///
    /// * `debug` - Whether to enable debug mode
    /// ```
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set debug mode on this template instance.
    ///
    /// Modifies this template's debug setting in place.
    ///
    /// # Arguments
    ///
    /// * `debug` - Whether to enable debug mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let mut template = Template::parse("{upper}").unwrap();
    /// template.set_debug(true);
    /// assert!(template.is_debug());
    /// ```
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /* -------- structured template processing ----------------------------- */

    /// Format template with multiple inputs per template section.
    ///
    /// This method enables advanced template processing where each template section
    /// can receive multiple input values that are joined with individual separators.
    /// This is useful for complex formatting scenarios like batch processing or
    /// command construction where different template sections need different data.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Slice of input slices, where each inner slice contains the inputs for one template section
    /// * `separators` - Slice of separators, one for each template section to join multiple inputs
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The formatted result with each template section processed with its joined inputs
    /// * `Err(String)` - Error if inputs/separators length doesn't match template section count or processing fails
    ///
    /// # Template Section Ordering
    ///
    /// Template sections are numbered from left to right, starting at 0. Literal sections
    /// are not counted. For example, in `"Hello {upper} world {lower}!"`:
    /// - Template section 0: `{upper}`
    /// - Template section 1: `{lower}`
    /// - Total template sections: 2
    ///
    /// # Input Processing
    ///
    /// For each template section:
    /// - **Empty slice `[]`**: Uses empty string as input
    /// - **Single item `["value"]`**: Uses "value" directly as input
    /// - **Multiple items `["a", "b", "c"]`**: Joins with corresponding separator
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of input slices doesn't match the number of template sections
    /// - The number of separators doesn't match the number of template sections
    /// - Any template section processing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Multiple inputs for first section, single input for second
    /// let template = Template::parse("Users: {upper} | Email: {lower}").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["john doe", "peter parker"],
    ///     &["ADMIN@EXAMPLE.COM"],
    /// ], &[" ", " "]).unwrap();
    /// assert_eq!(result, "Users: JOHN DOE PETER PARKER | Email: admin@example.com");
    ///
    /// // File batch processing with different separators
    /// let template = Template::parse("tar -czf {lower}.tar.gz {join: }").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["BACKUP"],
    ///     &["file1.txt", "file2.txt", "file3.txt"],
    /// ], &[" ", " "]).unwrap();
    /// assert_eq!(result, "tar -czf backup.tar.gz file1.txt file2.txt file3.txt");
    ///
    /// // Command construction with custom separators
    /// let template = Template::parse("grep {join:\\|} {join:,}").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["error", "warning"],
    ///     &["log1.txt", "log2.txt"],
    /// ], &["|", ","]).unwrap();
    /// assert_eq!(result, "grep error|warning log1.txt,log2.txt");
    /// ```
    pub fn format_with_inputs(
        &self,
        inputs: &[&[&str]],
        separators: &[&str],
    ) -> Result<String, String> {
        let template_sections_count = self.template_section_count();

        if inputs.len() != template_sections_count {
            return Err(format!(
                "Expected {} input slices for {} template sections, got {}",
                template_sections_count,
                template_sections_count,
                inputs.len()
            ));
        }

        if separators.len() != template_sections_count {
            return Err(format!(
                "Expected {} separators for {} template sections, got {}",
                template_sections_count,
                template_sections_count,
                separators.len()
            ));
        }

        let mut result = String::new();
        let mut template_index = 0;
        let mut cache = TemplateCache::new();

        for section in &self.sections {
            match section {
                TemplateSection::Literal(text) => {
                    result.push_str(text);
                }
                TemplateSection::Template(ops) => {
                    if template_index >= inputs.len() {
                        return Err("Internal error: template index out of bounds".to_string());
                    }

                    // Process each input individually, then join the results
                    let section_inputs = inputs[template_index];
                    let separator = separators[template_index];
                    let output = match section_inputs.len() {
                        0 => String::new(),
                        1 => {
                            let mut input_hasher = std::collections::hash_map::DefaultHasher::new();
                            std::hash::Hash::hash(&section_inputs[0], &mut input_hasher);
                            let input_hash = input_hasher.finish();

                            self.apply_template_section(
                                section_inputs[0],
                                ops,
                                input_hash,
                                &mut cache,
                                &None, // No debug tracing for structured processing
                            )?
                        }
                        _ => {
                            let mut results = Vec::new();
                            for input in section_inputs {
                                let mut input_hasher =
                                    std::collections::hash_map::DefaultHasher::new();
                                std::hash::Hash::hash(&input, &mut input_hasher);
                                let input_hash = input_hasher.finish();

                                let result = self.apply_template_section(
                                    input, ops, input_hash, &mut cache,
                                    &None, // No debug tracing for structured processing
                                )?;
                                results.push(result);
                            }
                            results.join(separator)
                        }
                    };
                    result.push_str(&output);
                    template_index += 1;
                }
            }
        }

        Ok(result)
    }

    /// Get information about template sections for introspection.
    ///
    /// Returns a vector of tuples containing the position and operations for each
    /// template section in the template. This is useful for understanding the
    /// structure of a template before processing it with `format_with_inputs`.
    ///
    /// # Returns
    ///
    /// A vector where each element is a tuple of:
    /// - `usize` - The position/index of the template section (0-based)
    /// - `&Vec<StringOp>` - Reference to the operations in that section
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Hello {upper} world {lower|trim}!").unwrap();
    /// let sections = template.get_template_sections();
    ///
    /// assert_eq!(sections.len(), 2);
    /// assert_eq!(sections[0].0, 0); // First template section at position 0
    /// assert_eq!(sections[1].0, 1); // Second template section at position 1
    /// assert_eq!(sections[0].1.len(), 1); // {upper} has 1 operation
    /// assert_eq!(sections[1].1.len(), 2); // {lower|trim} has 2 operations
    /// ```
    pub fn get_template_sections(&self) -> Vec<(usize, &Vec<StringOp>)> {
        let mut result = Vec::new();
        let mut template_index = 0;

        for section in &self.sections {
            if let TemplateSection::Template(ops) = section {
                result.push((template_index, ops));
                template_index += 1;
            }
        }

        result
    }

    /// Get detailed information about all sections in the template.
    ///
    /// Returns information about both literal and template sections, including
    /// their types, positions, and content. This provides a complete view of
    /// the template structure for debugging and introspection.
    ///
    /// # Returns
    ///
    /// A vector of section information structs containing:
    /// - Section type (literal or template)
    /// - Position within all sections
    /// - Content or operation details
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("Hello {upper} world!").unwrap();
    /// let info = template.get_section_info();
    ///
    /// assert_eq!(info.len(), 3);
    /// // info[0]: Literal("Hello ")
    /// // info[1]: Template(position=0, operations=[Upper])
    /// // info[2]: Literal(" world!")
    /// ```
    pub fn get_section_info(&self) -> Vec<SectionInfo> {
        let mut result = Vec::new();
        let mut template_position = 0;

        for (overall_position, section) in self.sections.iter().enumerate() {
            match section {
                TemplateSection::Literal(text) => {
                    result.push(SectionInfo {
                        section_type: SectionType::Literal,
                        overall_position,
                        template_position: None,
                        content: Some(text.clone()),
                        operations: None,
                    });
                }
                TemplateSection::Template(ops) => {
                    result.push(SectionInfo {
                        section_type: SectionType::Template,
                        overall_position,
                        template_position: Some(template_position),
                        content: None,
                        operations: Some(ops.clone()),
                    });
                    template_position += 1;
                }
            }
        }

        result
    }

    /* ------------------------------------------------------------------ */
    /*  internal helpers                                                   */
    /* ------------------------------------------------------------------ */

    fn apply_template_section(
        &self,
        input: &str,
        ops: &[StringOp],
        input_hash: u64,
        cache: &mut TemplateCache,
        dbg: &Option<&DebugTracer>,
    ) -> Result<String, String> {
        /* fast path: single split --------------------------------------- */
        if ops.len() == 1
            && let StringOp::Split { sep, range } = &ops[0]
        {
            if let Some(t) = dbg {
                t.cache_operation("FAST SPLIT", &format!("by '{sep}'"));
            }
            return Ok(self.fast_single_split(input, sep, range));
        }

        /* general path – memoised per call ------------------------------ */

        let key = CacheKey {
            input_hash,
            ops_signature: format!("{ops:?}"),
        };

        if let Some(cached) = cache.operations.get(&key) {
            if let Some(t) = dbg {
                t.cache_operation("CACHE HIT", "re-using formatted section");
            }
            return Ok(cached.clone());
        }

        if let Some(t) = dbg {
            t.cache_operation("CACHE MISS", "computing section");
        }

        let nested_dbg = if self.debug {
            Some(DebugTracer::new(true))
        } else {
            None
        };
        let out = apply_ops_internal(input, ops, self.debug, nested_dbg)?;
        cache.operations.insert(key, out.clone());
        Ok(out)
    }

    #[inline]
    fn fast_single_split(&self, input: &str, sep: &str, range: &RangeSpec) -> String {
        let parts = get_cached_split(input, sep);
        let selected = apply_range(&parts, range);
        match selected.len() {
            0 => String::new(),
            1 => selected[0].clone(),
            _ => selected.join(sep),
        }
    }

    fn format_operations_summary(ops: &[StringOp]) -> String {
        ops.iter()
            .map(|op| match op {
                StringOp::Split { sep, range } => format!(
                    "split('{sep}', {})",
                    match range {
                        RangeSpec::Index(i) => i.to_string(),
                        RangeSpec::Range(s, e, inc) => match (s, e) {
                            (None, None) => "..".into(),
                            (Some(s), None) => format!("{s}.."),
                            (None, Some(e)) => {
                                if *inc {
                                    format!("..={e}")
                                } else {
                                    format!("..{e}")
                                }
                            }
                            (Some(s), Some(e)) => {
                                let dots = if *inc { "..=" } else { ".." };
                                format!("{s}{dots}{e}")
                            }
                        },
                    }
                ),
                StringOp::Upper => "upper".into(),
                StringOp::Lower => "lower".into(),
                StringOp::Append { suffix } => format!("append('{suffix}')"),
                StringOp::Prepend { prefix } => format!("prepend('{prefix}')"),
                StringOp::Replace {
                    pattern,
                    replacement,
                    ..
                } => format!("replace('{pattern}' → '{replacement}')"),
                _ => format!("{op:?}").to_lowercase(),
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /* -------- helper: detect plain single-block templates ------------- */

    /// Detects and parses templates that consist of exactly one `{ ... }` block
    /// with no surrounding literal text. Returns `Ok(Some(MultiTemplate))` when
    /// the fast path can be applied, `Ok(None)` otherwise.
    fn try_single_block(template: &str) -> Result<Option<Self>, String> {
        // Must start with '{' and end with '}' to be a candidate.
        if !(template.starts_with('{') && template.ends_with('}')) {
            return Ok(None);
        }

        // Verify that the outer-most braces close at the very end and that the
        // brace nesting never returns to zero before the last char.
        let mut depth = 0u32;
        for ch in template[1..template.len() - 1].chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        // Closed the top-level early → literal content exists.
                        return Ok(None);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        if depth != 0 {
            // Unbalanced braces – fall back to full parser for proper error.
            return Ok(None);
        }

        // Safe to treat as single template block.
        let (ops, dbg_flag) = parser::parse_template(template)?;
        let sections = vec![TemplateSection::Template(ops)];
        Ok(Some(Self::new(template.to_string(), sections, dbg_flag)))
    }
}

/* ---------- trait impls -------------------------------------------------- */

/// Provides string representation of the template.
///
/// Returns the original template string, making it easy to display or serialize
/// template definitions.
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("Hello {upper}!").unwrap();
/// println!("{}", template); // Prints: Hello {upper}!
/// ```
impl Display for MultiTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/* ---------- backward compatibility alias --------------------------------- */

/// Type alias for backward compatibility.
///
/// `Template` is an alias for `MultiTemplate`, providing a shorter name for the
/// template type while maintaining compatibility with existing code.
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
/// use string_pipeline::MultiTemplate;
///
/// // These are equivalent:
/// let template1 = Template::parse("{upper}").unwrap();
/// let template2 = MultiTemplate::parse("{upper}").unwrap();
/// ```
pub type Template = MultiTemplate;
