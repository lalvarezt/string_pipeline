//! String transformation pipeline implementation.
//!
//! This module contains the core implementation of the string pipeline system,
//! including operation definitions, execution engine, and supporting types.
//!
//! The pipeline system processes strings through a sequence of operations,
//! supporting both individual string transformations and list-based operations
//! with efficient memory management and comprehensive error handling.

use regex::Regex;
mod debug;
mod parser;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use strip_ansi_escapes::strip;

pub use crate::pipeline::template::{MultiTemplate, Template};
pub use debug::{DebugContext, DebugSession};
mod template;

// Global regex cache to avoid recompiling patterns
static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Split operation cache to avoid re-splitting the same input with same separator
type SplitCacheKey = (u64, String);
type SplitCacheValue = Vec<String>;
type SplitCacheMap = HashMap<SplitCacheKey, SplitCacheValue>;
static SPLIT_CACHE: Lazy<Mutex<SplitCacheMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

// String interning for common separators to reduce allocations
static COMMON_SEPARATORS: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(" ", " ".to_string());
    map.insert(",", ",".to_string());
    map.insert("\n", "\n".to_string());
    map.insert("\t", "\t".to_string());
    map.insert(":", ":".to_string());
    map.insert(";", ";".to_string());
    map.insert("|", "|".to_string());
    map.insert("-", "-".to_string());
    map.insert("_", "_".to_string());
    map.insert("", "".to_string());
    map
});

/// Get an interned string for common separators, or clone for uncommon ones.
fn get_interned_separator(sep: &str) -> String {
    COMMON_SEPARATORS
        .get(sep)
        .cloned()
        .unwrap_or_else(|| sep.to_string())
}

/// Fast ASCII-only whitespace trimming
fn ascii_trim(s: &str) -> Option<&str> {
    if s.is_ascii() {
        Some(s.trim_matches(|c: char| c.is_ascii_whitespace()))
    } else {
        None
    }
}

/// Fast ASCII-only string reversal
fn ascii_reverse(s: &str) -> Option<String> {
    if s.is_ascii() {
        // For ASCII, we can safely reverse bytes
        let mut bytes: Vec<u8> = s.bytes().collect();
        bytes.reverse();
        // Safety: ASCII input guarantees valid UTF-8 output
        Some(unsafe { String::from_utf8_unchecked(bytes) })
    } else {
        None
    }
}

/// Get a compiled regex from cache or compile and cache it.
fn get_cached_regex(pattern: &str) -> Result<Regex, String> {
    // Try to get from cache first
    {
        let cache = REGEX_CACHE.lock().unwrap();
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }
    }

    // Not in cache, compile it
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;

    // Add to cache
    {
        let mut cache = REGEX_CACHE.lock().unwrap();
        // Double-check in case another thread added it while we were compiling
        if !cache.contains_key(pattern) {
            cache.insert(pattern.to_string(), regex.clone());
        }
    }

    Ok(regex)
}

/// Get split result from cache or compute and cache it.
fn get_cached_split(input: &str, separator: &str) -> Vec<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a hash of the input for cache key
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let input_hash = hasher.finish();

    let cache_key = (input_hash, separator.to_string());

    // Try to get from cache first
    {
        let cache = SPLIT_CACHE.lock().unwrap();
        if let Some(cached_split) = cache.get(&cache_key) {
            return cached_split.clone();
        }
    }

    // Not in cache, compute it
    let parts: Vec<String> = input.split(separator).map(str::to_string).collect();

    // Add to cache
    {
        let mut cache = SPLIT_CACHE.lock().unwrap();
        // Only cache if input is reasonably sized to avoid memory bloat
        if input.len() <= 10000 && parts.len() <= 1000 {
            cache.insert(cache_key, parts.clone());
        }
    }

    parts
}

/// Internal representation of values during pipeline processing.
///
/// Values can be either single strings or lists of strings, allowing operations
/// to work on both individual items and collections efficiently.
#[derive(Debug, Clone)]
pub(crate) enum Value {
    /// A single string value.
    Str(String),
    /// A list of string values.
    List(Vec<String>),
}

