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

use std::collections::{HashMap, HashSet, hash_map::DefaultHasher};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Range;

use crate::pipeline::get_cached_split;
use crate::pipeline::{DebugTracer, RangeSpec, StringOp, apply_ops_internal, apply_range, parser}; // ← use global split cache
use memchr::memchr_iter;

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
    compiled_sections: Vec<CompiledSectionPlan>,
    debug: bool,
}

/* ---------- helper enums ------------------------------------------------- */

#[derive(Debug, Clone)]
enum CompiledSectionPlan {
    Literal,
    Template {
        exec: TemplateExecutionPlan,
        cache_key: u64,
    },
}

#[derive(Debug, Clone)]
struct TemplateExecutionPlan {
    kind: TemplateExecutionKind,
    cache_policy: CachePolicy,
}

#[derive(Debug, Clone)]
enum TemplateExecutionKind {
    Passthrough,
    SplitIndex { sep: String, idx: isize },
    SplitJoinRewrite { split_sep: String, join_sep: String },
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CachePolicy {
    Never,
    PerCall,
}

/// Represents a section within a parsed template.
///
/// Templates are decomposed into alternating literal and template sections,
/// allowing for efficient processing and caching of the transformation parts.
#[derive(Debug, Clone)]
pub enum TemplateSection {
    /// A literal text section that appears unchanged in the output.
    Literal(String),
    /// A template section containing a sequence of string operations to apply.
    Template { ops: Vec<StringOp>, cache_key: u64 },
}

impl TemplateSection {
    pub(crate) fn from_ops(ops: Vec<StringOp>) -> Self {
        let cache_key = MultiTemplate::hash_ops(&ops);
        Self::Template { ops, cache_key }
    }
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

/// Rich output for a single template section.
///
/// This captures the exact string produced for one template section during
/// formatting, along with both its template-only and overall section positions.
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("A: {upper} B: {lower}").unwrap();
/// let result = template.format_rich("MiXeD").unwrap();
///
/// assert_eq!(result.template_outputs[0].template_position, 0);
/// assert_eq!(result.template_outputs[0].overall_position, 1);
/// assert_eq!(result.template_output(0), Some("MIXED"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateOutput {
    /// Position among template sections only.
    pub template_position: usize,
    /// Position among all sections, including literals.
    pub overall_position: usize,
    /// Byte range within [`RichFormatResult::rendered`] for this template section.
    pub rendered_range: Range<usize>,
}

impl TemplateOutput {
    /// Borrow this section's rendered output from the full rendered string.
    pub fn as_str<'a>(&self, rendered: &'a str) -> &'a str {
        &rendered[self.rendered_range.clone()]
    }
}

/// Rich formatting result containing the final rendered string and per-template outputs.
///
/// This type preserves the existing final string output while exposing the
/// individual rendered results for each template section.
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("asd {upper} bsd {lower}").unwrap();
/// let result = template.format_rich("MiXeD").unwrap();
///
/// assert_eq!(result.rendered, "asd MIXED bsd mixed");
/// assert_eq!(result.template_outputs.len(), 2);
/// assert_eq!(result.template_output(0), Some("MIXED"));
/// assert_eq!(result.template_output(1), Some("mixed"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichFormatResult {
    /// Final rendered output, identical to what `format()` would return.
    pub rendered: String,
    /// Outputs for each template section in left-to-right order.
    pub template_outputs: Vec<TemplateOutput>,
}

impl RichFormatResult {
    /// Borrow the output of the template section at `index`.
    pub fn template_output(&self, index: usize) -> Option<&str> {
        self.template_outputs
            .get(index)
            .map(|output| output.as_str(&self.rendered))
    }
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

struct ExecutionContext<'a> {
    input_hash: &'a mut Option<u64>,
    cache: &'a mut TemplateCache,
    dbg: Option<&'a DebugTracer>,
}

/// Cache key combining input hash and operation signature.
///
/// This key uniquely identifies a specific input string and operation sequence
/// combination, enabling safe result caching across template section executions.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    input_hash: u64,
    section_key: u64,
}

struct RenderBuffer {
    rendered: String,
    template_outputs: Option<Vec<TemplateOutput>>,
}

impl RenderBuffer {
    fn new(rendered_capacity: usize, rich_capacity: Option<usize>) -> Self {
        Self {
            rendered: String::with_capacity(rendered_capacity),
            template_outputs: rich_capacity.map(Vec::with_capacity),
        }
    }

    fn push_literal(&mut self, text: &str) {
        self.rendered.push_str(text);
    }

