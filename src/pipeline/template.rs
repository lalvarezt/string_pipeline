use std::collections::{HashMap, hash_map::DefaultHasher};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::pipeline::{DebugContext, RangeSpec, StringOp, apply_ops_internal, apply_range, parser};

/// A compiled string transformation template with chainable operations.
///
/// A `Template` represents a parsed sequence of string operations that can be applied to
/// transform input strings. Templates are defined using an intuitive template syntax and
/// compiled for efficient reuse across multiple inputs.
///
/// The template system supports over 20 different operations including splitting, joining,
/// regex replacement, filtering, mapping, sorting, and more. Operations can be chained
/// together to create powerful text processing pipelines.
///
/// # Template Syntax
///
/// Templates are enclosed in `{}` and consist of operations separated by `|`:
///
/// ```text
/// {operation1|operation2|operation3}
/// ```
///
/// ## Supported Operations
///
/// | Operation | Syntax | Description |
/// |-----------|--------|-------------|
/// | **Split** | `split:<sep>:<range>` | Split by separator, select by index/range |
/// | **Join** | `join:<sep>` | Join list items with separator |
/// | **Substring** | `substring:<range>` | Extract substring by character index/range |
/// | **Replace** | `replace:s/<pattern>/<replacement>/<flags>` | Regex find/replace (sed-like) |
/// | **Upper/Lower** | `upper`, `lower` | Case conversion |
/// | **Trim** | `trim[:<chars>][:<direction>]` | Remove whitespace or custom characters |
/// | **Append/Prepend** | `append:<text>`, `prepend:<text>` | Add text to ends |
/// | **Map** | `map:{<operations>}` | Apply sub-pipeline to each list item |
/// | **Filter** | `filter:<regex>`, `filter_not:<regex>` | Keep/remove items matching pattern |
/// | **Sort** | `sort[:asc\|desc]` | Sort list items |
/// | **Slice** | `slice:<range>` | Select list elements by range |
/// | **Unique** | `unique` | Remove duplicate list items |
/// | **Reverse** | `reverse` | Reverse string or list |
/// | **Pad** | `pad:<width>[:<char>][:<direction>]` | Pad string to width |
/// | **Regex Extract** | `regex_extract:<pattern>[:<group>]` | Extract regex matches |
/// | **Strip ANSI** | `strip_ansi` | Remove ANSI escape sequences |
///
/// ## Range Specifications
///
/// Ranges support Rust-like syntax with negative indexing:
///
/// | Syntax | Description | Example Result |
/// |--------|-------------|----------------|
/// | `N` | Single index | `{split:,:1}` → second element |
/// | `N..M` | Exclusive range | `{split:,:1..3}` → elements 1,2 |
/// | `N..=M` | Inclusive range | `{split:,:1..=3}` → elements 1,2,3 |
/// | `N..` | From N to end | `{split:,:2..}` → from 2nd to end |
/// | `..M` | From start to M-1 | `{split:,:..3}` → first 3 elements |
/// | `..=M` | From start to M inclusive | `{split:,:..=2}` → first 3 elements |
/// | `..` | All elements | `{split:,:..}` → all elements |
///
/// Negative indices count from the end (`-1` = last, `-2` = second to last).
///
/// ## Shorthand Syntax
///
/// Split operations have convenient shorthand forms:
///
/// ```rust
/// use string_pipeline::Template;
///
/// // These are equivalent:
/// let t1 = Template::parse("{split: :1}").unwrap();
/// let t2 = Template::parse("{1}").unwrap();
///
/// let result = t1.format("a b c").unwrap();
/// assert_eq!(result, "b");
/// ```
///
/// ## Escaping
///
/// Special characters can be escaped in arguments:
///
/// - `\:` - Literal colon
/// - `\|` - Literal pipe
/// - `\\` - Literal backslash
/// - `\n` - Newline
/// - `\t` - Tab
/// - `\r` - Carriage return
///
/// Context-aware parsing allows pipes in regex patterns and sed replacements without escaping.
///
/// # Examples
///
/// ## Basic Text Processing
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Clean and normalize text
/// let cleaner = Template::parse("{trim|replace:s/\\s+/ /g|lower}").unwrap();
/// let result = cleaner.format("  Hello    WORLD  ").unwrap();
/// assert_eq!(result, "hello world");
/// ```
///
/// ## List Operations
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Split, filter, and rejoin
/// let filter_template = Template::parse("{split:,:..|filter:^[aeiou]|join:\\|}").unwrap();
/// let result = filter_template.format("apple,banana,orange,grape").unwrap();
/// assert_eq!(result, "apple|orange");
/// ```
///
/// ## Data Extraction
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Extract usernames from process list
/// let ps_parser = Template::parse("{split:\\n:1..|map:{split: :0}|unique|sort|join:\\n}").unwrap();
/// let ps_output = "USER   PID\nroot   123\nalice  456\nroot   789";
/// let result = ps_parser.format(ps_output).unwrap();
/// assert_eq!(result, "alice\nroot");
/// ```
///
/// ## Complex Processing Pipeline
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Process CSV-like data
/// let csv_processor = Template::parse(
///     "{split:\\n:1..|map:{split:,:0|trim|upper|prepend:USER_}|join:\\|}"
/// ).unwrap();
///
/// let csv = "name,age\nAlice,25\nBob,30";
/// let result = csv_processor.format(csv).unwrap();
/// assert_eq!(result, "USER_ALICE|USER_BOB");
/// ```
///
/// ## Debug Mode
///
/// Enable debug mode by adding `!` after the opening brace:
///
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("{!split:,:..}").unwrap();
/// // Outputs detailed operation traces to stderr
/// let result = template.format("a,b,c").unwrap();
/// assert_eq!(result, "a,b,c");
/// ```
///
/// # Error Handling
///
/// Templates can fail during parsing or execution:
///
/// ## Parse Errors
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Invalid syntax
/// assert!(Template::parse("{split:}").is_err());
/// assert!(Template::parse("{unknown_op}").is_err());
/// assert!(Template::parse("{split:,:|invalid_range}").is_err());
/// ```
///
/// ## Runtime Errors
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Invalid regex patterns fail at runtime
/// let template = Template::parse("{filter:[}").unwrap();
/// let result = template.format("test");
/// assert!(result.is_err());
///
/// // Operations on wrong data types
/// let template = Template::parse("{sort}").unwrap();
/// let result = template.format("not_a_list");
/// assert!(result.is_err());
/// ```
///
/// # Performance Considerations
///
/// - Templates are compiled once and can be reused efficiently
/// - Operations use zero-copy techniques where possible
/// - Large datasets are processed with optimized algorithms
/// - Regex patterns are compiled and cached internally
/// - Memory allocation is minimized for common operations
///
/// For high-throughput applications, compile templates once and reuse them:
///
/// ```rust
/// use string_pipeline::Template;
///
/// // Compile once
/// let template = Template::parse("{split:,:0}").unwrap();
///
/// // Reuse many times
/// for input in &["a,b,c", "x,y,z", "1,2,3"] {
///     let result = template.format(input).unwrap();
///     println!("{}", result);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Template {
    /// The original template string for display and debugging.
    raw: String,
    /// Compiled sequence of operations to apply.
    ops: Vec<StringOp>,
    /// Whether debug mode is enabled for detailed operation tracing.
    debug: bool,
}