/// Enumeration of all supported string transformation operations.
///
/// Each variant represents a specific transformation that can be applied to strings
/// or lists of strings. Operations are designed to be composable and efficient,
/// supporting both functional-style transformations and imperative-style mutations.
///
/// # Operation Categories
///
/// - **🔪 Text Splitting & Joining**: [`Split`], [`Join`], [`Slice`]
/// - **✨ Text Transformation**: [`Upper`], [`Lower`], [`Trim`], [`Append`], [`Prepend`], [`Pad`], [`Substring`]
/// - **🔍 Pattern Matching & Replacement**: [`Replace`], [`RegexExtract`], [`Filter`], [`FilterNot`]
/// - **🗂️ List Processing**: [`Sort`], [`Reverse`], [`Unique`], [`Map`]
/// - **🧹 Utility**: [`StripAnsi`]
///
/// # Type System
///
/// Operations are categorized by their input/output type requirements:
///
/// - **String→String**: [`Upper`], [`Lower`], [`Trim`], [`Replace`], [`Append`], [`Prepend`], [`Pad`], [`Substring`], [`RegexExtract`], [`StripAnsi`]
/// - **List→List**: [`Sort`], [`Unique`], [`Slice`], [`Map`]
/// - **Type-preserving**: [`Filter`], [`FilterNot`], [`Reverse`]
/// - **Type-converting**: [`Split`] (String→List), [`Join`] (List→String)
///
/// Use `map:{operation}` to apply string operations to each item in a list.
///
/// [`Upper`]: StringOp::Upper
/// [`Lower`]: StringOp::Lower
/// [`Trim`]: StringOp::Trim
/// [`Replace`]: StringOp::Replace
/// [`Split`]: StringOp::Split
/// [`Join`]: StringOp::Join
/// [`Sort`]: StringOp::Sort
/// [`Unique`]: StringOp::Unique
/// [`Filter`]: StringOp::Filter
/// [`FilterNot`]: StringOp::FilterNot
/// [`Substring`]: StringOp::Substring
/// [`RegexExtract`]: StringOp::RegexExtract
/// [`Slice`]: StringOp::Slice
/// [`Map`]: StringOp::Map
/// [`Reverse`]: StringOp::Reverse
/// [`Pad`]: StringOp::Pad
/// [`Append`]: StringOp::Append
/// [`Prepend`]: StringOp::Prepend
/// [`StripAnsi`]: StringOp::StripAnsi
#[derive(Debug, Clone)]
pub enum StringOp {
    /// Split a string by separator and optionally select a range of parts.
    ///
    /// This operation converts a string into a list by splitting on the specified
    /// separator, then optionally selects a subset using the range specification.
    ///
    /// **Performance Optimization:** Common separators are cached to reduce memory allocations.
    ///
    /// # Fields
    ///
    /// * `sep` - The separator string to split on
    /// * `range` - Range specification for selecting parts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Split and take all parts
    /// let template = Template::parse("{split:,:..}").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "a,b,c");
    ///
    /// // Split and take specific index
    /// let template = Template::parse("{split:,:1}").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "b");
    ///
    /// // Split and take range
    /// let template = Template::parse("{split:,:1..3}").unwrap();
    /// assert_eq!(template.format("a,b,c,d").unwrap(), "b,c");
    /// ```
    Split { sep: String, range: RangeSpec },

    /// Join a list of strings with the specified separator.
    ///
    /// **Syntax:** `join:SEPARATOR`
    ///
    /// This operation takes a list of strings and combines them into a single
    /// string using the provided separator between each item.
    ///
    /// **Behavior on Different Input Types:**
    /// - **List:** Joins items with the separator in their current order (no sorting applied)
    /// - **String:** Returns the string unchanged (treats as single-item list)
    ///
    /// **Performance Optimization:** Common separators are cached for improved performance.
    ///
    /// # Fields
    ///
    /// * `sep` - The separator to insert between list items (empty string for no separator)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Join with hyphen
    /// let template = Template::parse("{split:,:..|join: - }").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "a - b - c");
    ///
    /// // Join with newlines
    /// let template = Template::parse("{split: :..|join:\\n}").unwrap();
    /// assert_eq!(template.format("hello world").unwrap(), "hello\nworld");
    ///
    /// // Join with no separator
    /// let template = Template::parse("{split:,:..|join:}").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "abc");
    /// ```
    Join { sep: String },

    /// Replace text using regex patterns with sed-like syntax.
    ///
    /// **Syntax:** `replace:s/PATTERN/REPLACEMENT/FLAGS`
    ///
    /// Supports full regex replacement with capture groups, flags for global/case-insensitive
    /// matching, and other standard regex features.
    ///
    /// **Performance Optimization:** Regex patterns are compiled and cached internally for
    /// reuse across operations. For simple string patterns without regex metacharacters
    /// and without global flag, a fast string replacement is used instead of regex compilation.
    ///
    /// # Fields
    ///
    /// * `pattern` - The regex pattern to search for
    /// * `replacement` - The replacement text (supports capture group references like `$1`, `$2`)
    /// * `flags` - Regex flags: `g` (global), `i` (case-insensitive), `m` (multiline), `s` (dot-all)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Basic replacement (first match only)
    /// let template = Template::parse("{replace:s/world/universe/}").unwrap();
    /// assert_eq!(template.format("hello world").unwrap(), "hello universe");
    ///
    /// // Global replacement with flags
    /// let template = Template::parse("{replace:s/l/L/g}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "heLLo");
    ///
    /// // Case-insensitive global replace
    /// let template = Template::parse("{replace:s/WORLD/universe/gi}").unwrap();
    /// assert_eq!(template.format("hello world").unwrap(), "hello universe");
    ///
    /// // Using capture groups
    /// let template = Template::parse("{replace:s/(.+)/[$1]/}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "[hello]");
    /// ```
    Replace {
        pattern: String,
        replacement: String,
        flags: String,
    },

    /// Convert text to uppercase.
    ///
    /// Applies Unicode-aware uppercase conversion to the entire string,
    /// properly handling international characters and special cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{upper}").unwrap();
    /// assert_eq!(template.format("hello world").unwrap(), "HELLO WORLD");
    /// assert_eq!(template.format("café").unwrap(), "CAFÉ");
    /// ```
    Upper,