    fn push_template_output(
        &mut self,
        template_position: usize,
        overall_position: usize,
        output: String,
    ) {
        let start = self.rendered.len();
        self.rendered.push_str(&output);
        let end = self.rendered.len();

        if let Some(template_outputs) = &mut self.template_outputs {
            template_outputs.push(TemplateOutput {
                template_position,
                overall_position,
                rendered_range: start..end,
            });
        }
    }

    fn into_rendered(self) -> String {
        self.rendered
    }

    fn into_rich(self) -> RichFormatResult {
        RichFormatResult {
            rendered: self.rendered,
            template_outputs: self.template_outputs.unwrap_or_default(),
        }
    }
}

/* ------------------------------------------------------------------------ */
/*  impl MultiTemplate                                                      */
/* ------------------------------------------------------------------------ */

impl MultiTemplate {
    fn new(raw: String, sections: Vec<TemplateSection>, debug: bool) -> Self {
        let compiled_sections = Self::compile_sections(&sections);
        Self {
            raw,
            sections,
            compiled_sections,
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
        self.render_single_input(input, false)
            .map(RenderBuffer::into_rendered)
    }

    /// Apply the template to input data, returning both the final string and
    /// each rendered template section result.
    ///
    /// `rendered` is identical to the output of [`MultiTemplate::format`], while
    /// `template_outputs` captures the exact text inserted for each template
    /// section in left-to-right order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("asd {upper} bsd {lower}").unwrap();
    /// let result = template.format_rich("MiXeD").unwrap();
    ///
    /// assert_eq!(result.rendered, "asd MIXED bsd mixed");
    /// assert_eq!(result.template_output(0), Some("MIXED"));
    /// assert_eq!(result.template_output(1), Some("mixed"));
    /// ```
    pub fn format_rich(&self, input: &str) -> Result<RichFormatResult, String> {
        self.render_single_input(input, true)
            .map(RenderBuffer::into_rich)
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
            .filter(|s| matches!(s, TemplateSection::Template { .. }))
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
    /// * `Err(String)` - Error if template section processing fails
    ///
    /// # Input/Template/Separator Count Handling
    ///
    /// This method gracefully handles mismatches between counts:
    /// - **Excess inputs**: Extra inputs beyond template section count are truncated/ignored
    /// - **Insufficient inputs**: Missing inputs are treated as empty strings for remaining template sections
    /// - **Excess separators**: Extra separators beyond template section count are truncated/ignored
    /// - **Insufficient separators**: Missing separators default to space " " for remaining template sections
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
    /// // Excess inputs are truncated
    /// let template = Template::parse("diff {} {}").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["file1.txt"],
    ///     &["file2.txt"],
    ///     &["file3.txt"], // This will be ignored
    /// ], &[" ", " "]).unwrap();
    /// assert_eq!(result, "diff file1.txt file2.txt");
    ///
    /// // Insufficient inputs use empty strings
    /// let template = Template::parse("cmd {} {} {}").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["arg1"],
    ///     &["arg2"],
    ///     // Missing third input - will use empty string
    /// ], &[" ", " ", " "]).unwrap();
    /// assert_eq!(result, "cmd arg1 arg2 ");
    ///
    /// // Insufficient separators default to space
    /// let template = Template::parse("files: {} more: {}").unwrap();
    /// let result = template.format_with_inputs(&[
    ///     &["a", "b", "c"],
    ///     &["x", "y", "z"],
    /// ], &[","]).unwrap(); // Only one separator provided
    /// assert_eq!(result, "files: a,b,c more: x y z"); // Second uses default space
    /// ```
    pub fn format_with_inputs(
        &self,
        inputs: &[&[&str]],
        separators: &[&str],
    ) -> Result<String, String> {
        self.render_structured_inputs(inputs, separators, false)
            .map(RenderBuffer::into_rendered)
    }

    /// Format template with multiple inputs per template section, returning both
    /// the final string and each per-section rendered output.
    ///
    /// `template_outputs` contains the exact joined output inserted for each
    /// template section after applying the same input and separator rules as
    /// [`MultiTemplate::format_with_inputs`].
    pub fn format_with_inputs_rich(
        &self,
        inputs: &[&[&str]],
        separators: &[&str],
    ) -> Result<RichFormatResult, String> {
        self.render_structured_inputs(inputs, separators, true)
            .map(RenderBuffer::into_rich)
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
            if let TemplateSection::Template { ops, .. } = section {
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
                TemplateSection::Template { ops, .. } => {
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

    fn render_single_input(&self, input: &str, collect_rich: bool) -> Result<RenderBuffer, String> {
        use std::time::Instant;

        let mut cache = TemplateCache::new();
        let mut input_hash = None;
        let start_time = self.debug.then(Instant::now);
        let tracer = self.debug.then(|| DebugTracer::new(true));

        if let Some(tracer) = tracer.as_ref() {
            let info = format!(
                "{} sections (literal: {}, template: {})",
                self.sections.len(),
                self.sections.len() - self.template_section_count(),
                self.template_section_count()
            );
            tracer.session_start("MULTI-TEMPLATE", &self.raw, input, Some(&info));
        }

        let buffer = self.render_sections(
            self.estimate_output_capacity(input),
            collect_rich,
            tracer.as_ref(),
            |_, ops, exec, cache_key, dbg| {
                self.execute_template_section(
                    input,
                    ops,
                    exec,
                    cache_key,
                    ExecutionContext {
                        input_hash: &mut input_hash,
                        cache: &mut cache,
                        dbg,
                    },
                )
            },
        )?;

        if let (Some(tracer), Some(start_time)) = (tracer.as_ref(), start_time) {
            tracer.session_end("MULTI-TEMPLATE", &buffer.rendered, start_time.elapsed());
        }

        Ok(buffer)
    }

    fn render_structured_inputs(
        &self,
        inputs: &[&[&str]],
        separators: &[&str],
        collect_rich: bool,
    ) -> Result<RenderBuffer, String> {
        let template_sections_count = self.template_section_count();

        let adjusted_inputs: Vec<&[&str]> = (0..template_sections_count)
            .map(|i| inputs.get(i).copied().unwrap_or(&[]))
            .collect();
        let adjusted_separators: Vec<&str> = (0..template_sections_count)
            .map(|i| separators.get(i).copied().unwrap_or(" "))
            .collect();

        let mut cache = TemplateCache::new();

        self.render_sections(
            self.literal_output_capacity(),
            collect_rich,
            None,
            |template_position, ops, exec, cache_key, _| {
                self.execute_structured_template_section(
                    adjusted_inputs[template_position],
                    adjusted_separators[template_position],
                    ops,
                    exec,
                    cache_key,
                    &mut cache,
                )
            },
        )
    }

    fn render_sections<F>(
        &self,
        rendered_capacity: usize,
        collect_rich: bool,
        tracer: Option<&DebugTracer>,
        mut render_template_section: F,
    ) -> Result<RenderBuffer, String>
    where
        F: FnMut(
            usize,
            &[StringOp],
            &TemplateExecutionPlan,
            u64,
            Option<&DebugTracer>,
        ) -> Result<String, String>,
    {
        let mut buffer = RenderBuffer::new(
            rendered_capacity,
            collect_rich.then_some(self.template_section_count()),
        );
        let mut template_position = 0;

        for (overall_position, (section, plan)) in self
            .sections
            .iter()
            .zip(self.compiled_sections.iter())
            .enumerate()
        {
            match (section, plan) {
                (TemplateSection::Literal(text), CompiledSectionPlan::Literal) => {
                    if let Some(tracer) = tracer {
                        let preview = Self::literal_preview(text);
                        tracer.section(
                            overall_position + 1,
                            self.sections.len(),
                            "literal",
                            &preview,
                        );
                    }

                    buffer.push_literal(text);

                    if let Some(tracer) = tracer
                        && overall_position + 1 < self.sections.len()
                    {
                        tracer.separator();
                    }
                }
                (
                    TemplateSection::Template { ops, .. },
                    CompiledSectionPlan::Template { exec, cache_key },
                ) => {
                    if let Some(tracer) = tracer {
                        let summary = Self::format_operations_summary(ops);
                        tracer.section(
                            overall_position + 1,
                            self.sections.len(),
                            "template",
                            &summary,
                        );
                    }

                    let output =
                        render_template_section(template_position, ops, exec, *cache_key, tracer)?;
                    buffer.push_template_output(template_position, overall_position, output);
                    template_position += 1;
                }
                _ => unreachable!("compiled section plan must match template sections"),
            }
        }

        Ok(buffer)
    }

    fn execute_structured_template_section(
        &self,
        section_inputs: &[&str],
        separator: &str,
        ops: &[StringOp],
        exec: &TemplateExecutionPlan,
        cache_key: u64,
        cache: &mut TemplateCache,
    ) -> Result<String, String> {
        match section_inputs.len() {
            0 => Ok(String::new()),
            1 => {
                let mut input_hash = Some(Self::hash_input(section_inputs[0]));
                self.execute_template_section(
                    section_inputs[0],
                    ops,
                    exec,
                    cache_key,
                    ExecutionContext {
                        input_hash: &mut input_hash,
                        cache,
                        dbg: None,
                    },
                )
            }
            _ => {
                let mut results = Vec::with_capacity(section_inputs.len());
                for input in section_inputs {
                    let mut input_hash = Some(Self::hash_input(input));
                    let result = self.execute_template_section(
                        input,
                        ops,
                        exec,
                        cache_key,
                        ExecutionContext {
                            input_hash: &mut input_hash,
                            cache,
                            dbg: None,
                        },
                    )?;
                    results.push(result);
                }
                Ok(results.join(separator))
            }
        }
    }

    fn execute_template_section(
        &self,
        input: &str,
        ops: &[StringOp],
        exec: &TemplateExecutionPlan,
        section_key: u64,
        ctx: ExecutionContext<'_>,
    ) -> Result<String, String> {
        match exec.cache_policy {
            CachePolicy::Never => {
                if let Some(t) = ctx.dbg {
                    t.cache_operation("DIRECT EXEC", "cache disabled for unique section");
                }
                self.execute_template_section_inner(input, ops, &exec.kind, ctx.dbg)
            }
            CachePolicy::PerCall => {
                let key = CacheKey {
                    input_hash: *ctx
                        .input_hash
                        .get_or_insert_with(|| Self::hash_input(input)),
                    section_key,
                };

                if let Some(cached) = ctx.cache.operations.get(&key) {
                    if let Some(t) = ctx.dbg {
                        t.cache_operation("CACHE HIT", "re-using formatted section");
                    }
                    return Ok(cached.clone());
                }

                if let Some(t) = ctx.dbg {
                    t.cache_operation("CACHE MISS", "computing section");
                }

                let out = self.execute_template_section_inner(input, ops, &exec.kind, ctx.dbg)?;
                ctx.cache.operations.insert(key, out.clone());
                Ok(out)
            }
        }
    }

    fn literal_preview(text: &str) -> String {
        if text.trim().is_empty() && text.len() <= 2 {
            "whitespace".to_string()
        } else if text.len() <= 20 {
            format!("'{text}'")
        } else {
            format!("'{}...' ({} chars)", &text[..15], text.len())
        }
    }

    fn execute_template_section_inner(
        &self,
        input: &str,
        ops: &[StringOp],
        kind: &TemplateExecutionKind,
        dbg: Option<&DebugTracer>,
    ) -> Result<String, String> {
        match kind {
            TemplateExecutionKind::Passthrough => {
                if let Some(t) = dbg {
                    t.cache_operation("FAST PASSTHROUGH", "empty template section");
                }
                Ok(input.to_string())
            }
            TemplateExecutionKind::SplitIndex { sep, idx } => {
                if let Some(t) = dbg {
                    t.cache_operation("FAST SPLIT", &format!("by '{sep}'"));
                }
                Ok(self.fast_split_index(input, sep, *idx))
            }
            TemplateExecutionKind::SplitJoinRewrite {
                split_sep,
                join_sep,
            } => {
                if let Some(t) = dbg {
                    t.cache_operation("FAST SPLIT+JOIN", "direct separator rewrite");
                }
                Ok(self.fast_split_join(input, split_sep, join_sep))
            }
            TemplateExecutionKind::Generic => {
                let nested_dbg = if self.debug {
                    Some(DebugTracer::new(true))
                } else {
                    None
                };
                apply_ops_internal(input, ops, self.debug, nested_dbg)
            }
        }
    }

    fn compile_sections(sections: &[TemplateSection]) -> Vec<CompiledSectionPlan> {
        let mut repeated_keys = HashSet::with_capacity(sections.len());
        let mut seen_keys = HashSet::with_capacity(sections.len());
        for section in sections {
            if let TemplateSection::Template { cache_key, .. } = section
                && !seen_keys.insert(*cache_key)
            {
                repeated_keys.insert(*cache_key);
            }
        }

        sections
            .iter()
            .map(|section| match section {
                TemplateSection::Literal(_) => CompiledSectionPlan::Literal,
                TemplateSection::Template { ops, cache_key } => CompiledSectionPlan::Template {
                    exec: TemplateExecutionPlan {
                        kind: Self::compile_template_execution_kind(ops),
                        cache_policy: if repeated_keys.contains(cache_key) {
                            CachePolicy::PerCall
                        } else {
                            CachePolicy::Never
                        },
                    },
                    cache_key: *cache_key,
                },
            })
            .collect()
    }

    fn compile_template_execution_kind(ops: &[StringOp]) -> TemplateExecutionKind {
        if ops.is_empty() {
            return TemplateExecutionKind::Passthrough;
        }

        if ops.len() == 1
            && let StringOp::Split {
                sep,
                range: RangeSpec::Index(idx),
            } = &ops[0]
        {
            return TemplateExecutionKind::SplitIndex {
                sep: sep.clone(),
                idx: *idx,
            };
        }

        if ops.len() == 2
            && let [
                StringOp::Split {
                    sep: split_sep,
                    range,
                },
                StringOp::Join { sep: join_sep },
            ] = ops
            && Self::is_full_range(range)
        {
            return TemplateExecutionKind::SplitJoinRewrite {
                split_sep: split_sep.clone(),
                join_sep: join_sep.clone(),
            };
        }

        TemplateExecutionKind::Generic
    }

    fn estimate_output_capacity(&self, input: &str) -> usize {
        self.sections
            .iter()
            .map(|section| match section {
                TemplateSection::Literal(text) => text.len(),
                TemplateSection::Template { .. } => 0,
            })
            .sum::<usize>()
            + input.len()
    }

    fn literal_output_capacity(&self) -> usize {
        self.sections
            .iter()
            .map(|section| match section {
                TemplateSection::Literal(text) => text.len(),
                TemplateSection::Template { .. } => 0,
            })
            .sum()
    }

    fn hash_input(input: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    #[inline]
    fn fast_split_join(&self, input: &str, split_sep: &str, join_sep: &str) -> String {
        if split_sep.is_empty() || split_sep == join_sep {
            return input.to_string();
        }

        if split_sep.len() == 1 {
            let split_byte = split_sep.as_bytes()[0];
            let estimated_len = if join_sep.len() == 1 {
                input.len()
            } else {
                let replacements = memchr_iter(split_byte, input.as_bytes()).count();
                input.len() + replacements.saturating_mul(join_sep.len().saturating_sub(1))
            };

            let mut result = String::with_capacity(estimated_len);
            let mut start = 0usize;
            for idx in memchr_iter(split_byte, input.as_bytes()) {
                result.push_str(&input[start..idx]);
                result.push_str(join_sep);
                start = idx + 1;
            }
            result.push_str(&input[start..]);
            result
        } else {
            input.replace(split_sep, join_sep)
        }
    }

    #[inline]
    fn fast_split_index(&self, input: &str, sep: &str, idx: isize) -> String {
        if sep.is_empty() {
            let parts = get_cached_split(input, sep);
            return apply_range(&parts, &RangeSpec::Index(idx))
                .into_iter()
                .next()
                .unwrap_or_default();
        }

        if sep.len() == 1 {
            let sep_byte = sep.as_bytes()[0];
            let parts_len = memchr_iter(sep_byte, input.as_bytes()).count() + 1;
            let resolved = Self::resolve_split_index(idx, parts_len);
            return Self::split_index_single_byte(input, sep_byte, resolved);
        }

        let parts_len = input.matches(sep).count() + 1;
        let resolved = Self::resolve_split_index(idx, parts_len);
        input
            .split(sep)
            .nth(resolved)
            .unwrap_or_default()
            .to_string()
    }

    #[inline]
    fn split_index_single_byte(input: &str, sep_byte: u8, target_idx: usize) -> String {
        let mut start = 0usize;

        for (current_idx, idx) in memchr_iter(sep_byte, input.as_bytes()).enumerate() {
            if current_idx == target_idx {
                return input[start..idx].to_string();
            }
            start = idx + 1;
        }

        input[start..].to_string()
    }

    #[inline]
    fn resolve_split_index(idx: isize, parts_len: usize) -> usize {
        let parts_len_i = parts_len as isize;
        let resolved = if idx < 0 { parts_len_i + idx } else { idx };
        resolved.clamp(0, parts_len_i.saturating_sub(1)) as usize
    }

    #[inline]
    fn is_full_range(range: &RangeSpec) -> bool {
        matches!(range, RangeSpec::Range(None, None, false))
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

    fn make_template_section(ops: Vec<StringOp>) -> TemplateSection {
        TemplateSection::from_ops(ops)
    }

    fn hash_ops(ops: &[StringOp]) -> u64 {
        let mut hasher = DefaultHasher::new();
        ops.hash(&mut hasher);
        hasher.finish()
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
        let sections = vec![Self::make_template_section(ops)];
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