impl Template {
    /// Creates a new Template with the given components.
    ///
    /// This is an internal constructor used by the parser. Use [`Template::parse`]
    /// to create templates from strings.
    ///
    /// # Arguments
    ///
    /// * `raw` - The original template string
    /// * `ops` - Compiled operations to execute
    /// * `debug` - Whether to enable debug output
    fn new(raw: String, ops: Vec<StringOp>, debug: bool) -> Self {
        Template { raw, ops, debug }
    }

    /// Parses a template string into a compiled `Template`.
    ///
    /// This method compiles the template syntax into an efficient sequence of operations
    /// that can be applied to multiple input strings. The template is validated during
    /// parsing to catch syntax errors early.
    ///
    /// # Arguments
    ///
    /// * `template` - A template string following the documented syntax
    ///
    /// # Returns
    ///
    /// * `Ok(Template)` - Successfully compiled template
    /// * `Err(String)` - Parse error with detailed description
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template syntax is invalid (malformed braces, missing arguments, etc.)
    /// - Unknown operations are used
    /// - Range specifications are malformed
    /// - Regex patterns are invalid
    /// - Operation arguments are missing or invalid
    ///
    /// # Examples
    ///
    /// ## Valid Templates
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Simple operations
    /// assert!(Template::parse("{upper}").is_ok());
    /// assert!(Template::parse("{split:,:..}").is_ok());
    /// assert!(Template::parse("{trim|upper|append:!}").is_ok());
    ///
    /// // Complex pipelines
    /// assert!(Template::parse("{split:,:..|map:{trim|upper}|join:-}").is_ok());
    ///
    /// // Debug mode
    /// assert!(Template::parse("{!split:,:..}").is_ok());
    /// ```
    ///
    /// ## Parse Errors
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Missing arguments
    /// assert!(Template::parse("{split:}").is_err());
    ///
    /// // Invalid syntax
    /// assert!(Template::parse("{split:,||}").is_err());
    /// assert!(Template::parse("no_braces").is_err());
    ///
    /// // Unknown operations
    /// assert!(Template::parse("{unknown_operation}").is_err());
    ///
    /// // Invalid ranges
    /// assert!(Template::parse("{split:,:abc}").is_err());
    /// assert!(Template::parse("{1..abc}").is_err());
    /// ```
    pub fn parse(template: &str) -> Result<Self, String> {
        match parser::parse_template(template) {
            Ok((ops, debug)) => Ok(Template::new(template.to_string(), ops, debug)),
            Err(e) => Err(e),
        }
    }