    /// Convert text to lowercase.
    ///
    /// Applies Unicode-aware lowercase conversion to the entire string,
    /// properly handling international characters and special cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{lower}").unwrap();
    /// assert_eq!(template.format("HELLO WORLD").unwrap(), "hello world");
    /// assert_eq!(template.format("CAFÉ").unwrap(), "café");
    /// ```
    Lower,

    /// Trim whitespace or custom characters from string ends.
    ///
    /// **Syntax:** `trim[:CHARACTERS][:DIRECTION]`
    ///
    /// Supports trimming from both ends, left only, or right only, with
    /// customizable character sets for specialized trimming needs.
    ///
    /// **Whitespace Characters:** When no characters are specified, removes standard
    /// whitespace: spaces, tabs (`\t`), newlines (`\n`), and carriage returns (`\r`).
    ///
    /// **Performance Note:** ASCII-only strings use fast whitespace detection.
    ///
    /// # Fields
    ///
    /// * `chars` - Characters to trim (empty string means whitespace)
    /// * `direction` - Which end(s) to trim from: `both` (default), `left`, `right`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Trim whitespace from both ends
    /// let template = Template::parse("{trim}").unwrap();
    /// assert_eq!(template.format("  hello  ").unwrap(), "hello");
    ///
    /// // Trim from left only
    /// let template = Template::parse("{trim:left}").unwrap();
    /// assert_eq!(template.format("  hello  ").unwrap(), "hello  ");
    ///
    /// // Trim custom characters
    /// let template = Template::parse("{trim:xy}").unwrap();
    /// assert_eq!(template.format("xyhelloxy").unwrap(), "hello");
    ///
    /// // Trim custom characters from right only
    /// let template = Template::parse("{trim:*-+:right}").unwrap();
    /// assert_eq!(template.format("hello***").unwrap(), "hello");
    /// ```
    Trim {
        chars: String,
        direction: TrimDirection,
    },

    /// Extract substring by character index or range.
    ///
    /// Supports Unicode-aware character indexing with negative indices
    /// for counting from the end. Handles out-of-bounds gracefully.
    ///
    /// # Fields
    ///
    /// * `range` - Character range specification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Single character
    /// let template = Template::parse("{substring:1}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "e");
    ///
    /// // Character range
    /// let template = Template::parse("{substring:1..4}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "ell");
    /// ```
    Substring { range: RangeSpec },

