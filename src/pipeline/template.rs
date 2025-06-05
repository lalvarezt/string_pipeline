use std::fmt::Display;

use crate::pipeline::{StringOp, apply_ops, parser};

/// A `Template` represents a string template with operations that can be applied to format input
/// strings.
///
/// It allows defining a sequence of operations to transform input strings, such as splitting,
/// joining, replacing, trimming, and more. The template is parsed from a string format that
/// specifies the operations in a concise syntax.
///
/// The template syntax supports a variety of operations, including:
/// - **Split**
/// - **Join**
/// - **Substring extraction**
/// - **Sed-like replacement using regex**
/// - **Uppercase and lowercase conversion**
/// - **Trimming whitespace or custom characters**
/// - **Appending or prepending text**
/// - **Map (apply sub-pipeline to each list item)**
/// - **Sort, reverse, unique, pad, regex_extract**
///
/// A `Template` can be created by parsing a string that follows the defined syntax (see
/// `Template::parse`), and it can then be used to format input strings by applying the specified
/// operations in sequence.
///
/// # Example
/// Trim, split and append a suffix to each resulting item:
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("{split:,:..|map:{trim|append:!}}").unwrap();
///
/// let result = template.format(" a, b,c , d , e ").unwrap();
///
/// assert_eq!(result, "a!,b!,c!,d!,e!");
/// ```
///
/// # Example: Map, Sort, Pad, Regex Extract
/// ```rust
/// use string_pipeline::Template;
///
/// let template = Template::parse("{split:,:..|map:{upper}|sort:desc|join:-}").unwrap();
/// let result = template.format("a,b,c").unwrap();
/// assert_eq!(result, "C-B-A");
///
/// let template = Template::parse("{split:,:..|map:{pad:3:*:both}}").unwrap();
/// let result = template.format("a,bb,c").unwrap();
/// assert_eq!(result, "*a*,bb*,*c*");
///
/// let template = Template::parse("{split:,:..|map:{regex_extract:\\d+}}").unwrap();
/// let result = template.format("a1,b22,c333").unwrap();
/// assert_eq!(result, "1,22,333");
/// ```
#[derive(Debug)]
pub struct Template {
    /// The raw template string.
    raw: String,
    /// A series of string operations to apply to the target string.
    ops: Vec<StringOp>,
    /// Whether to enable debug mode, which provides additional output for debugging purposes.
    debug: bool,
}

impl Template {
    fn new(raw: String, ops: Vec<StringOp>, debug: bool) -> Self {
        Template { raw, ops, debug }
    }