    /// Applies the template operations to the input string.
    ///
    /// This method executes the compiled operation sequence on the provided input,
    /// transforming it according to the template definition. Operations are applied
    /// in the order specified in the template.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to transform
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The transformed result
    /// * `Err(String)` - Runtime error with description
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Regex patterns fail to compile or match
    /// - Operations are applied to incompatible data types
    /// - Index/range operations go out of bounds (handled gracefully)
    /// - System operations fail (e.g., memory allocation)
    ///
    /// # Examples
    ///
    /// ## Successful Operations
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{split:,:0..2|join: and }").unwrap();
    /// let result = template.format("apple,banana,cherry,date").unwrap();
    /// assert_eq!(result, "apple and banana");
    /// ```
    ///
    /// ## Runtime Errors
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Operations on wrong types
    /// let template = Template::parse("{sort}").unwrap();
    /// let result = template.format("single_string");
    /// assert!(result.is_err());
    ///
    /// // Invalid regex (caught at runtime)
    /// let template_with_bad_regex = Template::parse("{filter:[}").unwrap();
    /// let invalid_regex_result = template_with_bad_regex.format("test");
    /// assert!(invalid_regex_result.is_err());
    /// ```
    ///
    /// ## Edge Cases
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{split:,:..}").unwrap();
    ///
    /// // Empty input
    /// assert_eq!(template.format("").unwrap(), "");
    ///
    /// // Single item
    /// assert_eq!(template.format("single").unwrap(), "single");
    ///
    /// // Out of bounds access (handled gracefully)
    /// let template = Template::parse("{split:,:10}").unwrap();
    /// assert_eq!(template.format("a,b").unwrap(), "b"); // Clamps to last item
    /// ```
    pub fn format(&self, input: &str) -> Result<String, String> {
        if self.debug {
            let debug_context = DebugContext::new_template(true, self.raw.clone());
            debug_context.print_template_header("SINGLE TEMPLATE", input, None);

            let result =
                apply_ops_internal(input, &self.ops, self.debug, Some(debug_context.clone()))?;

            // Show consistent cache statistics format
            use crate::pipeline::{REGEX_CACHE, SPLIT_CACHE};
            let regex_cache = REGEX_CACHE.lock().unwrap();
            let split_cache = SPLIT_CACHE.lock().unwrap();
            let cache_info = format!(
                "Cache stats: {} regex patterns, {} split operations cached",
                regex_cache.len(),
                split_cache.len()
            );
            debug_context.print_template_footer("SINGLE TEMPLATE", &result, Some(&cache_info));
            Ok(result)
        } else {
            apply_ops_internal(input, &self.ops, false, None)
        }
    }

    /// Returns the original template string.
    ///
    /// This method provides access to the raw template string used to create
    /// this Template instance, useful for debugging and logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{upper|trim}").unwrap();
    /// assert_eq!(template.template_string(), "{upper|trim}");
    /// ```
    pub fn template_string(&self) -> &str {
        &self.raw
    }

    /// Returns whether debug mode is enabled for this template.
    ///
    /// Debug mode can be enabled by adding `!` after the opening brace in the template.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let normal = Template::parse("{upper}").unwrap();
    /// assert!(!normal.is_debug_enabled());
    ///
    /// let debug = Template::parse("{!upper}").unwrap();
    /// assert!(debug.is_debug_enabled());
    /// ```
    pub fn is_debug_enabled(&self) -> bool {
        self.debug
    }