    /// Append text to the end of a string.
    ///
    /// Adds the specified suffix to the end of the input string,
    /// supporting escape sequences and Unicode text.
    ///
    /// # Fields
    ///
    /// * `suffix` - Text to append
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{append:!}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "hello!");
    /// ```
    Append { suffix: String },

    /// Prepend text to the beginning of a string.
    ///
    /// Adds the specified prefix to the beginning of the input string,
    /// supporting escape sequences and Unicode text.
    ///
    /// # Fields
    ///
    /// * `prefix` - Text to prepend
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{prepend:>>}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), ">>hello");
    /// ```
    Prepend { prefix: String },

    /// Remove ANSI escape sequences from text.
    ///
    /// Strips color codes, cursor movement commands, and other ANSI escape
    /// sequences while preserving the actual text content and Unicode characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{strip_ansi}").unwrap();
    /// let input = "\x1b[31mRed Text\x1b[0m";
    /// assert_eq!(template.format(input).unwrap(), "Red Text");
    /// ```
    StripAnsi,

    /// Keep only list items matching a regex pattern.
    ///
    /// **Syntax:** `filter:PATTERN`
    ///
    /// Filters a list to retain only items that match the specified regex pattern.
    /// When applied to a single string, keeps the string if it matches or returns empty.
    ///
    /// **Behavior on Different Input Types:**
    /// - **List:** Keeps items that match the pattern
    /// - **String:** Returns the string if it matches, empty string otherwise
    ///
    /// **Performance Optimization:** Regex patterns are compiled and cached internally
    /// for improved performance in repeated operations.
    ///
    /// # Fields
    ///
    /// * `pattern` - Regex pattern for matching items
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Keep items starting with vowels
    /// let template = Template::parse("{split:,:..|filter:^[aeiou]|join:,}").unwrap();
    /// assert_eq!(template.format("apple,banana,orange,grape").unwrap(), "apple,orange");
    ///
    /// // Keep items containing numbers
    /// let template = Template::parse("{split:,:..|filter:\\d+|join:,}").unwrap();
    /// assert_eq!(template.format("item1,test,file22,doc").unwrap(), "item1,file22");
    ///
    /// // Filter .txt files
    /// let template = Template::parse("{split:,:..|filter:\\.txt$|join:\\n}").unwrap();
    /// assert_eq!(template.format("file.txt,readme.md,data.txt").unwrap(), "file.txt\ndata.txt");
    /// ```
    Filter { pattern: String },

    /// Remove list items matching a regex pattern.
    ///
    /// **Syntax:** `filter_not:PATTERN`
    ///
    /// Filters a list to remove items that match the specified regex pattern.
    /// When applied to a single string, removes the string if it matches.
    ///
    /// **Behavior on Different Input Types:**
    /// - **List:** Removes items that match the pattern
    /// - **String:** Returns empty string if it matches, original string otherwise
    ///
    /// # Fields
    ///
    /// * `pattern` - Regex pattern for matching items to remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Remove specific items
    /// let template = Template::parse("{split:,:..|filter_not:banana|join:,}").unwrap();
    /// assert_eq!(template.format("apple,banana,orange").unwrap(), "apple,orange");
    ///
    /// // Remove comments (lines starting with #)
    /// let template = Template::parse("{split:\\n:..|filter_not:^#|join:\\n}").unwrap();
    /// let input = "line1\n# comment\nline2\n# another comment\nline3";
    /// assert_eq!(template.format(input).unwrap(), "line1\nline2\nline3");
    ///
    /// // Remove empty lines
    /// let template = Template::parse("{split:\\n:..|filter_not:^$|join:\\n}").unwrap();
    /// assert_eq!(template.format("line1\n\nline2\n\nline3").unwrap(), "line1\nline2\nline3");
    /// ```
    FilterNot { pattern: String },

    /// Select a range of items from a list.
    ///
    /// Extracts a subset of items from a list using range syntax,
    /// supporting negative indexing and various range types.
    ///
    /// # Fields
    ///
    /// * `range` - Range specification for item selection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{split:,:..|slice:1..3|join:,}").unwrap();
    /// assert_eq!(template.format("a,b,c,d,e").unwrap(), "b,c");
    /// ```
    Slice { range: RangeSpec },

    /// Apply a sub-pipeline to each item in a list.
    ///
    /// Maps a sequence of operations over each item in a list, enabling
    /// complex per-item transformations while maintaining list structure.
    ///
    /// # Fields
    ///
    /// * `operations` - List of operations to apply to each item
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{split:,:..|map:{trim|upper}|join:,}").unwrap();
    /// assert_eq!(template.format(" a , b , c ").unwrap(), "A,B,C");
    /// ```
    Map { operations: Vec<StringOp> },

    /// Sort list items alphabetically.
    ///
    /// Sorts a list of strings in ascending or descending alphabetical order
    /// using lexicographic comparison with Unicode support.
    ///
    /// # Fields
    ///
    /// * `direction` - Sort direction (ascending or descending)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// let template = Template::parse("{split:,:..|sort|join:,}").unwrap();
    /// assert_eq!(template.format("c,a,b").unwrap(), "a,b,c");
    ///
    /// let template = Template::parse("{split:,:..|sort:desc|join:,}").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "c,b,a");
    /// ```
    Sort { direction: SortDirection },

    /// Reverse a string or list order.
    ///
    /// For strings, reverses the character order. For lists, reverses the item order.
    /// Properly handles Unicode characters and grapheme clusters.
    ///
    /// **Performance Note:** ASCII-only strings use fast byte-level reversal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Reverse string
    /// let template = Template::parse("{reverse}").unwrap();
    /// assert_eq!(template.format("hello").unwrap(), "olleh");
    ///
    /// // Reverse list
    /// let template = Template::parse("{split:,:..|reverse|join:,}").unwrap();
    /// assert_eq!(template.format("a,b,c").unwrap(), "c,b,a");
    /// ```
    Reverse,

    /// Remove duplicate items from a list.
    ///
    /// **Syntax:** `unique`
    ///
    /// Filters a list to keep only the first occurrence of each unique item,
    /// preserving the original order of first appearances.
    ///
    /// **Order Preservation:** The first occurrence of each item is kept, maintaining
    /// the original order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Basic deduplication
    /// let template = Template::parse("{split:,:..|unique|join:,}").unwrap();
    /// assert_eq!(template.format("a,b,a,c,b").unwrap(), "a,b,c");
    ///
    /// // Remove duplicate lines
    /// let template = Template::parse("{split:\\n:..|unique|join:\\n}").unwrap();
    /// let input = "line1\nline2\nline1\nline3\nline2";
    /// assert_eq!(template.format(input).unwrap(), "line1\nline2\nline3");
    ///
    /// // Combine with sort for alphabetical unique list
    /// let template = Template::parse("{split:,:..|unique|sort|join:,}").unwrap();
    /// assert_eq!(template.format("c,a,b,a,c").unwrap(), "a,b,c");
    /// ```
    Unique,

    /// Pad a string to a specified width.
    ///
    /// Adds padding characters to reach the target width, supporting
    /// left, right, or both-sides padding with customizable fill characters.
    ///
    /// # Fields
    ///
    /// * `width` - Target width in characters
    /// * `char` - Character to use for padding
    /// * `direction` - Where to add padding (left, right, or both)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Right padding (default)
    /// let template = Template::parse("{pad:5}").unwrap();
    /// assert_eq!(template.format("hi").unwrap(), "hi   ");
    ///
    /// // Left padding with custom character
    /// let template = Template::parse("{pad:5:0:left}").unwrap();
    /// assert_eq!(template.format("42").unwrap(), "00042");
    /// ```
    Pad {
        width: usize,
        char: char,
        direction: PadDirection,
    },

    /// Extract text using regex patterns with optional capture groups.
    ///
    /// Extracts the first match of a regex pattern, optionally selecting
    /// a specific capture group for more precise extraction.
    ///
    /// # Fields
    ///
    /// * `pattern` - Regex pattern to match
    /// * `group` - Optional capture group number (0 = entire match)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use string_pipeline::Template;
    ///
    /// // Extract numbers
    /// let template = Template::parse(r"{regex_extract:\d+}").unwrap();
    /// assert_eq!(template.format("item123").unwrap(), "123");
    ///
    /// // Extract capture group
    /// let template = Template::parse(r"{regex_extract:(\w+)@(\w+):1}").unwrap();
    /// assert_eq!(template.format("user@domain.com").unwrap(), "user");
    /// ```
    RegexExtract {
        pattern: String,
        group: Option<usize>,
    },
}

