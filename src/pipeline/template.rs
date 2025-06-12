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
//! let template = Template::parse("Hello {upper}!", None).unwrap();
//! assert_eq!(template.format("world").unwrap(), "Hello WORLD!");
//!
//! // Multiple template sections
//! let template = Template::parse("Name: {split: :0} | Email: {split: :1}", None).unwrap();
//! assert_eq!(template.format("john doe john@example.com").unwrap(),
//!            "Name: john | Email: doe");
//!
//! // Complex transformations
//! let template = Template::parse("Files: {split:,:..|filter:\\.txt$|join:, }", None).unwrap();
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
/// let template = Template::parse("Result: {upper|trim}", None).unwrap();
/// assert_eq!(template.format("  hello  ").unwrap(), "Result: HELLO");
/// ```
///
/// ## Multi-Section Templates
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("User: {split: :0} ({split: :1})", None).unwrap();
/// assert_eq!(template.format("john smith").unwrap(), "User: john (smith)");
/// ```
///
/// ## Debug Mode
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("{split:,:..|sort|join: \\| }", Some(true)).unwrap();
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
    /// let template = Template::parse("Hello {upper}!", None).unwrap();
    ///
    /// // Complex pipeline
    /// let template = Template::parse("{split:,:..|sort|join: - }", None).unwrap();
    ///
    /// // Debug enabled
    /// let template = Template::parse("{!upper|trim}", None).unwrap();
    ///
    /// // Debug override
    /// let template = Template::parse("{upper}", Some(true)).unwrap();
    /// ```
    pub fn parse(template: &str, debug: Option<bool>) -> Result<Self, String> {
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
    /// let template = Template::parse("Name: {split: :0}, Age: {split: :1}", None).unwrap();
    /// let result = template.format("John 25").unwrap();
    /// assert_eq!(result, "Name: John, Age: 25");
    ///
    /// // List processing
    /// let template = Template::parse("Items: {split:,:..|sort|join: \\| }", None).unwrap();
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
    /// let template = Template::parse("Hello {upper}!", None).unwrap();
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
    /// let template = Template::parse("Hello {upper} world!", None).unwrap();
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
    /// let template = Template::parse("Hello {upper} world {lower}!", None).unwrap();
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
    /// let template = Template::parse("{upper}", Some(true)).unwrap();
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
    /// let mut template = Template::parse("{upper}", None).unwrap();
    /// template.set_debug(true);
    /// assert!(template.is_debug());
    /// ```
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
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
        if ops.len() == 1 {
            if let StringOp::Split { sep, range } = &ops[0] {
                if let Some(t) = dbg {
                    t.cache_operation("FAST SPLIT", &format!("by '{sep}'"));
                }
                return Ok(self.fast_single_split(input, sep, range));
            }
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
}

/* ---------- trait impls -------------------------------------------------- */

/// Provides convenient string-to-template conversion.
///
/// Enables creating templates directly from string literals using the `try_from` method,
/// with default settings (no debug mode override).
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::try_from("Hello {upper}!").unwrap();
/// assert_eq!(template.format("world").unwrap(), "Hello WORLD!");
/// ```
impl TryFrom<&str> for MultiTemplate {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s, None)
    }
}

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
/// let template = Template::parse("Hello {upper}!", None).unwrap();
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
/// let template1 = Template::parse("{upper}", None).unwrap();
/// let template2 = MultiTemplate::parse("{upper}", None).unwrap();
/// ```
pub type Template = MultiTemplate;