    /// Returns the number of operations in this template.
    ///
    /// This can be useful for performance analysis or debugging complex templates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let simple = Template::parse("{upper}").unwrap();
    /// assert_eq!(simple.operation_count(), 1);
    ///
    /// let complex = Template::parse("{trim|upper|split:,:..}").unwrap();
    /// assert_eq!(complex.operation_count(), 3);
    /// ```
    pub fn operation_count(&self) -> usize {
        self.ops.len()
    }
}

impl TryFrom<&str> for Template {
    type Error = String;

    /// Attempts to parse a template from a string reference.
    ///
    /// This provides a convenient way to convert string literals directly into templates
    /// using the `try_into()` method or `Template::try_from()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    /// use std::convert::TryFrom;
    ///
    /// let template = Template::try_from("{upper}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "HELLO");
    ///
    /// // Or using try_into()
    /// let template: Template = "{lower}".try_into().unwrap();
    /// assert_eq!(template.format("HELLO").unwrap(), "hello");
    /// ```
    fn try_from(template: &str) -> Result<Self, Self::Error> {
        Template::parse(template)
    }
}

impl Display for Template {
    /// Formats the template for display.
    ///
    /// Returns the original template string, making it easy to print or log templates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{upper|trim}").unwrap();
    /// println!("Using template: {}", template); // Prints: Using template: {upper|trim}
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// A compiled multi-template string processor that handles mixed text and template sections with caching.
///
/// A `MultiTemplate` represents a parsed string containing both literal text and template sections,
/// supporting efficient caching of intermediate results to avoid redundant computations.
///
/// # Multi-Template Syntax
///
/// Multi-templates consist of literal text mixed with template sections:
///
/// ```text
/// some literal text {operation1|operation2} more text {operation3}
/// ```
///
/// # Caching Benefits
///
/// When the same operation appears multiple times in a multi-template, intermediate results
/// are cached to avoid redundant computations:
///
/// ```rust
/// use string_pipeline::MultiTemplate;
///
/// // The split operation will only be performed once, cached for reuse
/// let template = MultiTemplate::parse("First: {split:,:0} Second: {split:,:1}").unwrap();
/// let result = template.format("apple,banana,cherry").unwrap();
/// assert_eq!(result, "First: apple Second: banana");
/// ```
#[derive(Debug, Clone)]
pub struct MultiTemplate {
    /// The original template string for display and debugging.
    raw: String,
    /// Parsed sections containing literal text and template operations.
    sections: Vec<TemplateSection>,
    /// Whether debug mode is enabled for detailed operation tracing.
    debug: bool,
}

/// A section within a multi-template, either literal text or a template operation.
#[derive(Debug, Clone)]
pub enum TemplateSection {
    /// Literal text to include as-is in the output.
    Literal(String),
    /// A template operation sequence to apply to the input.
    Template(Vec<StringOp>),
}

/// Unified cache system for MultiTemplate operations
struct TemplateCache {
    operations: HashMap<CacheKey, String>,
    splits: HashMap<SplitCacheKey, Vec<String>>,
}

impl TemplateCache {
    fn new() -> Self {
        Self {
            operations: HashMap::new(),
            splits: HashMap::new(),
        }
    }

    fn stats(&self) -> String {
        format!(
            "Operations: {}, Splits: {}",
            self.operations.len(),
            self.splits.len()
        )
    }
}

/// Cache key for memoizing split operation results.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SplitCacheKey {
    input_hash: u64,
    separator: String,
}

/// Cache key for memoizing complete operation results.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    input_hash: u64,
    ops_signature: String,
}

impl MultiTemplate {
    /// Creates a new MultiTemplate with the specified sections and debug flag.
    fn new(raw: String, sections: Vec<TemplateSection>, debug: bool) -> Self {
        Self {
            raw,
            sections,
            debug,
        }
    }