/// Specification for selecting ranges of items or characters.
///
/// Supports Rust-like range syntax with negative indexing for flexible
/// selection of subsequences from strings or lists.
///
/// # Variants
///
/// * [`Index`] - Single item selection
/// * [`Range`] - Range-based selection with optional bounds
///
/// [`Index`]: RangeSpec::Index
/// [`Range`]: RangeSpec::Range
#[derive(Debug, Clone, Copy)]
pub enum RangeSpec {
    /// Select a single item by index.
    ///
    /// Supports negative indexing where `-1` is the last item,
    /// `-2` is second to last, etc.
    ///
    /// # Examples
    ///
    /// - `1` - Second item (0-indexed)
    /// - `-1` - Last item
    /// - `0` - First item
    Index(isize),

    /// Select a range of items with optional start and end bounds.
    ///
    /// The third field indicates whether the end bound is inclusive.
    /// `None` values indicate open bounds (start from beginning or go to end).
    ///
    /// # Fields
    ///
    /// * `start` - Optional start index (None = from beginning)
    /// * `end` - Optional end index (None = to end)
    /// * `inclusive` - Whether end bound is inclusive
    ///
    /// # Examples
    ///
    /// - `(Some(1), Some(3), false)` - Items 1,2 (exclusive end)
    /// - `(Some(1), Some(3), true)` - Items 1,2,3 (inclusive end)
    /// - `(Some(2), None, false)` - From item 2 to end
    /// - `(None, Some(3), false)` - First 3 items
    Range(Option<isize>, Option<isize>, bool),
}

/// Direction for trimming operations.
///
/// Specifies which end(s) of a string to trim characters from.
#[derive(Debug, Clone, Copy)]
pub enum TrimDirection {
    /// Trim from both ends (default).
    Both,
    /// Trim from left (start) only.
    Left,
    /// Trim from right (end) only.
    Right,
}

/// Direction for sorting operations.
///
/// Specifies the order for sorting list items.
#[derive(Debug, Clone, Copy)]
pub enum SortDirection {
    /// Ascending order (A to Z).
    Asc,
    /// Descending order (Z to A).
    Desc,
}

/// Direction for padding operations.
///
/// Specifies where to add padding characters to reach target width.
#[derive(Debug, Clone, Copy)]
pub enum PadDirection {
    /// Add padding to the left (right-align text).
    Left,
    /// Add padding to the right (left-align text).
    Right,
    /// Add padding to both sides (center text).
    Both,
}

/// Resolves an index to a valid array position.
///
/// Handles negative indexing and bounds clamping to ensure valid array access.
/// Negative indices count backwards from the end of the collection.
///
/// # Arguments
///
/// * `idx` - The index to resolve (can be negative)
/// * `len` - The length of the collection
///
/// # Returns
///
/// A valid array index clamped to `[0, len)` range.
///
/// # Examples
///
/// ```rust
/// // This is an internal function, shown for documentation
/// // resolve_index(1, 5) -> 1
/// // resolve_index(-1, 5) -> 4 (last item)
/// // resolve_index(10, 5) -> 4 (clamped to last item)
/// ```
fn resolve_index(idx: isize, len: usize) -> usize {
    let len_i = len as isize;
    let resolved = if idx < 0 { len_i + idx } else { idx };
    resolved.clamp(0, len_i.max(0)) as usize
}

