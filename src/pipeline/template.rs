use std::fmt::Display;

use crate::pipeline::{StringOp, apply_ops, parser};

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
#[derive(Debug)]
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
        apply_ops(input, &self.ops, self.debug)
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
