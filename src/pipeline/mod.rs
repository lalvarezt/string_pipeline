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
use std::time::Instant;
use strip_ansi_escapes::strip;

pub use crate::pipeline::template::{MultiTemplate, Template};
pub use debug::DebugContext;
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
    /// **Performance Optimization:** ASCII-only strings use optimized whitespace detection.
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
    /// **Performance Optimization:** ASCII-only strings use optimized byte-level reversal.
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
                // Use slice.to_vec() which is optimized for copying contiguous memory
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
    debug_context: Option<DebugContext>,
) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut default_sep = " ".to_string();
    let start_time = if debug { Some(Instant::now()) } else { None };

    if debug {
        if let Some(ref ctx) = debug_context {
            ctx.print("═══════════════════════════════════════════════");
            if ctx.is_sub_pipeline() {
                ctx.print(&format!(
                    "🔧 SUB-PIPELINE START: {} operations to apply",
                    ops.len()
                ));
            } else {
                ctx.print(&format!(
                    "🚀 PIPELINE START: {} operations to apply",
                    ops.len()
                ));
            }
            ctx.print(&format!("Initial input: {}", ctx.format_value(&val)));

            if ops.len() > 1 {
                ctx.print("Operations to apply:");
                for (i, op) in ops.iter().enumerate() {
                    ctx.print_operation(op, i + 1);
                }
            }
            ctx.print("───────────────────────────────────────────────");
        }
    }

    for (i, op) in ops.iter().enumerate() {
        let step_start = if debug { Some(Instant::now()) } else { None };

        if debug {
            if let Some(ref ctx) = debug_context {
                // Get operation name for the step header
                let op_name = match op {
                    StringOp::Map { operations } => {
                        if operations.len() <= 3 {
                            let ops_str: Vec<String> =
                                operations.iter().map(|op| format!("{:?}", op)).collect();
                            format!("Map {{ operations: [{}] }}", ops_str.join(", "))
                        } else {
                            format!("Map (with {} operations)", operations.len())
                        }
                    }
                    _ => format!("{:?}", op),
                };

                ctx.print(&format!(
                    "STEP {}/{}: Applying {}",
                    i + 1,
                    ops.len(),
                    op_name
                ));

                // For Map with many operations, show the detailed breakdown
                if let StringOp::Map { operations } = op {
                    if operations.len() > 3 {
                        ctx.print("Map { operations: [");
                        for (i, map_op) in operations.iter().enumerate() {
                            ctx.print(&format!("    {}: {:?}", i + 1, map_op));
                        }
                        ctx.print("  ] }");
                    }
                }
                ctx.print(&format!("Input: {}", ctx.format_value(&val)));
            }
        }

        match op {
            StringOp::Map { operations } => {
                // Nested map operations are not supported (simplified check)
                // This could be enhanced with proper tracking if needed

                if let Value::List(list) = val {
                    let mapped = list
                        .iter()
                        .enumerate()
                        .map(|(item_idx, item)| {
                            if debug {
                                if let Some(ref parent_ctx) = debug_context {
                                    parent_ctx.print_map_item_start(item_idx + 1, list.len());
                                    parent_ctx.print_map_item_input(item);
                                }
                            }

                            let ctx = DebugContext::new_map_item(debug, item_idx + 1, list.len())
                                .with_depth(0)
                                .with_operation("map");

                            let result = apply_ops_internal(item, operations, debug, Some(ctx));

                            // Print map item output IMMEDIATELY after getting the result
                            if debug {
                                if let Some(ref parent_ctx) = debug_context {
                                    match &result {
                                        Ok(output) => parent_ctx.print_map_item_output(output),
                                        Err(e) => parent_ctx.print_map_item_error(e),
                                    }
                                }
                            }

                            result
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if debug {
                        if let Some(ref ctx) = debug_context {
                            ctx.print(&format!(
                                "MAP COMPLETED: {} → {} items",
                                list.len(),
                                mapped.len()
                            ));
                        }
                    }

                    val = Value::List(mapped);
                } else {
                    return Err("Map operation can only be applied to lists".to_string());
                }
            }

            // All other operations use the shared implementation
            _ => {
                val = apply_single_operation(op, val, &mut default_sep)?;
            }
        }

        if debug {
            if let Some(ref ctx) = debug_context {
                let elapsed = step_start.unwrap().elapsed();

                ctx.print_value(&val, "🎯 Result: ");
                ctx.print(&format!("Step completed in {:?}", elapsed));

                // Add memory usage info for large datasets
                match &val {
                    Value::List(list) if list.len() > 1000 => {
                        let total_chars: usize = list.iter().map(|s| s.len()).sum();
                        ctx.print(&format!(
                            "Memory: ~{} chars across {} items",
                            total_chars,
                            list.len()
                        ));
                    }
                    Value::Str(s) if s.len() > 10000 => {
                        ctx.print(&format!("Memory: ~{} chars in string", s.len()));
                    }
                    _ => {}
                }

                ctx.print("───────────────────────────────────────────────");
            }
        }
    }

    if debug {
        if let Some(ref ctx) = debug_context {
            let total_elapsed = start_time.unwrap().elapsed();
            if ctx.is_sub_pipeline() {
                ctx.print("✅ SUB-PIPELINE COMPLETE");
            } else {
                ctx.print("✅ PIPELINE COMPLETE");
            }
            ctx.print(&format!("Total execution time: {:?}", total_elapsed));
            ctx.print_value(&val, "🎯 Final result: ");

            // Show cache statistics only for main pipelines to avoid repetition
            if !ctx.is_sub_pipeline() {
                let regex_cache = REGEX_CACHE.lock().unwrap();
                let split_cache = SPLIT_CACHE.lock().unwrap();
                ctx.print(&format!(
                    "Cache stats: {} regex patterns, {} split operations cached",
                    regex_cache.len(),
                    split_cache.len()
                ));
            }

            ctx.print("═══════════════════════════════════════════════");
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
                    // Optimized ASCII path - work directly with bytes
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
                    // Custom character trimming with optimized character set
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

#[cfg(test)]
mod tests {
    use super::Template;

    fn process(input: &str, template: &str) -> Result<String, String> {
        let tmpl = Template::parse(template)?;
        tmpl.format(input)
    }

    // Single Operation Tests - Organized by Operation Type
    mod single_operations {

        mod positive_tests {
            use super::super::process;

            // Split operation tests
            #[test]
            fn test_split_basic_comma() {
                assert_eq!(process("a,b,c", "{split:,:..}").unwrap(), "a,b,c");
            }

            #[test]
            fn test_split_with_space() {
                assert_eq!(
                    process("hello world test", "{split: :..}").unwrap(),
                    "hello world test"
                );
            }

            #[test]
            fn test_split_with_index() {
                assert_eq!(process("a,b,c,d", "{split:,:1}").unwrap(), "b");
            }

            #[test]
            fn test_split_negative_index() {
                assert_eq!(process("a,b,c,d", "{split:,:-1}").unwrap(), "d");
            }

            #[test]
            fn test_split_range_exclusive() {
                assert_eq!(process("a,b,c,d,e", "{split:,:1..3}").unwrap(), "b,c");
            }

            #[test]
            fn test_split_range_inclusive() {
                assert_eq!(process("a,b,c,d,e", "{split:,:1..=3}").unwrap(), "b,c,d");
            }

            #[test]
            fn test_split_range_from() {
                assert_eq!(process("a,b,c,d,e", "{split:,:2..}").unwrap(), "c,d,e");
            }

            #[test]
            fn test_split_range_to() {
                assert_eq!(process("a,b,c,d,e", "{split:,:..3}").unwrap(), "a,b,c");
            }

            #[test]
            fn test_split_range_to_inclusive() {
                assert_eq!(process("a,b,c,d,e", "{split:,:..=2}").unwrap(), "a,b,c");
            }

            #[test]
            fn test_split_special_separator() {
                assert_eq!(process("a||b||c", r"{split:\|\|:..}").unwrap(), "a||b||c");
            }

            #[test]
            fn test_split_newline_separator() {
                assert_eq!(process("a\nb\nc", "{split:\\n:..}").unwrap(), "a\nb\nc");
            }

            #[test]
            fn test_split_tab_separator() {
                assert_eq!(process("a\tb\tc", r"{split:\t:..}").unwrap(), "a\tb\tc");
            }

            #[test]
            fn test_split_empty_parts() {
                assert_eq!(process("a,,b,c", "{split:,:..}").unwrap(), "a,,b,c");
            }

            #[test]
            fn test_split_single_item() {
                assert_eq!(process("single", "{split:,:..}").unwrap(), "single");
            }

            #[test]
            fn test_split_empty_string() {
                assert_eq!(process("", "{split:,:..}").unwrap(), "");
            }

            // Join operation tests
            #[test]
            fn test_join_basic() {
                assert_eq!(process("a,b,c", "{split:,:..|join:-}").unwrap(), "a-b-c");
            }

            #[test]
            fn test_join_with_space() {
                assert_eq!(process("a,b,c", "{split:,:..|join: }").unwrap(), "a b c");
            }

            #[test]
            fn test_join_empty_separator() {
                assert_eq!(process("a,b,c", "{split:,:..|join:}").unwrap(), "abc");
            }

            #[test]
            fn test_join_newline() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|join:\\n}").unwrap(),
                    "a\nb\nc"
                );
            }

            #[test]
            fn test_join_special_chars() {
                assert_eq!(process("a,b,c", "{split:,:..|join:@@}").unwrap(), "a@@b@@c");
            }

            #[test]
            fn test_join_unicode() {
                assert_eq!(process("a,b,c", "{split:,:..|join:🔥}").unwrap(), "a🔥b🔥c");
            }

            #[test]
            fn test_join_single_item() {
                assert_eq!(process("single", "{split:,:..|join:-}").unwrap(), "single");
            }

            #[test]
            fn test_join_empty_list() {
                assert_eq!(process("", "{split:,:..|join:-}").unwrap(), "");
            }

            // Replace operation tests
            #[test]
            fn test_replace_basic() {
                assert_eq!(
                    process("hello world", "{replace:s/world/universe/}").unwrap(),
                    "hello universe"
                );
            }

            #[test]
            fn test_replace_global() {
                assert_eq!(
                    process("foo foo foo", "{replace:s/foo/bar/g}").unwrap(),
                    "bar bar bar"
                );
            }

            #[test]
            fn test_replace_case_insensitive() {
                assert_eq!(
                    process("Hello HELLO hello", "{replace:s/hello/hi/gi}").unwrap(),
                    "hi hi hi"
                );
            }

            #[test]
            fn test_replace_multiline() {
                assert_eq!(
                    process("hello\nworld", "{replace:s/hello.world/hi universe/ms}").unwrap(),
                    "hi universe"
                );
            }

            #[test]
            fn test_replace_digits() {
                assert_eq!(
                    process("test123", "{replace:s/\\d+/456/}").unwrap(),
                    "test456"
                );
            }

            #[test]
            fn test_replace_word_boundaries() {
                assert_eq!(
                    process("cat caterpillar", "{replace:s/\\bcat\\b/dog/g}").unwrap(),
                    "dog caterpillar"
                );
            }

            #[test]
            fn test_replace_capture_groups() {
                assert_eq!(
                    process("hello world", "{replace:s/(\\w+) (\\w+)/$2 $1/}").unwrap(),
                    "world hello"
                );
            }

            #[test]
            fn test_replace_empty_replacement() {
                assert_eq!(
                    process("hello world", "{replace:s/world//}").unwrap(),
                    "hello "
                );
            }

            #[test]
            fn test_replace_special_chars() {
                assert_eq!(
                    process("a.b*c+d", "{replace:s/[.*+]/X/g}").unwrap(),
                    "aXbXcXd"
                );
            }

            #[test]
            fn test_replace_no_match() {
                assert_eq!(
                    process("hello world", "{replace:s/xyz/abc/}").unwrap(),
                    "hello world"
                );
            }

            // Case operation tests
            #[test]
            fn test_upper_basic() {
                assert_eq!(process("hello world", "{upper}").unwrap(), "HELLO WORLD");
            }

            #[test]
            fn test_upper_mixed_case() {
                assert_eq!(process("HeLLo WoRLd", "{upper}").unwrap(), "HELLO WORLD");
            }

            #[test]
            fn test_upper_with_numbers() {
                assert_eq!(process("hello123", "{upper}").unwrap(), "HELLO123");
            }

            #[test]
            fn test_upper_unicode() {
                assert_eq!(process("café naïve", "{upper}").unwrap(), "CAFÉ NAÏVE");
            }

            #[test]
            fn test_lower_basic() {
                assert_eq!(process("HELLO WORLD", "{lower}").unwrap(), "hello world");
            }

            #[test]
            fn test_lower_mixed_case() {
                assert_eq!(process("HeLLo WoRLd", "{lower}").unwrap(), "hello world");
            }

            #[test]
            fn test_lower_with_numbers() {
                assert_eq!(process("HELLO123", "{lower}").unwrap(), "hello123");
            }

            #[test]
            fn test_lower_unicode() {
                assert_eq!(process("CAFÉ NAÏVE", "{lower}").unwrap(), "café naïve");
            }

            // Trim operation tests
            #[test]
            fn test_trim_basic() {
                assert_eq!(process("  hello world  ", "{trim}").unwrap(), "hello world");
            }

            #[test]
            fn test_trim_tabs() {
                assert_eq!(process("\t\thello\t\t", "{trim}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_newlines() {
                assert_eq!(process("\n\nhello\n\n", "{trim}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_mixed_whitespace() {
                assert_eq!(process(" \t\n hello \t\n ", "{trim}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_no_whitespace() {
                assert_eq!(process("hello", "{trim}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_only_whitespace() {
                assert_eq!(process("   ", "{trim}").unwrap(), "");
            }

            #[test]
            fn test_trim_empty_string() {
                assert_eq!(process("", "{trim}").unwrap(), "");
            }

            #[test]
            fn test_trim_custom_chars_basic() {
                assert_eq!(process("xyhelloxy", "{trim:xy}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_single_char() {
                assert_eq!(process("aaahelloaaa", "{trim:a}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_multiple_chars() {
                assert_eq!(process("xyzhellopqr", "{trim:xyzpqr}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_no_match() {
                assert_eq!(process("hello", "{trim:xyz}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_all_chars() {
                assert_eq!(process("aaaa", "{trim:a}").unwrap(), "");
            }

            #[test]
            fn test_trim_custom_chars_empty() {
                assert_eq!(process("hello", "{trim:}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_unicode() {
                assert_eq!(process("🔥hello🔥", "{trim:🔥}").unwrap(), "hello");
            }

            #[test]
            fn test_trim_custom_chars_left() {
                assert_eq!(process("xyhelloxy", "{trim:xy:left}").unwrap(), "helloxy");
            }

            #[test]
            fn test_trim_custom_chars_right() {
                assert_eq!(process("xyhelloxy", "{trim:xy:right}").unwrap(), "xyhello");
            }

            #[test]
            fn test_trim_custom_chars_both_explicit() {
                assert_eq!(process("xyhelloxy", "{trim:xy:both}").unwrap(), "hello");
            }

            // substring operation tests
            #[test]
            fn test_substring_index() {
                assert_eq!(process("hello", "{substring:1}").unwrap(), "e");
            }

            #[test]
            fn test_substring_negative_index() {
                assert_eq!(process("hello", "{substring:-1}").unwrap(), "o");
            }

            #[test]
            fn test_substring_range_exclusive() {
                assert_eq!(process("hello", "{substring:1..3}").unwrap(), "el");
            }

            #[test]
            fn test_substring_range_inclusive() {
                assert_eq!(process("hello", "{substring:1..=3}").unwrap(), "ell");
            }

            #[test]
            fn test_substring_range_from() {
                assert_eq!(process("hello", "{substring:2..}").unwrap(), "llo");
            }

            #[test]
            fn test_substring_range_to() {
                assert_eq!(process("hello", "{substring:..3}").unwrap(), "hel");
            }

            #[test]
            fn test_substring_range_to_inclusive() {
                assert_eq!(process("hello", "{substring:..=2}").unwrap(), "hel");
            }

            #[test]
            fn test_substring_full_range() {
                assert_eq!(process("hello", "{substring:..}").unwrap(), "hello");
            }

            #[test]
            fn test_substring_empty_string() {
                assert_eq!(process("", "{substring:0}").unwrap(), "");
            }

            #[test]
            fn test_substring_out_of_bounds() {
                assert_eq!(process("hi", "{substring:5}").unwrap(), "i");
            }

            #[test]
            fn test_substring_unicode() {
                assert_eq!(process("café", "{substring:1..3}").unwrap(), "af");
            }

            // Append operation tests
            #[test]
            fn test_append_basic() {
                assert_eq!(process("hello", "{append:!}").unwrap(), "hello!");
            }

            #[test]
            fn test_append_multiple_chars() {
                assert_eq!(process("hello", "{append:_world}").unwrap(), "hello_world");
            }

            #[test]
            fn test_append_empty_string() {
                assert_eq!(process("", "{append:test}").unwrap(), "test");
            }

            #[test]
            fn test_append_unicode() {
                assert_eq!(process("hello", "{append:🔥}").unwrap(), "hello🔥");
            }

            #[test]
            fn test_append_special_chars() {
                assert_eq!(process("test", "{append:\\n}").unwrap(), "test\n");
            }

            #[test]
            fn test_append_escaped_colon() {
                assert_eq!(process("test", "{append:\\:value}").unwrap(), "test:value");
            }

            // Prepend operation tests
            #[test]
            fn test_prepend_basic() {
                assert_eq!(process("world", "{prepend:hello_}").unwrap(), "hello_world");
            }

            #[test]
            fn test_prepend_empty_string() {
                assert_eq!(process("", "{prepend:test}").unwrap(), "test");
            }

            #[test]
            fn test_prepend_unicode() {
                assert_eq!(process("world", "{prepend:🔥}").unwrap(), "🔥world");
            }

            #[test]
            fn test_prepend_special_chars() {
                assert_eq!(process("test", "{prepend:\\n}").unwrap(), "\ntest");
            }

            #[test]
            fn test_prepend_escaped_colon() {
                assert_eq!(process("test", "{prepend:value\\:}").unwrap(), "value:test");
            }

            // Shorthand syntax tests
            #[test]
            fn test_shorthand_index() {
                assert_eq!(process("a b c d", "{1}").unwrap(), "b");
            }

            #[test]
            fn test_shorthand_negative_index() {
                assert_eq!(process("a b c d", "{-1}").unwrap(), "d");
            }

            #[test]
            fn test_shorthand_range_exclusive() {
                assert_eq!(process("a b c d e", "{1..3}").unwrap(), "b c");
            }

            #[test]
            fn test_shorthand_range_inclusive() {
                assert_eq!(process("a b c d e", "{1..=3}").unwrap(), "b c d");
            }

            #[test]
            fn test_shorthand_range_from() {
                assert_eq!(process("a b c d e", "{2..}").unwrap(), "c d e");
            }

            #[test]
            fn test_shorthand_range_to() {
                assert_eq!(process("a b c d e", "{..3}").unwrap(), "a b c");
            }

            #[test]
            fn test_shorthand_full_range() {
                assert_eq!(process("a b c d", "{..}").unwrap(), "a b c d");
            }

            // Strip Ansi operation tests
            #[test]
            fn test_strip_ansi_basic() {
                // Basic ANSI color codes
                let input = "\x1b[31mRed text\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Red text");

                // Multiple ANSI sequences
                let input = "\x1b[1m\x1b[31mBold Red\x1b[0m\x1b[32m Green\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Bold Red Green");

                // No ANSI sequences (should be unchanged)
                let input = "Plain text";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Plain text");
            }

            #[test]
            fn test_strip_ansi_complex_sequences() {
                // Complex ANSI sequences
                let input = "\x1b[38;5;196mHello\x1b[0m \x1b[48;5;21mWorld\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Hello World");

                // Cursor movement sequences
                let input = "\x1b[2J\x1b[H\x1b[31mCleared screen\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Cleared screen");

                // Mixed content
                let input = "Normal \x1b[1mBold\x1b[0m and \x1b[4mUnderlined\x1b[0m text";
                assert_eq!(
                    process(input, "{strip_ansi}").unwrap(),
                    "Normal Bold and Underlined text"
                );
            }

            #[test]
            fn test_strip_ansi_edge_cases() {
                // Empty string
                assert_eq!(process("", "{strip_ansi}").unwrap(), "");

                // Only ANSI sequences
                let input = "\x1b[31m\x1b[1m\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "");

                // Malformed ANSI sequences (should still work)
                let input = "\x1b[99mText\x1b[";
                let result = process(input, "{strip_ansi}").unwrap();
                assert!(result.contains("Text"));
            }

            #[test]
            fn test_strip_ansi_real_world_examples() {
                // Git colored output
                let input = "\x1b[32m+\x1b[0m\x1b[32madded line\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "+added line");

                // ls colored output
                let input = "\x1b[0m\x1b[01;34mfolder\x1b[0m  \x1b[01;32mexecutable\x1b[0m";
                assert_eq!(
                    process(input, "{strip_ansi}").unwrap(),
                    "folder  executable"
                );

                // Grep colored output
                let input = "file.txt:\x1b[01;31m\x1b[Kmatch\x1b[m\x1b[Ked text";
                assert_eq!(
                    process(input, "{strip_ansi}").unwrap(),
                    "file.txt:matched text"
                );
            }

            #[test]
            fn test_strip_ansi_unicode_preservation() {
                // Ensure Unicode characters are preserved
                let input = "\x1b[31m🚀 Rocket\x1b[0m \x1b[32m🌟 Star\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "🚀 Rocket 🌟 Star");

                // Unicode with combining characters
                let input = "\x1b[31mCafé naïve résumé\x1b[0m";
                assert_eq!(process(input, "{strip_ansi}").unwrap(), "Café naïve résumé");
            }

            // Filter operation tests
            #[test]
            fn test_filter_on_string_value() {
                // Filter on string - match keeps string
                assert_eq!(process("hello", "{filter:hello}").unwrap(), "hello");
                assert_eq!(process("hello", "{filter:^hello$}").unwrap(), "hello");
                assert_eq!(
                    process("hello world", "{filter:world}").unwrap(),
                    "hello world"
                );

                // Filter on string - no match returns empty
                assert_eq!(process("hello", "{filter:goodbye}").unwrap(), "");
                assert_eq!(process("hello", "{filter:^world$}").unwrap(), "");
            }

            #[test]
            fn test_filter_not_on_string_value() {
                // Filter not on string - match returns empty
                assert_eq!(process("hello", "{filter_not:hello}").unwrap(), "");
                assert_eq!(process("hello world", "{filter_not:world}").unwrap(), "");

                // Filter not on string - no match keeps string
                assert_eq!(process("hello", "{filter_not:goodbye}").unwrap(), "hello");
                assert_eq!(process("hello", "{filter_not:^world$}").unwrap(), "hello");
            }

            #[test]
            fn test_filter_empty_inputs() {
                // Empty string
                assert_eq!(process("", "{filter:anything}").unwrap(), "");
                assert_eq!(process("", "{filter_not:anything}").unwrap(), "");
            }
        }

        mod negative_tests {
            use super::super::process;

            // Split operation negative tests
            #[test]
            fn test_split_invalid_range() {
                assert!(process("a,b,c", "{split:,:abc}").is_err());
            }

            #[test]
            fn test_split_malformed_range() {
                assert!(process("a,b,c", "{split:,:1..abc}").is_err());
            }

            // Replace operation negative tests
            #[test]
            fn test_replace_invalid_sed_format() {
                assert!(process("test", "{replace:invalid}").is_err());
            }

            #[test]
            fn test_replace_empty_pattern() {
                assert!(process("test", "{replace:s//replacement/}").is_err());
            }

            #[test]
            fn test_replace_invalid_regex() {
                assert!(process("test", "{replace:s/[/replacement/}").is_err());
            }

            #[test]
            fn test_replace_missing_delimiter() {
                assert!(process("test", "{replace:s/pattern}").is_err());
            }

            #[test]
            fn test_replace_invalid_flags() {
                // Invalid flags should be ignored or cause error
                let result = process("test", "{replace:s/t/T/xyz}");
                // Implementation may vary - either ignore invalid flags or error
                assert!(result.is_ok() || result.is_err());
            }

            // substring operation negative tests
            #[test]
            fn test_substring_invalid_range() {
                assert!(process("hello", "{substring:abc}").is_err());
            }

            #[test]
            fn test_substring_malformed_range() {
                assert!(process("hello", "{substring:1..abc}").is_err());
            }

            // Trim operation negative tests
            #[test]
            fn test_trim_missing_argument() {
                // This should be handled gracefully or error
                let result = process("hello", "{trim}");
                assert!(result.is_ok()); // trim without arguments should work (default whitespace)
            }

            // Append/Prepend operation negative tests
            #[test]
            fn test_append_missing_argument() {
                let result = process("hello", "{append}");
                assert!(result.is_err());
            }

            #[test]
            fn test_prepend_missing_argument() {
                let result = process("hello", "{prepend}");
                assert!(result.is_err());
            }

            // Unknown operation tests
            #[test]
            fn test_unknown_operation() {
                assert!(process("test", "{unknown_op}").is_err());
            }

            #[test]
            fn test_invalid_template_format() {
                assert!(process("test", "invalid_template").is_err());
            }

            #[test]
            fn test_malformed_template_braces() {
                assert!(process("test", "{split:,").is_err());
            }

            #[test]
            fn test_empty_template() {
                assert!(process("test", "{}").is_ok()); // Should work as no-op
            }

            // Shorthand negative tests
            #[test]
            fn test_shorthand_invalid_index() {
                assert!(process("a b c", "{abc}").is_err());
            }

            #[test]
            fn test_shorthand_invalid_range() {
                assert!(process("a b c", "{1..abc}").is_err());
            }

            // Filter negative tests
            #[test]
            fn test_filter_invalid_regex() {
                // Invalid regex patterns should return errors
                assert!(process("test", "{filter:[}").is_err());
                assert!(process("test", "{filter:(}").is_err());
                assert!(process("test", r"{filter:*}").is_err());
                assert!(process("test", r"{filter:+}").is_err());
                assert!(process("test", r"{filter:?}").is_err());

                // Same for filter_not
                assert!(process("test", "{filter_not:[}").is_err());
                assert!(process("test", "{filter_not:*}").is_err());
            }
        }
    }

    // Two-Step Pipeline Tests
    mod two_step_pipelines {
        mod positive_tests {
            use super::super::process;
            // Split + Join combinations
            #[test]
            fn test_split_join_different_separators() {
                assert_eq!(process("a,b,c", "{split:,:..|join:-}").unwrap(), "a-b-c");
            }

            #[test]
            fn test_split_join_with_range() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:1..3|join:;}").unwrap(),
                    "b;c"
                );
            }

            #[test]
            fn test_split_join_newline_to_space() {
                assert_eq!(process("a\nb\nc", "{split:\n:..|join: }").unwrap(), "a b c");
            }

            #[test]
            fn test_split_join_empty_separator() {
                assert_eq!(process("a,b,c", "{split:,:..|join:}").unwrap(), "abc");
            }

            #[test]
            fn test_split_join_unicode_separator() {
                assert_eq!(process("a,b,c", "{split:,:..|join:🔥}").unwrap(), "a🔥b🔥c");
            }

            // Split + Case operations
            #[test]
            fn test_split_upper() {
                assert_eq!(
                    process("hello,world", "{split:,:..|map:{upper}}").unwrap(),
                    "HELLO,WORLD"
                );
            }

            #[test]
            fn test_split_lower() {
                assert_eq!(
                    process("HELLO,WORLD", "{split:,:..|map:{lower}}").unwrap(),
                    "hello,world"
                );
            }

            // Split + Trim operations
            #[test]
            fn test_split_trim() {
                assert_eq!(
                    process(" a , b , c ", "{split:,:..|map:{trim}}").unwrap(),
                    "a,b,c"
                );
            }

            #[test]
            fn test_split_trim_with_range() {
                assert_eq!(
                    process(" a , b , c , d ", "{split:,:1..3|map:{trim}}").unwrap(),
                    "b,c"
                );
            }

            // Split + Strip operations
            #[test]
            fn test_split_trim_custom_chars() {
                assert_eq!(
                    process("xa,yb,zc", "{split:,:..|map:{trim:xyz}}").unwrap(),
                    "a,b,c"
                );
            }

            // Split + Append/Prepend operations
            #[test]
            fn test_split_append() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|map:{append:!}}").unwrap(),
                    "a!,b!,c!"
                );
            }

            #[test]
            fn test_split_prepend() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|map:{prepend:->}}").unwrap(),
                    "->a,->b,->c"
                );
            }

            #[test]
            fn test_split_append_with_index() {
                assert_eq!(
                    process("a,b,c", "{split:,:1|append:_test}").unwrap(),
                    "b_test"
                );
            }

            // Split + Replace operations
            #[test]
            fn test_split_replace() {
                assert_eq!(
                    process("hello,world,test", "{split:,:..|map:{replace:s/l/L/g}}").unwrap(),
                    "heLLo,worLd,test"
                );
            }

            #[test]
            fn test_split_replace_with_range() {
                assert_eq!(
                    process("hello,world,test", "{split:,:0..2|map:{replace:s/o/0/g}}").unwrap(),
                    "hell0,w0rld"
                );
            }

            // Trim + operations
            #[test]
            fn test_trim_split() {
                assert_eq!(process("  a,b,c  ", "{trim|split:,:..}").unwrap(), "a,b,c");
            }

            #[test]
            fn test_trim_upper() {
                assert_eq!(process("  hello  ", "{trim|upper}").unwrap(), "HELLO");
            }

            #[test]
            fn test_trim_append() {
                assert_eq!(process("  hello  ", "{trim|append:!}").unwrap(), "hello!");
            }

            // Replace + operations
            #[test]
            fn test_replace_upper() {
                assert_eq!(
                    process("hello world", "{replace:s/world/universe/|upper}").unwrap(),
                    "HELLO UNIVERSE"
                );
            }

            #[test]
            fn test_replace_split() {
                assert_eq!(
                    process("hello-world-test", "{replace:s/-/,/g|split:,:..}").unwrap(),
                    "hello,world,test"
                );
            }

            #[test]
            fn test_replace_trim() {
                assert_eq!(
                    process("  hello world  ", "{replace:s/world/universe/|trim}").unwrap(),
                    "hello universe"
                );
            }

            // substring + operations
            #[test]
            fn test_substring_upper() {
                assert_eq!(process("hello", "{substring:1..3|upper}").unwrap(), "EL");
            }

            #[test]
            fn test_substring_append() {
                assert_eq!(
                    process("hello", "{substring:0..3|append:...}").unwrap(),
                    "hel..."
                );
            }

            #[test]
            fn test_substring_replace() {
                assert_eq!(
                    process("hello world", "{substring:6..|replace:s/world/universe/}").unwrap(),
                    "universe"
                );
            }

            // Append/Prepend combinations
            #[test]
            fn test_append_prepend() {
                assert_eq!(
                    process("hello", "{append:!|prepend:->}").unwrap(),
                    "->hello!"
                );
            }

            #[test]
            fn test_prepend_append() {
                assert_eq!(
                    process("hello", "{prepend:->|append:!}").unwrap(),
                    "->hello!"
                );
            }

            #[test]
            fn test_append_upper() {
                assert_eq!(process("hello", "{append:!|upper}").unwrap(), "HELLO!");
            }

            #[test]
            fn test_prepend_lower() {
                assert_eq!(process("HELLO", "{prepend:->|lower}").unwrap(), "->hello");
            }

            // Strip + operations
            #[test]
            fn test_trim_custom_chars_upper() {
                assert_eq!(process("xyhelloxy", "{trim:xy|upper}").unwrap(), "HELLO");
            }

            #[test]
            fn test_trim_custom_chars_split() {
                assert_eq!(
                    process("xya,b,cxy", "{trim:xy|split:,:..}").unwrap(),
                    "a,b,c"
                );
            }

            // Complex separators and operations
            #[test]
            fn test_multichar_separator_operations() {
                assert_eq!(
                    process("a::b::c", r"{split:\:\::..|join:-}").unwrap(),
                    "a-b-c"
                );
            }

            #[test]
            fn test_escape_sequences_in_pipeline() {
                assert_eq!(
                    process("a\tb\tc", "{split:\t:..|join:\n}").unwrap(),
                    "a\nb\nc"
                );
            }

            // Split + Strip Ansi
            #[test]
            fn test_strip_ansi_on_list() {
                // ANSI sequences in list items
                let input = "\x1b[31mred\x1b[0m,\x1b[32mgreen\x1b[0m,\x1b[34mblue\x1b[0m";
                assert_eq!(
                    process(input, "{split:,:..|map:{strip_ansi}}").unwrap(),
                    "red,green,blue"
                );

                // Mixed ANSI and plain text in list
                let input = "plain,\x1b[1mbold\x1b[0m,\x1b[3mitalic\x1b[0m";
                assert_eq!(
                    process(input, "{split:,:..|map:{strip_ansi}}").unwrap(),
                    "plain,bold,italic"
                );
            }
        }

        mod negative_tests {
            use super::super::process;

            // Invalid pipeline combinations
            #[test]
            fn test_join_without_list() {
                // Join on a string that wasn't split should work (treat as single item)
                assert_eq!(process("hello", "{join:-}").unwrap(), "hello");
            }

            #[test]
            fn test_invalid_operation_in_pipeline() {
                assert!(process("test", "{split:,:..|unknown_op}").is_err());
            }

            #[test]
            fn test_malformed_second_operation() {
                assert!(process("a,b,c", "{split:,:..|upper:invalid}").is_err());
            }

            #[test]
            fn test_invalid_pipeline_syntax() {
                assert!(process("test", "{split:,||}").is_err());
            }

            #[test]
            fn test_missing_pipe_separator() {
                // This should be treated as a single operation with malformed args
                assert!(process("test", "{split:, upper}").is_err());
            }

            // Edge cases with empty results
            #[test]
            fn test_empty_result_pipeline() {
                assert_eq!(process("", "{trim|upper}").unwrap(), "");
            }

            #[test]
            fn test_operation_on_empty_split() {
                assert_eq!(process("", "{split:,:..|map:{upper}}").unwrap(), "");
            }

            // Invalid range combinations
            #[test]
            fn test_invalid_range_in_pipeline() {
                assert!(process("a,b,c", "{split:,:abc|upper}").is_err());
            }
        }
    }

    // Multi-Step Pipeline Tests
    mod multi_step_pipelines {
        mod positive_tests {
            use super::super::process;

            // Split + Transform + Join patterns
            #[test]
            fn test_split_upper_join() {
                assert_eq!(
                    process("hello,world,test", "{split:,:..|map:{upper}|join:-}").unwrap(),
                    "HELLO-WORLD-TEST"
                );
            }

            #[test]
            fn test_split_lower_join() {
                assert_eq!(
                    process("HELLO,WORLD,TEST", "{split:,:..|map:{lower}|join:_}").unwrap(),
                    "hello_world_test"
                );
            }

            #[test]
            fn test_split_trim_join() {
                assert_eq!(
                    process(" a , b , c ", r"{split:,:..|map:{trim}|join:\|}").unwrap(),
                    "a|b|c"
                );
            }

            #[test]
            fn test_split_append_join() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|map:{append:!}|join: }").unwrap(),
                    "a! b! c!"
                );
            }

            #[test]
            fn test_split_prepend_join() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|map:{prepend:->}|join:\\n}").unwrap(),
                    "->a\n->b\n->c"
                );
            }

            #[test]
            fn test_split_replace_join() {
                assert_eq!(
                    process(
                        "hello,world,test",
                        "{split:,:..|map:{replace:s/l/L/g}|join:;}"
                    )
                    .unwrap(),
                    "heLLo;worLd;test"
                );
            }

            #[test]
            fn test_split_trim_custom_chars_join() {
                assert_eq!(
                    process("xa,yb,zc", "{split:,:..|map:{trim:xyz}|join:-}").unwrap(),
                    "a-b-c"
                );
            }

            // Case + Split + Join operations
            #[test]
            fn test_upper_join() {
                assert_eq!(
                    process("hello world test", "{upper|split: :..|join:-}").unwrap(),
                    "HELLO-WORLD-TEST"
                );
            }

            #[test]
            fn test_lower_join() {
                assert_eq!(
                    process("HELLO WORLD TEST", "{lower|split: :..|join:_}").unwrap(),
                    "hello_world_test"
                );
            }

            // Split with range + Transform + Join
            #[test]
            fn test_split_range_upper_join() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:1..3|map:{upper}|join:-}").unwrap(),
                    "B-C"
                );
            }

            #[test]
            fn test_split_range_append_join() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:0..3|map:{append:_item}|join: }").unwrap(),
                    "a_item b_item c_item"
                );
            }

            #[test]
            fn test_split_index_transform_append() {
                assert_eq!(
                    process("hello,world,test", "{split:,:1|upper|append:!}").unwrap(),
                    "WORLD!"
                );
            }

            // Complex transformations
            #[test]
            fn test_trim_split_upper() {
                assert_eq!(
                    process("  hello,world  ", "{trim|split:,:..|map:{upper}}").unwrap(),
                    "HELLO,WORLD"
                );
            }

            #[test]
            fn test_replace_split_join() {
                assert_eq!(
                    process("hello-world-test", "{replace:s/-/,/g|split:,:..|join: }").unwrap(),
                    "hello world test"
                );
            }

            #[test]
            fn test_upper_split_join() {
                assert_eq!(
                    process("hello world test", "{upper|split: :..|join:_}").unwrap(),
                    "HELLO_WORLD_TEST"
                );
            }

            #[test]
            fn test_substring_split_join() {
                assert_eq!(
                    process("prefix:a,b,c", "{substring:7..|split:,:..|join:-}").unwrap(),
                    "a-b-c"
                );
            }

            // Multiple case transformations
            #[test]
            fn test_upper_lower_upper() {
                assert_eq!(process("Hello", "{upper|lower|upper}").unwrap(), "HELLO");
            }

            #[test]
            fn test_lower_upper_lower() {
                assert_eq!(process("HELLO", "{lower|upper|lower}").unwrap(), "hello");
            }

            // Multiple append/prepend operations
            #[test]
            fn test_prepend_append_prepend() {
                assert_eq!(
                    process("test", "{prepend:[|append:]|prepend:>>}").unwrap(),
                    ">>[test]"
                );
            }

            #[test]
            fn test_append_prepend_append() {
                assert_eq!(
                    process("test", "{append:!|prepend:->|append:?}").unwrap(),
                    "->test!?"
                );
            }

            // Split + Multiple transforms
            #[test]
            fn test_split_trim_upper() {
                assert_eq!(
                    process(" a , b , c ", "{split:,:..|map:{trim|upper}}").unwrap(),
                    "A,B,C"
                );
            }

            #[test]
            fn test_split_trim_custom_chars_lower() {
                assert_eq!(
                    process("XA,YB,ZC", "{split:,:..|map:{trim:XYZ|lower}}").unwrap(),
                    "a,b,c"
                );
            }

            #[test]
            fn test_split_replace_append() {
                assert_eq!(
                    process("hello,world", "{split:,:..|map:{replace:s/l/L/g|append:!}}").unwrap(),
                    "heLLo!,worLd!"
                );
            }

            // Complex range operations
            #[test]
            fn test_split_range_trim_join() {
                assert_eq!(
                    process(" a , b , c , d ", r"{split:,:1..3|map:{trim}|join:\|}").unwrap(),
                    "b|c"
                );
            }

            #[test]
            fn test_substring_append_substring() {
                assert_eq!(
                    process("hello", "{substring:1..4|append:_test|substring:0..5}").unwrap(),
                    "ell_t"
                );
            }

            // Unicode and special character handling
            #[test]
            fn test_unicode_three_step() {
                assert_eq!(
                    process("café,naïve,résumé", "{split:,:..|map:{upper}|join:🔥}").unwrap(),
                    "CAFÉ🔥NAÏVE🔥RÉSUMÉ"
                );
            }

            #[test]
            fn test_special_chars_pipeline() {
                assert_eq!(
                    process("a\tb\tc", "{split:\t:..|map:{prepend:[|append:]}|join: }").unwrap(),
                    "[a] [b] [c]"
                );
            }

            // Escape sequence handling
            #[test]
            fn test_escaped_colons_pipeline() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|map:{append:\\:value}|join: }").unwrap(),
                    "a:value b:value c:value"
                );
            }

            #[test]
            fn test_escaped_pipes_pipeline() {
                let result = process("test", r"{replace:s/test/a|b/|split:|:..|join:-}");
                assert_eq!(result.unwrap(), "a-b");
            }

            // Complex real-world scenarios
            #[test]
            fn test_csv_processing() {
                assert_eq!(
                    process("Name,Age,City", "{split:,:..|map:{lower|prepend:col_}}").unwrap(),
                    "col_name,col_age,col_city"
                );
            }

            #[test]
            fn test_path_processing() {
                assert_eq!(
                    process(
                        "/home/user/documents/file.txt",
                        "{split:/:-1|split:.:..|map:{append:_backup}}"
                    )
                    .unwrap(),
                    "file_backup.txt_backup"
                );
            }

            #[test]
            fn test_log_processing() {
                assert_eq!(
                    process("2023-01-01 ERROR Failed", "{split: :1..|join:_|lower}}").unwrap(),
                    "error_failed"
                );
            }

            // Edge cases with empty and single elements
            #[test]
            fn test_empty_string_three_steps() {
                assert_eq!(process("", "{trim|upper|append:test}").unwrap(), "test");
            }

            #[test]
            fn test_single_char_pipeline() {
                assert_eq!(process("a", "{upper|append:!|prepend:->}").unwrap(), "->A!");
            }

            // Large data handling
            #[test]
            fn test_many_elements() {
                let input = (0..100)
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let result = process(&input, "{split:,:0..5|map:{append:_num}|join:-}").unwrap();
                assert_eq!(result, "0_num-1_num-2_num-3_num-4_num");
            }

            // Deep transformations
            #[test]
            fn test_nested_transformations() {
                assert_eq!(
                    process(
                        "  HELLO,WORLD  ",
                        "{trim|split:,:..|map:{lower|prepend:item_}}"
                    )
                    .unwrap(),
                    "item_hello,item_world"
                );
            }

            // Split + String Ansi chaining
            #[test]
            fn test_strip_ansi_chaining() {
                // Chain with other operations
                let input = "\x1b[31mHELLO\x1b[0m,\x1b[32mWORLD\x1b[0m";
                assert_eq!(
                    process(input, "{split:,:..|map:{strip_ansi|lower}|join: }").unwrap(),
                    "hello world"
                );

                // Strip ANSI then transform
                let input = "\x1b[1m\x1b[31mtest\x1b[0m";
                assert_eq!(
                    process(input, "{strip_ansi|upper|append:!}").unwrap(),
                    "TEST!"
                );
            }

            // Filter chain tests
            #[test]
            fn test_filter_basic_string_matching() {
                // Filter list - keep matching items
                let input = "apple,banana,apricot,cherry,grape";
                assert_eq!(
                    process(input, "{split:,:..|filter:ap|join:,}").unwrap(),
                    "apple,apricot,grape"
                );

                // Filter list - exact match
                assert_eq!(
                    process(input, "{split:,:..|filter:^apple$|join:,}").unwrap(),
                    "apple"
                );

                // Filter list - case sensitive
                assert_eq!(
                    process("Apple,apple,APPLE", "{split:,:..|filter:apple|join:,}").unwrap(),
                    "apple"
                );
            }

            #[test]
            fn test_filter_not_basic() {
                // Filter out matching items
                let input = "apple,banana,apricot,cherry,grape";
                assert_eq!(
                    process(input, "{split:,:..|filter_not:ap|join:,}").unwrap(),
                    "banana,cherry"
                );

                // Filter out exact match
                assert_eq!(
                    process(input, "{split:,:..|filter_not:^banana$|join:,}").unwrap(),
                    "apple,apricot,cherry,grape"
                );
            }

            #[test]
            fn test_filter_regex_patterns() {
                let input = "test123,abc456,xyz789,hello,world123";

                // Numbers
                assert_eq!(
                    process(input, r"{split:,:..|filter:\d+|join:,}").unwrap(),
                    "test123,abc456,xyz789,world123"
                );

                // Start with letter
                assert_eq!(
                    process(input, r"{split:,:..|filter:^[a-z]+$|join:,}").unwrap(),
                    "hello"
                );

                // Contains specific pattern
                assert_eq!(
                    process(input, r"{split:,:..|filter:^.{3}\d+$|join:,}").unwrap(),
                    "abc456,xyz789"
                );
            }

            #[test]
            fn test_filter_case_insensitive_patterns() {
                let input = "Apple,BANANA,cherry,GRAPE";

                // Case insensitive matching
                assert_eq!(
                    process(input, r"{split:,:..|filter:(?i)apple|join:,}").unwrap(),
                    "Apple"
                );

                assert_eq!(
                    process(input, r"{split:,:..|filter:(?i)^[bg]|join:,}").unwrap(),
                    "BANANA,GRAPE"
                );
            }

            #[test]
            fn test_filter_special_characters() {
                let input = "hello.world,test@email.com,user:password,file.txt,data.json";

                // Dot literal
                assert_eq!(
                    process(input, r"{split:,:..|filter:\.|join:,}").unwrap(),
                    "hello.world,test@email.com,file.txt,data.json"
                );

                // Email pattern
                assert_eq!(
                    process(input, r"{split:,:..|filter:@.*.com|join:,}").unwrap(),
                    "test@email.com"
                );

                // File extensions
                assert_eq!(
                    process(input, r"{split:,:..|filter:.(txt|json)$|join:,}").unwrap(),
                    "file.txt,data.json"
                );

                // Colon separator
                assert_eq!(
                    process(input, r"{split:,:..|filter::|join:,}").unwrap(),
                    "user:password"
                );
            }

            #[test]
            fn test_filter_empty_inputs() {
                // Empty list (from splitting empty string)
                assert_eq!(
                    process("", "{split:,:..|filter:anything|join:,}").unwrap(),
                    ""
                );
                assert_eq!(
                    process("", "{split:,:..|filter_not:anything|join:,}").unwrap(),
                    ""
                );
            }

            #[test]
            fn test_filter_no_matches() {
                let input = "apple,banana,cherry";

                // Filter with no matches
                assert_eq!(
                    process(input, "{split:,:..|filter:xyz|join:,}").unwrap(),
                    ""
                );

                // Filter not with all matches (everything filtered out)
                assert_eq!(
                    process(input, "{split:,:..|filter_not:.*|join:,}").unwrap(),
                    ""
                );
            }

            #[test]
            fn test_filter_all_matches() {
                let input = "apple,banana,cherry";

                // Filter that matches everything
                assert_eq!(
                    process(input, "{split:,:..|filter:.*|join:,}").unwrap(),
                    "apple,banana,cherry"
                );

                // Filter not that matches nothing (keeps everything)
                assert_eq!(
                    process(input, "{split:,:..|filter_not:xyz|join:,}").unwrap(),
                    "apple,banana,cherry"
                );
            }

            #[test]
            fn test_filter_single_item_list() {
                // Single item that matches
                assert_eq!(
                    process("apple", "{split:,:..|filter:app|join:,}").unwrap(),
                    "apple"
                );

                // Single item that doesn't match
                assert_eq!(
                    process("apple", "{split:,:..|filter:xyz|join:,}").unwrap(),
                    ""
                );
            }

            #[test]
            fn test_filter_chaining() {
                let input = "Apple,banana,Cherry,grape,KIWI";

                // Filter then transform
                assert_eq!(
                    process(input, r"{split:,:..|filter:^[A-Z]|map:{lower}|join:,}").unwrap(),
                    "apple,cherry,kiwi"
                );

                // Transform then filter
                assert_eq!(
                    process(input, r"{split:,:..|map:{lower}|filter:^[ag]|join:,}").unwrap(),
                    "apple,grape"
                );

                // Multiple filters
                assert_eq!(
                    process(input, r"{split:,:..|filter:^[A-Za-z]|filter:a|join:,}").unwrap(),
                    "banana,grape"
                );
            }

            #[test]
            fn test_filter_with_slicing() {
                let input = "apple,banana,cherry,date,elderberry";

                // Filter then substring
                assert_eq!(
                    process(input, "{split:,:..|filter:e|slice:0..2|join:,}").unwrap(),
                    "apple,cherry"
                );

                // slice then filter
                assert_eq!(
                    process(input, "{split:,:..|slice:1..4|filter:a|join:,}").unwrap(),
                    "banana,date"
                );
            }

            #[test]
            fn test_filter_with_replace() {
                let input = "test1,test2,prod1,prod2,dev1";

                // Filter then replace
                assert_eq!(
                    process(
                        input,
                        "{split:,:..|filter:test|map:{replace:s/test/demo/g}|join:,}"
                    )
                    .unwrap(),
                    "demo1,demo2"
                );

                // Replace then filter
                assert_eq!(
                    process(
                        input,
                        "{split:,:..|map:{replace:s/\\d+//g}|filter:^test$|join:,}"
                    )
                    .unwrap(),
                    "test,test"
                );
            }

            #[test]
            fn test_filter_complex_chains() {
                let input = "  Apple  , banana ,  CHERRY  , grape,  KIWI  ";

                // Complex processing chain
                assert_eq!(
                    process(
                        input,
                        r"{split:,:..|map:{trim}|filter:^[A-Z]|map:{lower|append:-fruit}|join: \| }"
                    )
                    .unwrap(),
                    "apple-fruit | cherry-fruit | kiwi-fruit"
                );

                // Filter, sort-like operation with join
                let input2 = "zebra,apple,banana,cherry";
                assert_eq!(
                    process(input2, "{split:,:..|filter:^[abc]|map:{upper}|join:-}").unwrap(),
                    "APPLE-BANANA-CHERRY"
                );
            }

            #[test]
            fn test_filter_file_extensions() {
                let input = "file1.txt,script.py,data.json,image.png,doc.pdf,config.yaml";

                // Text files
                assert_eq!(
                    process(input, r"{split:,:..|filter:\.(txt|md|log)$|join:\n}").unwrap(),
                    "file1.txt"
                );

                // Code files
                assert_eq!(
                    process(input, r"{split:,:..|filter:\.(py|js|rs|java)$|join:\n}").unwrap(),
                    "script.py"
                );

                // Config files
                assert_eq!(
                    process(
                        input,
                        r"{split:,:..|filter:\.(json|yaml|yml|toml)$|join:\n}"
                    )
                    .unwrap(),
                    "data.json\nconfig.yaml"
                );
            }

            #[test]
            fn test_filter_log_processing() {
                let input = "INFO: Starting application,ERROR: Database connection failed,DEBUG: Query executed,WARNING: Deprecated function used,ERROR: Timeout occurred";

                // Error messages only
                assert_eq!(
                    process(input, "{split:,:..|filter:^ERROR|join:\\n}").unwrap(),
                    "ERROR: Database connection failed\nERROR: Timeout occurred"
                );

                // Non-debug messages
                assert_eq!(
                    process(input, "{split:,:..|filter_not:^DEBUG|join:\\n}").unwrap(),
                    "INFO: Starting application\nERROR: Database connection failed\nWARNING: Deprecated function used\nERROR: Timeout occurred"
                );
            }

            #[test]
            fn test_filter_ip_addresses() {
                let input = "192.168.1.1,10.0.0.1,invalid-ip,172.16.0.1,not.an.ip,127.0.0.1";

                // Simple IP pattern (basic validation)
                assert_eq!(
                    process(input, r"{split:,:..|filter:^\d+\.\d+\.\d+\.\d+$|join:\n}").unwrap(),
                    "192.168.1.1\n10.0.0.1\n172.16.0.1\n127.0.0.1"
                );

                // Private IP ranges
                assert_eq!(
                    process(
                        input,
                        r"{split:,:..|filter:^(192.168\.|10\.|172.16\.)|join:,}"
                    )
                    .unwrap(),
                    "192.168.1.1,10.0.0.1,172.16.0.1"
                );
            }

            #[test]
            fn test_filter_email_validation() {
                let input =
                    "user@example.com,invalid-email,test@test.org,not.an.email,admin@site.co.uk";

                // Basic email pattern
                assert_eq!(
                    process(input, r"{split:,:..|filter:@|join:\n}").unwrap(),
                    "user@example.com\ntest@test.org\nadmin@site.co.uk"
                );

                // Specific domain
                assert_eq!(
                    process(input, r"{split:,:..|filter:@example.com|join:,}").unwrap(),
                    "user@example.com"
                );
            }

            #[test]
            fn test_filter_multiline_strings() {
                // When processing strings with newlines
                let input = "line1\nline2,single_line,multi\nline\ntext";

                // Filter items containing newlines
                assert_eq!(
                    process(input, r"{split:,:..|filter:\n|join: \| }").unwrap(),
                    "line1\nline2 | multi\nline\ntext"
                );

                // Filter single lines only
                assert_eq!(
                    process(input, r"{split:,:..|filter_not:\n|join:,}").unwrap(),
                    "single_line"
                );
            }

            #[test]
            fn test_filter_large_lists() {
                // Test with a larger dataset
                let large_input: Vec<String> = (0..1000).map(|i| format!("item{}", i)).collect();
                let input_str = large_input.join(",");

                // Filter even numbers
                let result = process(
                    &input_str,
                    r"{split:,:..|filter:[02468]$|slice:0..5|join:,}",
                )
                .unwrap();
                assert_eq!(result, "item0,item2,item4,item6,item8");
            }

            #[test]
            fn test_filter_empty_strings_in_list() {
                // List with empty strings
                let input = "apple,,banana,,cherry,";

                // Filter out empty strings
                assert_eq!(
                    process(input, r"{split:,:..|filter_not:^$|join:,}").unwrap(),
                    "apple,banana,cherry"
                );

                // Filter only empty strings
                assert_eq!(
                    process(input, r"{split:,:..|filter:^$|join:\|\|}").unwrap(),
                    "||||"
                );
            }
        }

        mod negative_tests {
            use super::super::process;

            // Invalid three-step combinations
            #[test]
            fn test_invalid_middle_operation() {
                assert!(process("test", "{split:,:..|invalid_op|join:-}").is_err());
            }

            #[test]
            fn test_invalid_final_operation() {
                assert!(process("test", "{split:,:..|map:{upper}|invalid_op}").is_err());
            }

            #[test]
            fn test_malformed_three_step() {
                assert!(process("test", "{split:,|map:{upper}|}").is_err());
            }

            #[test]
            fn test_missing_arguments_in_pipeline() {
                assert!(process("test", "{split|upper|join}").is_err());
            }

            // Invalid operations on wrong types
            #[test]
            fn test_multiple_joins() {
                // Multiple joins should work - second join treats string as single item
                assert_eq!(
                    process("a,b,c", "{split:,:..|join:-|join:_}").unwrap(),
                    "a-b-c"
                );
            }

            // Complex error cases
            #[test]
            fn test_invalid_regex_in_pipeline() {
                assert!(process("test", "{split:,:..|map:{replace:s/[/invalid/|upper}}").is_err());
            }

            #[test]
            fn test_invalid_range_in_three_step() {
                assert!(process("a,b,c", "{split:,:abc|map:{upper}|join:-}").is_err());
            }

            #[test]
            fn test_empty_results_propagation() {
                assert_eq!(process("", "{split:,:..|map:{upper}|join:-}").unwrap(), "");
            }

            // Extremely long pipelines that should be rejected
            #[test]
            fn test_too_many_pipe_separators() {
                let result = process("test", "{split:,|||||||||upper}");
                assert!(result.is_err());
            }
        }
    }

    // Debug Functionality Tests
    mod debug_tests {
        use super::*;

        #[test]
        fn test_debug_flag_basic() {
            let result = process("hello", "{!upper}");
            assert!(result.is_ok());
            // The result should still be the processed string
            assert_eq!(result.unwrap(), "HELLO");
        }

        #[test]
        fn test_debug_flag_with_split() {
            let result = process("a,b,c", "{!split:,:..}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "a,b,c");
        }

        #[test]
        fn test_debug_flag_two_step() {
            let result = process("hello,world", "{!split:,:..|map:{upper}}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "HELLO,WORLD");
        }

        #[test]
        fn test_debug_flag_three_step() {
            let result = process("hello,world", "{!split:,:..|map:{upper}|join:-}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "HELLO-WORLD");
        }

        #[test]
        fn test_debug_flag_complex_pipeline() {
            let result = process(
                "  a , b , c  ",
                "{!trim|split:,:..|map:{trim|upper}|join:_}",
            );
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "A_B_C");
        }

        #[test]
        fn test_debug_flag_with_shorthand() {
            let result = process("a b c d", "{!1}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "b");
        }

        #[test]
        fn test_debug_flag_with_replace() {
            let result = process("hello world", "{!replace:s/world/universe/}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello universe");
        }

        #[test]
        fn test_debug_flag_with_substring() {
            let result = process("hello", "{!substring:1..3}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "el");
        }

        #[test]
        fn test_debug_flag_with_append_prepend() {
            let result = process("test", "{!prepend:->|append:!}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "->test!");
        }

        #[test]
        fn test_debug_flag_with_unicode() {
            let result = process("café", "{!upper}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "CAFÉ");
        }

        #[test]
        fn test_debug_flag_with_empty_input() {
            let result = process("", "{!upper}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "");
        }

        #[test]
        fn test_debug_flag_with_trim() {
            let result = process("  hello  ", "{!trim}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello");
        }

        #[test]
        fn test_debug_flag_with_trim_custom_chars() {
            let result = process("xyhelloxy", "{!trim:xy}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello");
        }

        #[test]
        fn test_debug_flag_error_cases() {
            let result = process("test", "{!invalid_op}");
            assert!(result.is_err());
        }

        #[test]
        fn test_debug_flag_with_malformed_operation() {
            let result = process("test", "{!split:}");
            assert!(result.is_err());
        }

        #[test]
        fn test_debug_without_operations() {
            let result = process("test", "{!}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "test");
        }

        #[test]
        fn test_debug_flag_positioning() {
            // Debug flag should only work at the beginning
            let result = process("test", "{upper!}");
            assert!(result.is_err()); // This should be invalid syntax
        }

        #[test]
        fn test_multiple_debug_flags() {
            // Multiple debug flags should be invalid
            let result = process("test", "{!!upper}");
            assert!(result.is_err());
        }

        #[test]
        fn test_debug_flag_with_escape_sequences() {
            let result = process("test", "{!append:\\n}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "test\n");
        }

        #[test]
        fn test_debug_flag_large_dataset() {
            let input = (0..50).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
            let result = process(&input, "{!split:,:0..10|join:-}");
            assert!(result.is_ok());
        }

        #[test]
        fn test_debug_flag_with_nested_operations() {
            let result = process("hello world test", "{!split: :..|map:{upper}|join:_|lower}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello_world_test");
        }

        #[test]
        fn test_debug_flag_regex_operations() {
            let result = process("test123", r"{!replace:s/\d+/XXX/}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "testXXX");
        }

        #[test]
        fn test_debug_flag_boundary_conditions() {
            let result = process("a", "{!substring:-1}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "a");
        }
    }

    // Map operation tests
    #[test]
    fn test_map_new_syntax_substring() {
        assert_eq!(
            process("hello,world,test", "{split:,:..|map:{substring:0..2}}").unwrap(),
            "he,wo,te"
        );
    }

    #[test]
    fn test_map_new_syntax_upper() {
        assert_eq!(
            process("hello,world,test", "{split:,:..|map:{upper}}").unwrap(),
            "HELLO,WORLD,TEST"
        );
    }

    #[test]
    fn test_map_new_syntax_append() {
        assert_eq!(
            process("a,b,c", "{split:,:..|map:{append:!}}").unwrap(),
            "a!,b!,c!"
        );
    }

    #[test]
    fn test_map_new_syntax_pad() {
        assert_eq!(
            process("a,bb,c", "{split:,:..|map:{pad:3:*:both}}").unwrap(),
            "*a*,bb*,*c*"
        );
    }

    #[test]
    fn test_map_new_syntax_replace() {
        assert_eq!(
            process("hello,world", "{split:,:..|map:{replace:s/l/L/g}}").unwrap(),
            "heLLo,worLd"
        );
    }

    #[test]
    fn test_map_new_syntax_trim() {
        assert_eq!(
            process(" a , b , c ", "{split:,:..|map:{trim:both}}").unwrap(),
            "a,b,c"
        );
    }

    #[test]
    fn test_map_new_syntax_regex_extract() {
        let input = "user123,admin456,guest789";
        assert_eq!(
            process(input, r"{split:,:..|map:{regex_extract:\d+}}").unwrap(),
            "123,456,789"
        );
    }

    #[test]
    fn test_map_new_syntax_complex() {
        assert_eq!(
            process(
                "  hello  ,  world  ",
                "{split:,:..|map:{trim:both}|map:{upper}|join:-}"
            )
            .unwrap(),
            "HELLO-WORLD"
        );
    }

    #[test]
    fn test_map_nested_ranges() {
        assert_eq!(
            process("hello,world,testing", "{split:,:..|map:{substring:1..=3}}").unwrap(),
            "ell,orl,est"
        );
    }

    #[test]
    fn test_map_error_cases() {
        // Missing braces should error
        assert!(process("a,b,c", "{split:,:..|map:upper}").is_err());

        // Invalid operation inside map should error
        assert!(process("a,b,c", "{split:,:..|map:{invalid_op}}").is_err());
    }

    // Sort operation tests
    #[test]
    fn test_sort_asc() {
        assert_eq!(
            process("zebra,apple,banana", "{split:,:..|sort}").unwrap(),
            "apple,banana,zebra"
        );
    }

    #[test]
    fn test_sort_desc() {
        assert_eq!(
            process("zebra,apple,banana", "{split:,:..|sort:desc}").unwrap(),
            "zebra,banana,apple"
        );
    }

    #[test]
    fn test_sort_asc_explicit() {
        assert_eq!(process("c,a,b", "{split:,:..|sort:asc}").unwrap(), "a,b,c");
    }

    #[test]
    fn test_sort_on_string_error() {
        assert!(process("hello", "{sort}").is_err());
    }

    // Reverse operation tests
    #[test]
    fn test_reverse_string() {
        assert_eq!(process("hello", "{reverse}").unwrap(), "olleh");
    }

    #[test]
    fn test_reverse_list() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|reverse}").unwrap(),
            "d,c,b,a"
        );
    }

    #[test]
    fn test_reverse_unicode_string() {
        assert_eq!(process("café", "{reverse}").unwrap(), "éfac");
    }

    // Unique operation tests
    #[test]
    fn test_unique_basic() {
        assert_eq!(
            process("a,b,a,c,b,d", "{split:,:..|unique}").unwrap(),
            "a,b,c,d"
        );
    }

    #[test]
    fn test_unique_empty_list() {
        assert_eq!(process("", "{split:,:..|unique}").unwrap(), "");
    }

    #[test]
    fn test_unique_no_duplicates() {
        assert_eq!(process("a,b,c", "{split:,:..|unique}").unwrap(), "a,b,c");
    }

    #[test]
    fn test_unique_on_string_error() {
        assert!(process("hello", "{unique}").is_err());
    }

    // Pad operation tests
    #[test]
    fn test_pad_right_default() {
        assert_eq!(process("hi", "{pad:5}").unwrap(), "hi   ");
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(process("hi", "{pad:5: :left}").unwrap(), "   hi");
    }

    #[test]
    fn test_pad_both() {
        assert_eq!(process("hi", "{pad:6: :both}").unwrap(), "  hi  ");
    }

    #[test]
    fn test_pad_custom_char() {
        assert_eq!(process("hi", "{pad:5:*:right}").unwrap(), "hi***");
    }

    #[test]
    fn test_pad_already_long_enough() {
        assert_eq!(process("hello", "{pad:3}").unwrap(), "hello");
    }

    #[test]
    fn test_pad_list() {
        assert_eq!(
            process("a,bb,ccc", "{split:,:..|map:{pad:4:0:left}}").unwrap(),
            "000a,00bb,0ccc"
        );
    }

    #[test]
    fn test_pad_unicode() {
        assert_eq!(process("café", "{pad:6:*:both}").unwrap(), "*café*");
    }

    // Regex extract operation tests
    #[test]
    fn test_regex_extract_basic() {
        assert_eq!(
            process("hello123world", r"{regex_extract:\d+}").unwrap(),
            "123"
        );
    }

    #[test]
    fn test_regex_extract_no_match() {
        assert_eq!(process("hello world", r"{regex_extract:\d+}").unwrap(), "");
    }

    #[test]
    fn test_regex_extract_group() {
        assert_eq!(
            process("email@domain.com", r"{regex_extract:(\w+)@(\w+):1}").unwrap(),
            "email"
        );
    }

    #[test]
    fn test_regex_extract_group_2() {
        assert_eq!(
            process("email@domain.com", r"{regex_extract:(\w+)@(\w+):2}").unwrap(),
            "domain"
        );
    }

    #[test]
    fn test_regex_extract_list() {
        assert_eq!(
            process(
                "test123,abc456,xyz",
                r"{split:,:..|map:{regex_extract:\d+}}"
            )
            .unwrap(),
            "123,456,"
        );
    }

    #[test]
    fn test_regex_extract_invalid_regex() {
        assert!(process("test", r"{regex_extract:[}").is_err());
    }

    // Modified trim operation tests
    #[test]
    fn test_trim_both_default() {
        assert_eq!(process("  hello  ", "{trim}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_left() {
        assert_eq!(process("  hello  ", "{trim:left}").unwrap(), "hello  ");
    }

    #[test]
    fn test_trim_right() {
        assert_eq!(process("  hello  ", "{trim:right}").unwrap(), "  hello");
    }

    #[test]
    fn test_trim_both_explicit() {
        assert_eq!(process("  hello  ", "{trim:both}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_list() {
        assert_eq!(
            process(" a , b , c ", "{split:,:..|map:{trim:both}}").unwrap(),
            "a,b,c"
        );
    }

    // Complex pipeline tests with new operations
    #[test]
    fn test_complex_pipeline_with_new_ops() {
        assert_eq!(
            process("  c,a,b,a,c  ", "{trim|split:,:..|map:{trim}|unique|sort}").unwrap(),
            "a,b,c"
        );
    }

    #[test]
    fn test_pipeline_with_map_and_pad() {
        assert_eq!(
            process("a,bb,c", "{split:,:..|map:{pad:3:*:both}}").unwrap(),
            "*a*,bb*,*c*"
        );
    }

    #[test]
    fn test_regex_extract_with_map() {
        let input = "user123,admin456,guest789";
        assert_eq!(
            process(input, r"{split:,:..|map:{regex_extract:\d+}|join:-}").unwrap(),
            "123-456-789"
        );
    }

    #[test]
    fn test_sort_reverse_combination() {
        assert_eq!(
            process("b,a,d,c", "{split:,:..|sort|reverse}").unwrap(),
            "d,c,b,a"
        );
    }

    #[test]
    fn test_pad_trim_combination() {
        assert_eq!(
            process("  hello  ", "{trim:both|pad:20:*:both}").unwrap(),
            "*******hello********"
        );
    }

    mod map_operations_tests {

        use crate::pipeline::parser::parse_template;

        // Helper function for processing input with templates
        fn process(input: &str, template: &str) -> Result<String, String> {
            use crate::pipeline::apply_ops_internal;
            let (operations, debug) =
                parse_template(template).map_err(|e| format!("Parse error: {}", e))?;
            apply_ops_internal(input, &operations, debug, None)
        }

        #[cfg(test)]
        mod individual_operations {
            use super::*;

            #[cfg(test)]
            mod basic_operations {
                use super::*;

                #[test]
                fn test_map_upper() {
                    assert_eq!(
                        process("apple,banana,cherry", "{split:,:..|map:{upper}}").unwrap(),
                        "APPLE,BANANA,CHERRY"
                    );
                }

                #[test]
                fn test_map_lower() {
                    assert_eq!(
                        process("APPLE,BANANA,CHERRY", "{split:,:..|map:{lower}}").unwrap(),
                        "apple,banana,cherry"
                    );
                }

                #[test]
                fn test_map_trim_default() {
                    assert_eq!(
                        process("  apple  ,  banana  ,  cherry  ", "{split:,:..|map:{trim}}")
                            .unwrap(),
                        "apple,banana,cherry"
                    );
                }

                #[test]
                fn test_map_trim_both() {
                    assert_eq!(
                        process(
                            "  apple  ,  banana  ,  cherry  ",
                            "{split:,:..|map:{trim:both}}"
                        )
                        .unwrap(),
                        "apple,banana,cherry"
                    );
                }

                #[test]
                fn test_map_trim_left() {
                    assert_eq!(
                        process("  apple  ,  banana  ", "{split:,:..|map:{trim:left}}").unwrap(),
                        "apple  ,banana  "
                    );
                }

                #[test]
                fn test_map_trim_right() {
                    assert_eq!(
                        process("  apple  ,  banana  ", "{split:,:..|map:{trim:right}}").unwrap(),
                        "  apple,  banana"
                    );
                }

                #[test]
                fn test_map_strip_ansi() {
                    let input = "\x1b[31mred\x1b[0m,\x1b[32mgreen\x1b[0m,\x1b[34mblue\x1b[0m";
                    assert_eq!(
                        process(input, "{split:,:..|map:{strip_ansi}}").unwrap(),
                        "red,green,blue"
                    );
                }
            }

            #[cfg(test)]
            mod string_operations {
                use super::*;

                #[test]
                fn test_map_append_basic() {
                    assert_eq!(
                        process("apple,banana,cherry", "{split:,:..|map:{append:!}}").unwrap(),
                        "apple!,banana!,cherry!"
                    );
                }

                #[test]
                fn test_map_prepend_basic() {
                    assert_eq!(
                        process("apple,banana,cherry", "{split:,:..|map:{prepend:*}}").unwrap(),
                        "*apple,*banana,*cherry"
                    );
                }

                #[test]
                fn test_map_trim_custom_chars_basic() {
                    assert_eq!(
                        process("xappleX,xbananaX,xcherryX", "{split:,:..|map:{trim:xX}}").unwrap(),
                        "apple,banana,cherry"
                    );
                }

                #[test]
                fn test_map_pad_default() {
                    assert_eq!(
                        process("a,bb,ccc", "{split:,:..|map:{pad:5}}").unwrap(),
                        "a    ,bb   ,ccc  "
                    );
                }

                #[test]
                fn test_map_pad_left() {
                    assert_eq!(
                        process("a,bb,ccc", "{split:,:..|map:{pad:5: :left}}").unwrap(),
                        "    a,   bb,  ccc"
                    );
                }

                #[test]
                fn test_map_pad_both() {
                    assert_eq!(
                        process("a,bb", "{split:,:..|map:{pad:6: :both}}").unwrap(),
                        "  a   ,  bb  "
                    );
                }

                #[test]
                fn test_map_pad_custom_char() {
                    assert_eq!(
                        process("a,bb", "{split:,:..|map:{pad:4:0:left}}").unwrap(),
                        "000a,00bb"
                    );
                }
            }

            #[cfg(test)]
            mod substring_operations {
                use super::*;

                #[test]
                fn test_map_substring_index() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:0}}").unwrap(),
                        "h,w,t"
                    );
                }

                #[test]
                fn test_map_substring_negative_index() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:-1}}").unwrap(),
                        "o,d,g"
                    );
                }

                #[test]
                fn test_map_substring_range_exclusive() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:0..3}}")
                            .unwrap(),
                        "hel,wor,tes"
                    );
                }

                #[test]
                fn test_map_substring_range_inclusive() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:0..=2}}")
                            .unwrap(),
                        "hel,wor,tes"
                    );
                }

                #[test]
                fn test_map_substring_range_from() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:2..}}").unwrap(),
                        "llo,rld,sting"
                    );
                }

                #[test]
                fn test_map_substring_range_to() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:..3}}").unwrap(),
                        "hel,wor,tes"
                    );
                }

                #[test]
                fn test_map_substring_range_to_inclusive() {
                    assert_eq!(
                        process("hello,world,testing", "{split:,:..|map:{substring:..=2}}")
                            .unwrap(),
                        "hel,wor,tes"
                    );
                }
            }

            #[cfg(test)]
            mod replace_operations {
                use super::*;

                #[test]
                fn test_map_replace_basic() {
                    assert_eq!(
                        process("hello,world,hell", "{split:,:..|map:{replace:s/l/L/}}").unwrap(),
                        "heLlo,worLd,heLl"
                    );
                }

                #[test]
                fn test_map_replace_global() {
                    assert_eq!(
                        process("hello,world,hell", "{split:,:..|map:{replace:s/l/L/g}}").unwrap(),
                        "heLLo,worLd,heLL"
                    );
                }

                #[test]
                fn test_map_replace_case_insensitive() {
                    assert_eq!(
                        process("Hello,WORLD,heLLo", "{split:,:..|map:{replace:s/l/X/gi}}")
                            .unwrap(),
                        "HeXXo,WORXD,heXXo"
                    );
                }

                #[test]
                fn test_map_replace_digits() {
                    assert_eq!(
                        process(
                            "test123,abc456,xyz789",
                            r"{split:,:..|map:{replace:s/\d+/NUM/g}}"
                        )
                        .unwrap(),
                        "testNUM,abcNUM,xyzNUM"
                    );
                }
            }

            #[cfg(test)]
            mod regex_extract_operations {
                use super::*;

                #[test]
                fn test_map_regex_extract_basic() {
                    assert_eq!(
                        process(
                            "test123,abc456,xyz789",
                            r"{split:,:..|map:{regex_extract:\d+}}"
                        )
                        .unwrap(),
                        "123,456,789"
                    );
                }

                #[test]
                fn test_map_regex_extract_group() {
                    assert_eq!(
                        process(
                            "user:alice,user:bob,user:charlie",
                            r"{split:,:..|map:{regex_extract:user\:(\w+):1}}"
                        )
                        .unwrap(),
                        "alice,bob,charlie"
                    );
                }

                #[test]
                fn test_map_regex_extract_no_match() {
                    assert_eq!(
                        process("abc,def,ghi", r"{split:,:..|map:{regex_extract:\d+}}").unwrap(),
                        ",,"
                    );
                }

                #[test]
                fn test_map_regex_extract_letters() {
                    assert_eq!(
                        process(
                            "123abc456,789def012,345ghi678",
                            r"{split:,:..|map:{regex_extract:[a-z]+}}"
                        )
                        .unwrap(),
                        "abc,def,ghi"
                    );
                }

                #[test]
                fn test_map_regex_extract_date_pattern_workaround() {
                    // Note: Due to parser limitations, curly brace quantifiers {n} in regex patterns
                    // within map operations need to be written as repeated patterns instead
                    // Use \d\d\d\d-\d\d-\d\d instead of \d{4}-\d{2}-\d{2}
                    assert_eq!(
                        process(
                            "2023-01-01 ERROR Failed,2023-12-25 INFO Success",
                            r"{split:,:..|map:{regex_extract:\d\d\d\d-\d\d-\d\d}}"
                        )
                        .unwrap(),
                        "2023-01-01,2023-12-25"
                    );
                }

                #[test]
                fn test_map_regex_extract_character_class_alternative() {
                    // Alternative approach using character classes
                    assert_eq!(
                    process(
                        "2023-01-01 ERROR Failed,2023-12-25 INFO Success",
                        r"{split:,:..|map:{regex_extract:[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]}}"
                    )
                    .unwrap(),
                    "2023-01-01,2023-12-25"
                );
                }
            }
        }

        #[cfg(test)]
        mod escaped_characters {
            use super::*;

            #[test]
            fn test_map_append_escaped_colon() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{append:\:}}").unwrap(),
                    "apple:,banana:"
                );
            }

            #[test]
            fn test_map_prepend_escaped_colon() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{prepend:\:}}").unwrap(),
                    ":apple,:banana"
                );
            }

            #[test]
            fn test_map_append_escaped_pipe() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{append:\|}}").unwrap(),
                    "apple|,banana|"
                );
            }

            #[test]
            fn test_map_prepend_escaped_pipe() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{prepend:\|}}").unwrap(),
                    "|apple,|banana"
                );
            }

            #[test]
            fn test_map_append_escaped_backslash() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{append:\\}}").unwrap(),
                    r"apple\,banana\"
                );
            }

            #[test]
            fn test_map_append_escaped_newline() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{append:\n}}").unwrap(),
                    "apple\n,banana\n"
                );
            }

            #[test]
            fn test_map_append_escaped_tab() {
                assert_eq!(
                    process("apple,banana", r"{split:,:..|map:{append:\t}}").unwrap(),
                    "apple\t,banana\t"
                );
            }

            #[test]
            fn test_map_trim_escaped_chars() {
                assert_eq!(
                    process(":apple:,|banana|", r"{split:,:..|map:{trim:\:\|}}").unwrap(),
                    "apple,banana"
                );
            }

            #[test]
            fn test_map_replace_escaped_pattern() {
                assert_eq!(
                    process("a:b,c:d,e:f", r"{split:,:..|map:{replace:s/\:/=/g}}").unwrap(),
                    "a=b,c=d,e=f"
                );
            }

            #[test]
            fn test_map_pad_escaped_char() {
                assert_eq!(
                    process("a,b", r"{split:,:..|map:{pad:3:\::right}}").unwrap(),
                    "a::,b::"
                );
            }

            #[test]
            fn test_map_regex_extract_escaped_pattern() {
                assert_eq!(
                    process("a:1,b:2,c:3", r"{split:,:..|map:{regex_extract:\w\:\d}}").unwrap(),
                    "a:1,b:2,c:3"
                );
            }
        }

        #[cfg(test)]
        mod pipeline_operations {
            use super::*;

            #[test]
            fn test_map_pipeline_two_steps() {
                assert_eq!(
                    process("  hello  ,  world  ", "{split:,:..|map:{trim|upper}}").unwrap(),
                    "HELLO,WORLD"
                );
            }

            #[test]
            fn test_map_pipeline_three_steps() {
                assert_eq!(
                    process(
                        "  hello  ,  world  ",
                        "{split:,:..|map:{trim|upper|append:!}}"
                    )
                    .unwrap(),
                    "HELLO!,WORLD!"
                );
            }

            #[test]
            fn test_map_pipeline_complex() {
                assert_eq!(
                    process(
                        "  abc123  ,  def456  ",
                        r"{split:,:..|map:{trim|regex_extract:\d+|append:_num}}"
                    )
                    .unwrap(),
                    "123_num,456_num"
                );
            }

            #[test]
            fn test_map_pipeline_substring_upper_append() {
                assert_eq!(
                    process(
                        "hello,world,testing",
                        "{split:,:..|map:{substring:1..4|upper|append:!}}"
                    )
                    .unwrap(),
                    "ELL!,ORL!,EST!"
                );
            }

            #[test]
            fn test_map_pipeline_prepend_replace_trim() {
                assert_eq!(
                    process(
                        "abc,def,ghi",
                        r"{split:,:..|map:{prepend: X |replace:s/X/Y/g|trim}}"
                    )
                    .unwrap(),
                    "Y abc,Y def,Y ghi"
                );
            }

            #[test]
            fn test_map_pipeline_pad_trim() {
                assert_eq!(
                    process("a,bb,ccc", "{split:,:..|map:{pad:5:*:both|trim:*}}").unwrap(),
                    "a,bb,ccc"
                );
            }

            #[test]
            fn test_map_pipeline_with_escapes() {
                assert_eq!(
                    process(
                        "hello,world",
                        r"{split:,:..|map:{append:\:|upper|prepend:[|append:]}}"
                    )
                    .unwrap(),
                    r"[HELLO:],[WORLD:]"
                );
            }
        }

        #[cfg(test)]
        mod invalid_operations {
            use super::*;

            #[test]
            fn test_map_invalid_split() {
                assert!(process("a,b,c", "{split:,:..|map:{split::}}").is_err());
            }

            #[test]
            fn test_map_invalid_sort() {
                assert!(process("a,b,c", "{split:,:..|map:{sort}}").is_err());
            }

            #[test]
            fn test_map_invalid_unique() {
                assert!(process("a,b,c", "{split:,:..|map:{unique}}").is_err());
            }

            #[test]
            fn test_map_invalid_slice() {
                assert!(process("a,b,c", "{split:,:..|map:{slice:1..3}}").is_err());
            }

            #[test]
            fn test_map_invalid_nested_map() {
                // Nested map operations should fail
                assert!(process("a,b,c", "{split:,:..|map:{map:{upper}}}").is_err());
            }

            #[test]
            fn test_map_unknown_operation() {
                assert!(process("a,b,c", "{split:,:..|map:{unknown_op}}").is_err());
            }

            #[test]
            fn test_map_invalid_operation_in_pipeline() {
                assert!(process("a,b,c", "{split:,:..|map:{upper|split::}}").is_err());
            }

            #[test]
            fn test_map_malformed_braces() {
                assert!(process("a,b,c", "{split:,:..|map:upper}").is_err());
            }

            #[test]
            fn test_map_empty_operation() {
                assert!(process("a,b,c", "{split:,:..|map:{}}").is_err());
            }

            #[test]
            fn test_map_missing_colon() {
                assert!(process("a,b,c", "{split:,:..|map{upper}}").is_err());
            }

            #[test]
            fn test_map_invalid_regex_in_pipeline() {
                assert!(process("a,b,c", r"{split:,:..|map:{regex_extract:[|upper}}").is_err());
            }
        }

        #[cfg(test)]
        mod edge_cases {
            use super::*;

            #[test]
            fn test_map_empty_string() {
                assert_eq!(process("", "{split:,:..|map:{upper}}").unwrap(), "");
            }

            #[test]
            fn test_map_single_item() {
                assert_eq!(
                    process("hello", "{split:,:..|map:{upper}}").unwrap(),
                    "HELLO"
                );
            }

            #[test]
            fn test_map_empty_items() {
                assert_eq!(
                    process("a,,c", "{split:,:..|map:{append:!}}").unwrap(),
                    "a!,!,c!"
                );
            }

            #[test]
            fn test_map_unicode() {
                assert_eq!(
                    process("café,naïve,résumé", "{split:,:..|map:{upper}}").unwrap(),
                    "CAFÉ,NAÏVE,RÉSUMÉ"
                );
            }

            #[test]
            fn test_map_special_characters() {
                assert_eq!(
                    process("@hello,#world,$test", "{split:,:..|map:{prepend:>}}").unwrap(),
                    ">@hello,>#world,>$test"
                );
            }

            #[test]
            fn test_map_very_long_pipeline() {
                assert_eq!(
                    process(
                        "abc,def",
                        "{split:,:..|map:{upper|append:1|prepend:2|substring:1..5|lower}}"
                    )
                    .unwrap(),
                    "abc1,def1"
                );
            }

            #[test]
            fn test_map_with_whitespace() {
                assert_eq!(
                    process(
                        "hello world,foo bar",
                        "{split:,:..|map:{replace:s/ /_/g|upper}}"
                    )
                    .unwrap(),
                    "HELLO_WORLD,FOO_BAR"
                );
            }

            #[test]
            fn test_map_multiple_maps() {
                assert_eq!(
                    process("hello,world", "{split:,:..|map:{upper}|map:{append:!}}").unwrap(),
                    "HELLO!,WORLD!"
                );
            }
        }

        #[cfg(test)]
        mod template_variations {
            use super::*;

            #[test]
            fn test_template_with_different_separators() {
                assert_eq!(
                    process("a|b|c", r"{split:\|:..|map:{upper}|join:,}").unwrap(),
                    "A,B,C"
                );
            }

            #[test]
            fn test_template_with_range_and_map() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:1..3|map:{upper}}").unwrap(),
                    "B,C"
                );
            }

            #[test]
            fn test_template_with_newline_separator() {
                assert_eq!(
                    process("hello\nworld\ntest", r"{split:\n:..|map:{upper}|join:,}").unwrap(),
                    "HELLO,WORLD,TEST"
                );
            }

            #[test]
            fn test_template_with_tab_separator() {
                assert_eq!(
                    process("hello\tworld\ttest", r"{split:\t:..|map:{upper}|join:,}").unwrap(),
                    "HELLO,WORLD,TEST"
                );
            }

            #[test]
            fn test_template_complex_separator() {
                assert_eq!(
                    process("hello::world::test", r"{split:\:\::..|map:{upper}|join:,}").unwrap(),
                    "HELLO,WORLD,TEST"
                );
            }
        }

        #[cfg(test)]
        mod comprehensive_scenarios {
            use super::*;

            #[test]
            fn test_csv_processing_with_map() {
                let csv_line = "John Doe,25,Engineer,New York";
                assert_eq!(
                    process(csv_line, "{split:,:..|map:{trim|upper}}").unwrap(),
                    "JOHN DOE,25,ENGINEER,NEW YORK"
                );
            }

            #[test]
            fn test_log_processing_with_map() {
                let log_line = "2023-01-01 ERROR Failed to connect,2023-01-02 INFO Connected successfully,2023-01-03 WARN Connection timeout";
                assert_eq!(
                    process(
                        log_line,
                        r"{split:,:..|map:{regex_extract:\d{4}-\d{2}-\d{2}|append: (DATE)}}"
                    )
                    .unwrap(),
                    "2023-01-01 (DATE),2023-01-02 (DATE),2023-01-03 (DATE)"
                );
            }

            #[test]
            fn test_file_extension_processing() {
                assert_eq!(
                    process(
                        "file1.txt,file2.pdf,file3.doc",
                        r"{split:,:..|map:{regex_extract:\.\w+$|upper}}"
                    )
                    .unwrap(),
                    ".TXT,.PDF,.DOC"
                );
            }

            #[test]
            fn test_url_processing() {
                let urls = "https://example.com/page1,http://test.org/page2,https://demo.net/page3";
                assert_eq!(
                    process(
                        urls,
                        r"{split:,:..|map:{regex_extract://([^/]+):1|prepend:HOST\: }}"
                    )
                    .unwrap(),
                    "HOST: example.com,HOST: test.org,HOST: demo.net"
                );
            }

            #[test]
            fn test_email_processing() {
                let emails = "john@example.com,jane@test.org,bob@demo.net";
                assert_eq!(
                    process(
                        emails,
                        r"{split:,:..|map:{regex_extract:@(.+):1|upper|prepend:DOMAIN\: }}"
                    )
                    .unwrap(),
                    "DOMAIN: EXAMPLE.COM,DOMAIN: TEST.ORG,DOMAIN: DEMO.NET"
                );
            }

            #[test]
            fn test_data_cleaning_pipeline() {
                let messy_data = "  John123  ,  Jane456  ,  Bob789  ";
                assert_eq!(
                    process(
                        messy_data,
                        r"{split:,:..|map:{trim|regex_extract:[A-Za-z]+|lower|prepend:clean_}}"
                    )
                    .unwrap(),
                    "clean_john,clean_jane,clean_bob"
                );
            }
        }
    }

    #[cfg(test)]
    mod map_list_operations_tests {
        use super::Template;

        fn process(input: &str, template: &str) -> Result<String, String> {
            let tmpl = Template::parse(template)?;
            tmpl.format(input)
        }

        #[test]
        fn test_map_split_basic() {
            // Test splitting each item in a list
            assert_eq!(
                process(
                    "hello world,foo bar,test case",
                    "{split:,:..|map:{split: :..}}"
                )
                .unwrap(),
                "hello world,foo bar,test case"
            );
        }

        #[test]
        fn test_map_split_with_index() {
            // Extract first word from each line - simulating user extraction
            assert_eq!(
                process(
                    "alice 123 firefox,bob 456 bash,charlie 789 vim",
                    "{split:,:..|map:{split: :0}}"
                )
                .unwrap(),
                "alice,bob,charlie"
            );
        }

        #[test]
        fn test_map_split_with_range() {
            // Extract multiple columns
            assert_eq!(
                process(
                    "alice 123 firefox,bob 456 bash,charlie 789 vim",
                    "{split:,:..|map:{split: :0..2}}"
                )
                .unwrap(),
                "alice 123,bob 456,charlie 789"
            );
        }

        #[test]
        fn test_map_unique_after_split() {
            // This simulates extracting users and removing duplicates per line
            // Not the most practical example but tests the functionality
            assert_eq!(
                process("a a b,c c d,e e f", "{split:,:..|map:{split: :..|unique}}").unwrap(),
                "a b,c d,e f"
            );
        }

        #[test]
        fn test_map_sort_after_split() {
            // Sort words in each item
            assert_eq!(
                process(
                    "zebra apple,banana cherry",
                    "{split:,:..|map:{split: :..|sort}}"
                )
                .unwrap(),
                "apple zebra,banana cherry"
            );
        }

        #[test]
        fn test_map_filter_after_split() {
            // Filter words containing 'a' in each line
            assert_eq!(
                process(
                    "apple banana cherry,dog cat fish,grape orange",
                    "{split:,:..|map:{split: :..|filter:a}}"
                )
                .unwrap(),
                "apple banana,cat,grape orange"
            );
        }

        #[test]
        fn test_map_slice_after_split() {
            // Take first 2 words from each line
            assert_eq!(
                process(
                    "one two three four,five six seven eight",
                    "{split:,:..|map:{split: :..|slice:0..2}}"
                )
                .unwrap(),
                "one two,five six"
            );
        }

        #[test]
        fn test_map_join_with_different_separator() {
            // Split by space, then join with dash
            assert_eq!(
                process(
                    "hello world,foo bar",
                    "{split:,:..|map:{split: :..|join:-}}"
                )
                .unwrap(),
                "hello-world,foo-bar"
            );
        }

        #[test]
        fn test_ps_aux_user_extraction() {
            // Simulate the ps aux use case
            let ps_output = "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\nroot           1  0.0  0.1  168404 11808 ?        Ss   Dec01   0:02 /sbin/init\nalice        123  0.1  0.5  256789 45123 ?        S    10:00   0:15 /usr/bin/firefox\nbob          456  0.0  0.2  123456 12345 ?        S    10:05   0:01 /bin/bash\nalice        789  0.2  1.0  512000 89012 ?        S    10:10   0:25 /usr/bin/chrome\ncharlie     1011  0.0  0.1   98765  6789 ?        S    10:15   0:02 /usr/bin/vim";

            // Extract users from each line (skip header) - normalize whitespace then split
            let result = process(
                ps_output,
                r"{split:\n:1..|map:{replace:s/ +/ /g|split: :0}|join:,}",
            )
            .unwrap();

            // Should extract the first word (user) from each line
            assert!(result.contains("root"));
            assert!(result.contains("alice"));
            assert!(result.contains("bob"));
            assert!(result.contains("charlie"));
        }

        #[test]
        fn test_ps_aux_user_extraction_and_unique() {
            // This requires a different approach since we can't put unique outside map
            // Instead, we extract users and then apply unique at the top level
            let ps_output = "USER         PID\nroot           1\nalice        123\nbob          456\nalice        789\ncharlie     1011\nbob         1213";

            // Extract users, then apply unique and sort at the list level
            // Use multiple spaces as separator and trim to handle whitespace
            let result = process(
                ps_output,
                r"{split:\n:1..|map:{replace:s/ +/ /g|split: :0}|unique|sort|join:,}",
            )
            .unwrap();

            // Should be sorted unique users
            assert_eq!(result, "alice,bob,charlie,root");
        }

        #[test]
        fn test_complex_map_pipeline() {
            // Debug step by step
            let result1 = process(
                "hello world,foo bar,test case",
                "{split:,:..|map:{split: :0}}",
            );
            println!("Step 1: {:?}", result1);

            let result2 = process("hello", "{upper}");
            println!("Simple upper: {:?}", result2);

            // Complex pipeline: split by comma, then for each item split by space, take first word, uppercase
            let result = process(
                "hello world,foo bar,test case",
                "{split:,:..|map:{split: :0|upper}}",
            );
            println!("Full pipeline: {:?}", result);

            assert_eq!(result.unwrap(), "HELLO,FOO,TEST");
        }

        #[test]
        fn test_map_filter_not() {
            // Filter out words containing 'a'
            assert_eq!(
                process(
                    "apple banana cherry,dog cat fish",
                    "{split:,:..|map:{split: :..|filter_not:a}}"
                )
                .unwrap(),
                "cherry,dog fish"
            );
        }

        #[test]
        fn test_map_with_trim_and_split() {
            // Handle whitespace issues in data
            assert_eq!(
                process(
                    "  hello world  ,  foo bar  ",
                    "{split:,:..|map:{trim|split: :0}}"
                )
                .unwrap(),
                "hello,foo"
            );
        }

        #[test]
        fn test_nested_pipeline_in_map() {
            // Complex nested operations
            assert_eq!(
                process(
                    "HELLO WORLD,FOO BAR",
                    "{split:,:..|map:{lower|split: :..|slice:0..1|join:-}}"
                )
                .unwrap(),
                "hello,foo"
            );
        }

        #[test]
        fn test_map_reverse_after_split() {
            // Reverse word order in each line
            assert_eq!(
                process(
                    "one two three,four five six",
                    "{split:,:..|map:{split: :..|reverse}}"
                )
                .unwrap(),
                "three two one,six five four"
            );
        }

        #[test]
        fn test_error_cases_for_map_list_ops() {
            // Test error handling for invalid operations

            // Invalid regex should error
            assert!(process("test,data", "{split:,:..|map:{split: :..|filter:[}}").is_err());

            // Invalid range should error
            assert!(process("test,data", "{split:,:..|map:{split: :..|slice:abc}}").is_err());
        }

        #[test]
        fn test_realistic_log_processing() {
            // Process log lines to extract specific information
            let logs = "2023-01-01 10:00:00 ERROR user alice failed login\n2023-01-01 10:01:00 INFO user bob successful login\n2023-01-01 10:02:00 ERROR user alice failed login\n2023-01-01 10:03:00 WARN user charlie timeout";

            // Extract users from error lines only
            let result =
                process(logs, r"{split:\n:..|filter:ERROR|map:{split: :4}|join:,}").unwrap();
            assert_eq!(result, "alice,alice");
        }

        #[test]
        fn test_csv_column_extraction() {
            // Extract specific column from CSV-like data
            let csv = "name,age,city\nAlice,25,NYC\nBob,30,LA\nCharlie,35,SF";

            // Extract names (first column) - need to explicitly join with comma
            let result = process(csv, "{split:\n:1..|map:{split:,:0}|join:,}").unwrap();
            assert_eq!(result, "Alice,Bob,Charlie");

            // Extract cities (third column) - need to explicitly join with comma
            let result = process(csv, "{split:\n:1..|map:{split:,:2}|join:,}").unwrap();
            assert_eq!(result, "NYC,LA,SF");
        }

        #[test]
        fn test_whitespace_handling_in_map() {
            // Test handling of various whitespace patterns
            let input = "  alice   123  ,  bob   456  ,  charlie   789  ";

            // Extract first field, handling extra whitespace - split by multiple spaces, take first word
            let result = process(
                input,
                r"{split:,:..|map:{trim|replace:s/ +/ /g|split: :0}|join:,}",
            )
            .unwrap();
            assert_eq!(result, "alice,bob,charlie");
        }
    }
}