/// Applies a range specification to a slice, returning selected items.
///
/// This is a generic function that works with any cloneable type, supporting
/// both single index selection and range-based selection with proper bounds checking.
///
/// # Arguments
///
/// * `items` - The slice to select from
/// * `range` - The range specification
///
/// # Returns
///
/// A vector containing the selected items.
///
/// # Examples
///
/// ```rust
/// // This is an internal function, shown for documentation
/// // let items = vec!["a", "b", "c", "d"];
/// // apply_range(&items, &RangeSpec::Index(1)) -> vec!["b"]
/// // apply_range(&items, &RangeSpec::Range(Some(1), Some(3), false)) -> vec!["b", "c"]
/// ```
fn apply_range<T: Clone>(items: &[T], range: &RangeSpec) -> Vec<T> {
    let len = items.len();
    if len == 0 {
        return Vec::new();
    }

    match range {
        RangeSpec::Index(idx) => {
            let i = resolve_index(*idx, len).min(len - 1);
            if let Some(item) = items.get(i) {
                vec![item.clone()]
            } else {
                Vec::new()
            }
        }
        RangeSpec::Range(start, end, inclusive) => {
            let s_idx = start.map_or(0, |s| resolve_index(s, len));
            if s_idx >= len {
                return Vec::new();
            }

            let mut e_idx = end.map_or(len, |e| resolve_index(e, len));
            if *inclusive {
                e_idx = e_idx.saturating_add(1);
            }
            let e_idx = e_idx.min(len);

            if s_idx >= e_idx {
                Vec::new()
            } else {
                // Use slice.to_vec() for efficient contiguous memory copying
                items[s_idx..e_idx].to_vec()
            }
        }
    }
}

/// Applies a sequence of operations to an input string.
///
/// This is the main execution engine for the pipeline system. It processes
/// operations sequentially, maintaining type safety and providing comprehensive
/// error handling with optional debug output.
///
/// # Arguments
///
/// * `input` - The input string to transform
/// * `ops` - Slice of operations to apply in sequence
/// * `debug` - Whether to output detailed debug information with hierarchical tracing to stderr
/// * `context` - The debug context (template, multi-template, or sub-pipeline)
/// * `template_raw` - Optional template string for debug display
///
/// # Returns
///
/// * `Ok(String)` - The transformed result
/// * `Err(String)` - Error description if any operation fails
///
/// # Errors
///
/// Returns an error if:
/// - Any regex pattern fails to compile
/// - Operations are applied to incompatible types
/// - Nested map operations are attempted
/// - Invalid arguments are provided to operations
///
/// # Examples
///
/// ```rust
/// use string_pipeline::Template;
///
/// // This function is used internally by Template::format()
/// let template = Template::parse("{upper|trim}").unwrap();
/// let result = template.format("  hello  ").unwrap();
/// assert_eq!(result, "HELLO");
/// ```
pub fn apply_ops_internal(
    input: &str,
    ops: &[StringOp],
    debug: bool,
    context: DebugContext,
    template_raw: Option<&str>,
) -> Result<String, String> {
    if debug {
        apply_ops_with_debug(input, ops, context, template_raw)
    } else {
        apply_ops_without_debug(input, ops)
    }
}

/// Execute operations without debug tracing
fn apply_ops_without_debug(input: &str, ops: &[StringOp]) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut default_sep = " ".to_string();

    for op in ops {
        match op {
            StringOp::Map { operations } => {
                if let Value::List(list) = val {
                    let mapped = list
                        .iter()
                        .map(|item| apply_ops_without_debug(item, operations))
                        .collect::<Result<Vec<_>, _>>()?;

                    val = Value::List(mapped);
                } else {
                    return Err("Map operation can only be applied to lists".to_string());
                }
            }
            _ => {
                val = apply_single_operation(op, val, &mut default_sep)?;
            }
        }
    }

    Ok(match val {
        Value::Str(s) => s,
        Value::List(list) => {
            if list.is_empty() {
                String::new()
            } else {
                list.join(&default_sep)
            }
        }
    })
}