    /// Format operations summary for debug output
    fn format_operations_summary(ops: &[StringOp]) -> String {
        ops.iter()
            .map(|op| match op {
                StringOp::Split { sep, range } => format!(
                    "split('{}',{})",
                    sep,
                    match range {
                        RangeSpec::Index(i) => format!("{}", i),
                        RangeSpec::Range(start, end, inclusive) => {
                            match (start, end) {
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
                            }
                        }
                    }
                ),
                StringOp::Upper => "upper".to_string(),
                StringOp::Lower => "lower".to_string(),
                StringOp::Append { suffix } => format!("append('{}')", suffix),
                StringOp::Prepend { prefix } => format!("prepend('{}')", prefix),
                StringOp::Replace {
                    pattern,
                    replacement,
                    ..
                } => format!("replace('{}' -> '{}')", pattern, replacement),
                _ => format!("{:?}", op).to_lowercase(),
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Parses a multi-template string into a reusable MultiTemplate.
    ///
    /// # Arguments
    ///
    /// * `template` - The multi-template string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(MultiTemplate)` - Successfully parsed multi-template
    /// * `Err(String)` - Parse error with detailed description
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::MultiTemplate;
    ///
    /// // Parse a multi-template with literal text and operations
    /// let template = MultiTemplate::parse("Name: {split: :0} Age: {split: :1}").unwrap();
    /// let result = template.format("John 25").unwrap();
    /// assert_eq!(result, "Name: John Age: 25");
    ///
    /// // Templates with the same operation will be cached
    /// let template = MultiTemplate::parse("A: {split:,:0} B: {split:,:1} C: {split:,:0}").unwrap();
    /// let result = template.format("x,y,z").unwrap();
    /// assert_eq!(result, "A: x B: y C: x");
    /// ```
    pub fn parse(template: &str) -> Result<Self, String> {
        let (sections, debug) = parser::parse_multi_template(template)?;
        Ok(Self::new(template.to_string(), sections, debug))
    }

    /// Applies the multi-template to an input string with optimized caching.
    ///
    /// This implementation caches split operations separately from index selection
    /// to avoid redundant splitting when accessing different indices from the same split.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to transform
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The transformed result
    /// * `Err(String)` - Processing error description
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::MultiTemplate;
    ///
    /// let template = MultiTemplate::parse("Start {upper} End").unwrap();
    /// let result = template.format("hello").unwrap();
    /// assert_eq!(result, "Start HELLO End");
    /// ```
    pub fn format(&self, input: &str) -> Result<String, String> {
        use std::time::Instant;

        let mut cache = TemplateCache::new();
        let mut result = String::new();

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let input_hash = hasher.finish();

        let start_time = if self.debug {
            Some(Instant::now())
        } else {
            None
        };

        if self.debug {
            let debug_context = DebugContext::new_template(true, self.raw.clone());
            let additional_info = format!(
                "{} sections to process (literal: {}, template: {})",
                self.sections.len(),
                self.sections.len() - self.template_section_count(),
                self.template_section_count()
            );
            debug_context.print_template_header("MULTI-TEMPLATE", input, Some(&additional_info));

            for (i, section) in self.sections.iter().enumerate() {
                match section {
                    TemplateSection::Literal(text) => {
                        let content = if text.trim().is_empty() && text.len() <= 2 {
                            "whitespace".to_string()
                        } else if text.len() <= 20 {
                            format!("'{}'", text)
                        } else {
                            format!("'{}...' ({} chars)", &text[..15], text.len())
                        };
                        debug_context.print_section(
                            i + 1,
                            self.sections.len(),
                            "literal",
                            &content,
                        );
                        result.push_str(text);
                    }
                    TemplateSection::Template(ops) => {
                        let ops_summary = Self::format_operations_summary(ops);
                        debug_context.print_section(
                            i + 1,
                            self.sections.len(),
                            "template",
                            &ops_summary,
                        );

                        let section_result = self.apply_template_section_optimized(
                            input,
                            ops,
                            input_hash,
                            &mut cache,
                            &Some(&debug_context),
                        )?;
                        result.push_str(&section_result);
                        debug_context.print_result("", &section_result);
                    }
                }
            }

            let total_elapsed = start_time.unwrap().elapsed();
            let cache_info = format!(
                "Total execution time: {:?}, Cache stats - {}",
                total_elapsed,
                cache.stats()
            );
            debug_context.print_template_footer("MULTI-TEMPLATE", &result, Some(&cache_info));
        } else {
            for section in &self.sections {
                match section {
                    TemplateSection::Literal(text) => {
                        result.push_str(text);
                    }
                    TemplateSection::Template(ops) => {
                        let section_result = self.apply_template_section_optimized(
                            input, ops, input_hash, &mut cache, &None,
                        )?;
                        result.push_str(&section_result);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Apply a template section with optimized caching for split operations.
    fn apply_template_section_optimized(
        &self,
        input: &str,
        ops: &[StringOp],
        input_hash: u64,
        cache: &mut TemplateCache,
        debug_context: &Option<&DebugContext>,
    ) -> Result<String, String> {
        // Check if this is a simple split+index operation that can be optimized
        if ops.len() == 1 {
            if let StringOp::Split { sep, range } = &ops[0] {
                return self.apply_optimized_split(
                    input,
                    sep,
                    range,
                    input_hash,
                    &mut cache.splits,
                    debug_context,
                );
            }
        }

        // Fall back to regular operation caching for complex operations
        let ops_signature = format!("{:?}", ops);
        let cache_key = CacheKey {
            input_hash,
            ops_signature: ops_signature.clone(),
        };

        if let Some(cached) = cache.operations.get(&cache_key) {
            if let Some(ctx) = debug_context {
                ctx.print_cache_operation("CACHE HIT", "Reusing previous result");
            }
            Ok(cached.clone())
        } else {
            if let Some(ctx) = debug_context {
                ctx.print_cache_operation("CACHE MISS", "Computing and storing result");
            }
            // Keep verbose debug for apply_ops_internal to show detailed pipeline execution
            let section_result = apply_ops_internal(
                input,
                ops,
                self.debug,
                debug_context.as_ref().map(|ctx| (*ctx).clone()),
            )?;
            cache.operations.insert(cache_key, section_result.clone());
            Ok(section_result)
        }
    }

    /// Apply an optimized split operation with separate caching for split results.
    fn apply_optimized_split(
        &self,
        input: &str,
        separator: &str,
        range: &RangeSpec,
        input_hash: u64,
        split_cache: &mut HashMap<SplitCacheKey, Vec<String>>,
        debug_context: &Option<&DebugContext>,
    ) -> Result<String, String> {
        let split_key = SplitCacheKey {
            input_hash,
            separator: separator.to_string(),
        };

        // Get or compute the split result
        let split_result = if let Some(cached_split) = split_cache.get(&split_key) {
            if let Some(ctx) = debug_context {
                ctx.print_cache_operation(
                    "SPLIT CACHE HIT",
                    &format!("Reusing split by '{}'", separator),
                );
            }
            cached_split.clone()
        } else {
            if let Some(ctx) = debug_context {
                ctx.print_cache_operation(
                    "SPLIT CACHE MISS",
                    &format!("Computing split by '{}'", separator),
                );
            }
            let parts: Vec<String> = if separator.is_empty() {
                input.chars().map(|c| c.to_string()).collect()
            } else {
                input.split(separator).map(|s| s.to_string()).collect()
            };
            split_cache.insert(split_key, parts.clone());
            parts
        };

        // Apply the range selection to the cached split result
        let selected = apply_range(&split_result, range);

        if let Some(ctx) = debug_context {
            let range_desc = match range {
                RangeSpec::Index(i) => format!("item[{}]", i),
                RangeSpec::Range(start, end, inclusive) => {
                    let start_str = start
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "start".to_string());
                    let end_str = end
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "end".to_string());
                    let op = if *inclusive { "..=" } else { ".." };
                    format!("range[{}{}{}]", start_str, op, end_str)
                }
            };
            ctx.print_step(&format!(
                "Selecting {} from {} parts",
                range_desc,
                split_result.len()
            ));
        }

        // Convert back to the expected format (join with same separator if multiple items)
        match selected.len() {
            0 => Ok(String::new()),
            1 => Ok(selected[0].clone()),
            _ => {
                // For multiple items, this should be rare in split+index operations
                // but we handle it by joining with the original separator
                Ok(selected.join(separator))
            }
        }
    }

    /// Returns the original multi-template string.
    pub fn template_string(&self) -> &str {
        &self.raw
    }

    /// Returns whether debug mode is enabled.
    pub fn is_debug_enabled(&self) -> bool {
        self.debug
    }

    /// Returns the number of sections in this multi-template.
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Returns the number of template sections (excluding literal text).
    pub fn template_section_count(&self) -> usize {
        self.sections
            .iter()
            .filter(|s| matches!(s, TemplateSection::Template(_)))
            .count()
    }
}

impl TryFrom<&str> for MultiTemplate {
    type Error = String;

    /// Creates a MultiTemplate from a string slice.
    ///
    /// This is equivalent to calling `MultiTemplate::parse()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::MultiTemplate;
    /// use std::convert::TryInto;
    ///
    /// let template: MultiTemplate = "Hello {upper}!".try_into().unwrap();
    /// assert_eq!(template.format("world").unwrap(), "Hello WORLD!");
    /// ```
    fn try_from(template: &str) -> Result<Self, Self::Error> {
        Self::parse(template)
    }
}

impl Display for MultiTemplate {
    /// Formats the MultiTemplate for display, showing the original template string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::MultiTemplate;
    ///
    /// let template = MultiTemplate::parse("Hello {upper}!").unwrap();
    /// println!("{}", template); // Output: "Hello {upper}!"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[cfg(test)]
mod multi_template_tests {
    use super::*;

    #[test]
    fn test_multi_template_basic() {
        let template = MultiTemplate::parse("Hello {upper} world!").unwrap();
        let result = template.format("test").unwrap();
        assert_eq!(result, "Hello TEST world!");
    }

    #[test]
    fn test_multi_template_multiple_operations() {
        let template = MultiTemplate::parse("First: {split:,:0} Second: {split:,:1}").unwrap();
        let result = template.format("apple,banana,cherry").unwrap();
        assert_eq!(result, "First: apple Second: banana");
    }

    #[test]
    fn test_multi_template_caching() {
        // This should use the same split operation twice, demonstrating caching
        let template =
            MultiTemplate::parse("A: {split:,:0} B: {split:,:1} C: {split:,:0}").unwrap();
        let result = template.format("x,y,z").unwrap();
        assert_eq!(result, "A: x B: y C: x");
    }

    #[test]
    fn test_multi_template_no_templates() {
        let template = MultiTemplate::parse("Just literal text").unwrap();
        let result = template.format("anything").unwrap();
        assert_eq!(result, "Just literal text");
    }

    #[test]
    fn test_multi_template_only_template() {
        let template = MultiTemplate::parse("{upper}").unwrap();
        let result = template.format("hello").unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_multi_template_complex_example() {
        // Example from the user's request
        let template =
            MultiTemplate::parse("some string {split:,:1} some string {split:,:2}").unwrap();
        let result = template.format("a,b,c,d").unwrap();
        assert_eq!(result, "some string b some string c");
    }

    #[test]
    fn test_multi_template_debug_mode() {
        let template = MultiTemplate::parse("Test {!upper} mode").unwrap();
        assert!(template.is_debug_enabled());
        let result = template.format("hello").unwrap();
        assert_eq!(result, "Test HELLO mode");
    }

    #[test]
    fn test_multi_template_nested_braces() {
        let template = MultiTemplate::parse("Result: {split:,:..|map:{upper}|join:,}").unwrap();
        let result = template.format("hello,world").unwrap();
        assert_eq!(result, "Result: HELLO,WORLD");
    }

    #[test]
    fn test_multi_template_error_unclosed_brace() {
        let result = MultiTemplate::parse("Hello {upper world");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unclosed template brace"));
    }

    #[test]
    fn test_multi_template_section_counts() {
        let template = MultiTemplate::parse("A {upper} B {lower} C").unwrap();
        assert_eq!(template.section_count(), 5); // "A ", upper, " B ", lower, " C"
        assert_eq!(template.template_section_count(), 2); // upper and lower
    }

    #[test]
    fn test_multi_template_split_optimization() {
        // This test verifies that multiple split operations with the same separator
        // reuse the cached split result rather than splitting multiple times
        let template = MultiTemplate::parse("A: {0} B: {1} C: {2} D: {0}").unwrap();
        let result = template.format("apple banana cherry").unwrap();
        assert_eq!(result, "A: apple B: banana C: cherry D: apple");

        // Test with comma-separated data
        let template = MultiTemplate::parse("{split:,:0}-{split:,:1}-{split:,:0}").unwrap();
        let result = template.format("x,y,z").unwrap();
        assert_eq!(result, "x-y-x");
    }

    #[test]
    fn test_multi_template_mixed_operations() {
        // Test that split optimization doesn't interfere with other operations
        let template = MultiTemplate::parse("First: {0} Upper: {upper} Last: {2}").unwrap();
        let result = template.format("hello world test").unwrap();
        assert_eq!(result, "First: hello Upper: HELLO WORLD TEST Last: test");
    }
}