    /// Attempts to Parse a template string into a `Template` object.
    ///
    /// Templates are enclosed in `{}` and consist of a chain of operations separated by `|`.
    /// Arguments to operations are separated by `:`.
    ///
    /// # Syntax Reference
    ///
    /// - **Template**: `{ [!] operation_list? }`
    ///   - Add `!` after `{` to enable debug mode.
    /// - **Operation List**: `operation ('|' operation)*`
    /// - **Operation**:
    ///   - `split:<sep>:<range>`
    ///     - **Shorthand for split**:
    ///       - `{index}` (e.g. `{1}`, equivalent to `{split: :1}`)
    ///       - `{range}` (e.g. `{1..3}`, equivalent to `{split: :1..3}`)
    ///   - `join:<sep>`
    ///   - `substring:<range>`
    ///   - `replace:s/<pattern>/<replacement>/<flags>`
    ///   - `upper`
    ///   - `lower`
    ///   - `trim[:<chars>][:left|right|both]`
    ///   - `append:<suffix>`
    ///   - `prepend:<prefix>`
    ///   - `strip_ansi`
    ///   - `filter:<regex_pattern>`
    ///   - `filter_not:<regex_pattern>`
    ///   - `slice:<range>`
    ///   - `map:{<operation_list>}`
    ///   - `sort[:asc|desc]`
    ///   - `reverse`
    ///   - `unique`
    ///   - `pad:<width>[:<char>][:left|right|both]`
    ///   - `regex_extract:<pattern>[:<group>]`
    ///
    /// ## Supported Operations
    ///
    /// | Operation    | Syntax                                      | Description                                         |
    /// | ------------ | ------------------------------------------- | --------------------------------------------------- |
    /// | Split        | `split:<sep>:<range>`                       | Split by separator, select by index/range           |
    /// | Join         | `join:<sep>`                                | Join a list with separator                          |
    /// | Substring    | `substring:<range>`                         | Extract substring(s) by character index/range       |
    /// | Replace      | `replace:s/<pattern>/<replacement>/<flags>` | Regex replace (sed-like, supports flags)            |
    /// | Uppercase    | `upper`                                     | Convert to uppercase                                |
    /// | Lowercase    | `lower`                                     | Convert to lowercase                                |
    /// | Trim         | `trim[:<chars>][:left|right|both]`          | Trim whitespace (or side-specific)                  |
    /// | Append       | `append:<suffix>`                           | Append text                                         |
    /// | Prepend      | `prepend:<prefix>`                          | Prepend text                                        |
    /// | StripAnsi    | `strip_ansi`                                | Remove ANSI escape sequences                        |
    /// | Filter       | `filter:<regex_pattern>`                    | Keep only items matching regex pattern              |
    /// | FilterNot    | `filter_not:<regex_pattern>`                | Remove items matching regex pattern                 |
    /// | Slice        | `slice:<range>`                             | Select elements from a list by index/range          |
    /// | Map          | `map:{<operation_list>}`                    | Apply a sub-pipeline to each list item              |
    /// | Sort         | `sort[:asc|desc]`                           | Sort list ascending/descending                      |
    /// | Reverse      | `reverse`                                   | Reverse string or list                              |
    /// | Unique       | `unique`                                    | Remove duplicate items from a list                  |
    /// | Pad          | `pad:<width>[:<char>][:left|right|both]`    | Pad string/list items to width with char/side       |
    /// | RegexExtract | `regex_extract:<pattern>[:<group>]`         | Extract first match or group from string/list items |
    ///
    ///
    /// ## Range Specifications
    ///
    /// Ranges use Rust-like syntax and support negative indices like Python:
    ///
    /// | Range   | Description               | Example                              |
    /// | ------- | ------------------------- | ------------------------------------ |
    /// | `N`     | Single index              | `{split:,:1}`     → second element   |
    /// | `N..M`  | Exclusive range           | `{split:,:1..3}`  → elements 1,2     |
    /// | `N..=M` | Inclusive range           | `{split:,:1..=3}` → elements 1,2,3   |
    /// | `N..`   | From N to end             | `{split:,:2..}`   → from 2nd to end  |
    /// | `..N`   | From start to N           | `{split:,:..3}`   → first 3 elements |
    /// | `..=N`  | From start to N inclusive | `{split:,:..=2}`  → first 3 elements |
    /// | `..`    | All elements              | `{split:,:..}`    → all elements     |
    ///
    /// Negative indices count from the end:
    ///
    /// - `-1` = last element
    /// - `-2` = second to last element
    /// - `-3..` = last 3 elements
    ///
    /// ## Escaping
    ///
    /// The parser intelligently handles pipe characters (`|`) based on context:
    ///
    /// **Pipes are automatically allowed in:**
    ///
    /// - **Split separators**: `{split:|:..}` (splits on pipe)
    /// - **Regex patterns**: `{filter:\.(txt|md|log)}` (alternation)
    /// - **Sed replacements**: `{replace:s/test/a|b/}` (pipe in replacement)
    ///
    /// **Manual escaping needed for:**
    ///
    /// - **Other arguments**: Use `\|` for literal pipes in join, append, prepend, etc.
    /// - **Special characters**: Use `\:` for literal colons, `\\` for backslashes
    /// - **Escape sequences**: Use `\n`, `\t`, `\r` for newline, tab, carriage return
    ///
    /// ## Enable Debug Mode
    ///
    /// - Add `!` after `{` to enable debug output for each operation:
    /// - Example: `{!split:,:..|upper|join:-}`
    pub fn parse(template: &str) -> Result<Self, String> {
        match parser::parse_template(template) {
            Ok((ops, debug)) => Ok(Template::new(template.to_string(), ops, debug)),
            Err(e) => Err(e),
        }
    }

    /// Formats the input string using the operations defined in the template.
    ///
    /// # Example
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Create a template that splits a string by commas, takes the first two items, and joins
    /// // them with " and "
    /// let template = Template::parse("{split:,:0..2|join: and }").unwrap();
    ///
    /// // Format a string using the template
    /// let result = template.format("a,b,c,d").unwrap();
    ///
    /// assert_eq!(result, "a and b");
    /// ```
    pub fn format(&self, input: &str) -> Result<String, String> {
        apply_ops(input, &self.ops, self.debug)
    }
}

impl TryFrom<&str> for Template {
    type Error = String;

    fn try_from(template: &str) -> Result<Self, Self::Error> {
        Template::parse(template)
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