/// Execute operations with debug tracing enabled
fn apply_ops_with_debug(
    input: &str,
    ops: &[StringOp],
    context: DebugContext,
    template_raw: Option<&str>,
) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut default_sep = " ".to_string();

    // Create appropriate debug session
    let mut debug_session = match context {
        DebugContext::SingleTemplate => {
            DebugSession::new_single_template(true, template_raw.unwrap_or("").to_string())
        }
        DebugContext::SubPipeline {
            item_num,
            total_items,
        } => DebugSession::new_sub_pipeline(true, item_num, total_items),
    };

    // Start the debug session
    let pipeline_debugger = debug_session.start(input, ops);

    for (i, op) in ops.iter().enumerate() {
        let step_debugger = if let Some(ref debugger) = pipeline_debugger {
            debugger.start_step(i + 1, ops.len(), op, &val, i == ops.len() - 1)
        } else {
            None
        };

        match op {
            StringOp::Map { operations } => {
                if let Value::List(list) = val {
                    let mapped = list
                        .iter()
                        .enumerate()
                        .map(|(item_idx, item)| {
                            // Debug map item start within the pipeline debugger scope
                            if let Some(ref debugger) = pipeline_debugger {
                                debugger.debug_map_item_start(item_idx + 1, list.len(), item);
                            }

                            let result = apply_ops_internal(
                                item,
                                operations,
                                true,
                                DebugContext::SubPipeline {
                                    item_num: item_idx + 1,
                                    total_items: list.len(),
                                },
                                None, // Sub-pipelines don't have template strings
                            );

                            // Debug map item result within the pipeline debugger scope
                            if let Some(ref debugger) = pipeline_debugger {
                                debugger.debug_map_item_end(item_idx + 1, list.len(), &result);
                            }

                            result
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    val = Value::List(mapped);
                } else {
                    return Err("Map operation can only be applied to lists".to_string());
                }
            }
            _ => {
                val = apply_single_operation(op, val, &mut default_sep)?;
            }
        }

        // Complete step debugging and record timing
        if let Some(step_debugger) = step_debugger {
            let duration = step_debugger.complete(&val);
            debug_session.record_step_timing(i + 1, op, duration);
        }
    }

    // End the debug session
    debug_session.end(&val);

    Ok(match val {
        Value::Str(s) => s,
        Value::List(list) => {
            if list.is_empty() {
                String::new()
            } else {
                list.join(&default_sep)
            }
        }
    })
}

fn apply_string_operation<F>(val: Value, transform: F, op_name: &str) -> Result<Value, String>
where
    F: FnOnce(String) -> String,
{
    if let Value::Str(s) = val {
        Ok(Value::Str(transform(s)))
    } else {
        Err(format!(
            "{} operation can only be applied to strings. Use map:{{{}}} for lists.",
            op_name,
            op_name.to_lowercase()
        ))
    }
}

fn apply_list_operation<F>(val: Value, transform: F, op_name: &str) -> Result<Value, String>
where
    F: FnOnce(Vec<String>) -> Vec<String>,
{
    if let Value::List(list) = val {
        Ok(Value::List(transform(list)))
    } else {
        Err(format!(
            "{} operation can only be applied to lists",
            op_name
        ))
    }
}

fn apply_single_operation(
    op: &StringOp,
    val: Value,
    default_sep: &mut String,
) -> Result<Value, String> {
    match op {
        // List operations - work on lists
        StringOp::Split { sep, range } => {
            let parts: Vec<String> = match &val {
                Value::Str(s) => {
                    // Use cached split for string inputs
                    get_cached_split(s, sep)
                }
                Value::List(list) => list.iter().flat_map(|s| get_cached_split(s, sep)).collect(),
            };
            *default_sep = get_interned_separator(sep);

            let result = apply_range(&parts, range);

            // If the range is a single index, return a string instead of a list
            match range {
                RangeSpec::Index(_) => {
                    if result.len() == 1 {
                        Ok(Value::Str(result[0].clone()))
                    } else if result.is_empty() {
                        Ok(Value::Str(String::new()))
                    } else {
                        // This shouldn't happen with a single index, but handle gracefully
                        Ok(Value::List(result))
                    }
                }
                _ => Ok(Value::List(result)),
            }
        }
        StringOp::Join { sep } => {
            let result = match val {
                Value::List(list) => Value::Str(list.join(sep)),
                Value::Str(s) => Value::Str(s), // Pass through strings unchanged
            };
            *default_sep = get_interned_separator(sep);
            Ok(result)
        }
        StringOp::Slice { range } => {
            apply_list_operation(val, |list| apply_range(&list, range), "Slice")
        }
        StringOp::Filter { pattern } => {
            let re = get_cached_regex(pattern)?;
            match val {
                Value::List(list) => Ok(Value::List(
                    list.into_iter().filter(|s| re.is_match(s)).collect(),
                )),
                Value::Str(s) => Ok(Value::Str(if re.is_match(&s) { s } else { String::new() })),
            }
        }
        StringOp::FilterNot { pattern } => {
            let re = get_cached_regex(pattern)?;
            match val {
                Value::List(list) => Ok(Value::List(
                    list.into_iter().filter(|s| !re.is_match(s)).collect(),
                )),
                Value::Str(s) => Ok(Value::Str(if re.is_match(&s) { String::new() } else { s })),
            }
        }
        StringOp::Sort { direction } => {
            if let Value::List(mut list) = val {
                match direction {
                    SortDirection::Asc => list.sort(),
                    SortDirection::Desc => {
                        list.sort();
                        list.reverse();
                    }
                }
                Ok(Value::List(list))
            } else {
                Err("Sort operation can only be applied to lists".to_string())
            }
        }
        StringOp::Reverse => match val {
            Value::Str(s) => Ok(Value::Str(
                ascii_reverse(&s).unwrap_or_else(|| s.chars().rev().collect()),
            )),
            Value::List(mut list) => {
                list.reverse();
                Ok(Value::List(list))
            }
        },
        StringOp::Unique => apply_list_operation(
            val,
            |list| {
                let mut seen = std::collections::HashSet::new();
                list.into_iter()
                    .filter(|item| seen.insert(item.clone()))
                    .collect()
            },
            "Unique",
        ),
        StringOp::Substring { range } => {
            if let Value::Str(s) = val {
                if s.is_ascii() {
                    // Fast ASCII path - work directly with bytes
                    let bytes = s.as_bytes();
                    let result_bytes = apply_range(bytes, range);
                    // Safety: ASCII input guarantees valid UTF-8 output
                    let result = unsafe { String::from_utf8_unchecked(result_bytes) };
                    Ok(Value::Str(result))
                } else {
                    // UTF-8 handling for Unicode strings
                    let chars: Vec<char> = s.chars().collect();
                    let result: String = apply_range(&chars, range).into_iter().collect();
                    Ok(Value::Str(result))
                }
            } else {
                Err("Substring operation can only be applied to strings. Use map:{substring:...} for lists.".to_string())
            }
        }
        StringOp::Replace {
            pattern,
            replacement,
            flags,
        } => {
            if let Value::Str(s) = val {
                // Early exit for simple string patterns (not regex)
                if !flags.contains('g')
                    && !pattern.contains([
                        '\\', '.', '*', '+', '?', '^', '$', '|', '[', ']', '(', ')', '{', '}',
                    ])
                    && !s.contains(pattern)
                {
                    return Ok(Value::Str(s));
                }

                let pattern_to_use = if flags.is_empty() {
                    pattern.clone()
                } else {
                    let mut inline_flags = String::with_capacity(4);
                    for (flag, c) in [('i', 'i'), ('m', 'm'), ('s', 's'), ('x', 'x')] {
                        if flags.contains(flag) {
                            inline_flags.push(c);
                        }
                    }
                    if inline_flags.is_empty() {
                        pattern.clone()
                    } else {
                        format!("(?{}){}", inline_flags, pattern)
                    }
                };

                let re = get_cached_regex(&pattern_to_use)?;
                let result = if flags.contains('g') {
                    re.replace_all(&s, replacement.as_str()).to_string()
                } else {
                    re.replace(&s, replacement.as_str()).to_string()
                };
                Ok(Value::Str(result))
            } else {
                Err(
                    "Replace operation can only be applied to strings. Use map:{replace:...} for lists."
                        .to_string(),
                )
            }
        }
        StringOp::Upper => apply_string_operation(val, |s| s.to_uppercase(), "Upper"),
        StringOp::Lower => apply_string_operation(val, |s| s.to_lowercase(), "Lower"),
        StringOp::Trim { chars, direction } => {
            if let Value::Str(s) = val {
                // Fast path for default whitespace trimming
                let result = if chars.is_empty() || chars.trim().is_empty() {
                    match direction {
                        TrimDirection::Both => {
                            if let Some(trimmed) = ascii_trim(&s) {
                                trimmed.to_string()
                            } else {
                                s.trim().to_string()
                            }
                        }
                        TrimDirection::Left => s.trim_start().to_string(),
                        TrimDirection::Right => s.trim_end().to_string(),
                    }
                } else {
                    // Custom character trimming
                    let chars_to_trim: Vec<char> = chars.chars().collect();
                    match direction {
                        TrimDirection::Both => {
                            s.trim_matches(|c| chars_to_trim.contains(&c)).to_string()
                        }
                        TrimDirection::Left => s
                            .trim_start_matches(|c| chars_to_trim.contains(&c))
                            .to_string(),
                        TrimDirection::Right => s
                            .trim_end_matches(|c| chars_to_trim.contains(&c))
                            .to_string(),
                    }
                };
                Ok(Value::Str(result))
            } else {
                Err(
                    "Trim operation can only be applied to strings. Use map:{trim} for lists."
                        .to_string(),
                )
            }
        }

        StringOp::Append { suffix } => {
            apply_string_operation(val, |s| format!("{}{}", s, suffix), "Append")
        }
        StringOp::Prepend { prefix } => {
            apply_string_operation(val, |s| format!("{}{}", prefix, s), "Prepend")
        }
        StringOp::StripAnsi => {
            if let Value::Str(s) = val {
                let result = String::from_utf8(strip(s.as_bytes()))
                    .map_err(|_| "Failed to convert stripped bytes to UTF-8".to_string())?;
                Ok(Value::Str(result))
            } else {
                Err("StripAnsi operation can only be applied to strings. Use map:{strip_ansi} for lists.".to_string())
            }
        }
        StringOp::Pad {
            width,
            char,
            direction,
        } => {
            if let Value::Str(s) = val {
                let current_len = s.chars().count();
                let result = if current_len >= *width {
                    s
                } else {
                    let padding_needed = *width - current_len;
                    match direction {
                        PadDirection::Left => {
                            format!("{}{}", char.to_string().repeat(padding_needed), s)
                        }
                        PadDirection::Right => {
                            format!("{}{}", s, char.to_string().repeat(padding_needed))
                        }
                        PadDirection::Both => {
                            let left_pad = padding_needed / 2;
                            let right_pad = padding_needed - left_pad;
                            format!(
                                "{}{}{}",
                                char.to_string().repeat(left_pad),
                                s,
                                char.to_string().repeat(right_pad)
                            )
                        }
                    }
                };
                Ok(Value::Str(result))
            } else {
                Err(
                    "Pad operation can only be applied to strings. Use map:{pad:...} for lists."
                        .to_string(),
                )
            }
        }
        StringOp::RegexExtract { pattern, group } => {
            if let Value::Str(s) = val {
                let re = get_cached_regex(pattern)?;
                let result = if let Some(group_idx) = group {
                    re.captures(&s)
                        .and_then(|caps| caps.get(*group_idx))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                } else {
                    re.find(&s)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                };
                Ok(Value::Str(result))
            } else {
                Err("RegexExtract operation can only be applied to strings. Use map:{regex_extract:...} for lists.".to_string())
            }
        }
        StringOp::Map { .. } => Err("Map operations should be handled separately".to_string()),
    }
}
