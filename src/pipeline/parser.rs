//! Template parsing implementation.
//!
//! This module contains the parser for string pipeline templates, converting
//! template syntax into executable operation sequences. The parser uses the
//! Pest parser generator for robust syntax handling with comprehensive error reporting.
//!
//! The parser supports the full template syntax including operations, ranges,
//! escape sequences, and debug flags.
//!

use pest::Parser;
use pest_derive::Parser;
use smallvec::SmallVec;

use super::{PadDirection, RangeSpec, SortDirection, StringOp, TrimDirection};

// Import the new template section types
use super::template::TemplateSection;

// Common separator constant to avoid repeated allocations
const SPACE_SEP: &str = " ";

/// Pest parser for template syntax.
///
/// This parser handles the complete template grammar defined in `template.pest`,
/// including operations, arguments, ranges, and escape sequences.
#[derive(Parser)]
#[grammar = "pipeline/template.pest"]
struct TemplateParser;

/// Parses a template string into operations and debug flag.
///
/// This is the main entry point for template parsing. It processes the complete
/// template syntax and returns a sequence of operations along with any debug settings.
///
/// # Arguments
///
/// * `template` - The template string to parse
///
/// # Returns
///
/// * `Ok((Vec<StringOp>, bool))` - Operations and debug flag
/// * `Err(String)` - Parse error with detailed description
///
/// # Errors
///
/// Returns an error if:
/// - Template syntax is malformed
/// - Unknown operations are used
/// - Arguments are missing or invalid
/// - Range specifications are incorrect
/// - Regex patterns are invalid
///
/// # Examples
///
/// ```rust
/// // This is an internal function used by Template::parse()
/// // let (ops, debug) = parse_template("{upper|trim}").unwrap();
/// // assert_eq!(ops.len(), 2);
/// // assert!(!debug);
/// ```
pub fn parse_template(template: &str) -> Result<(Vec<StringOp>, bool), String> {
    let pairs = TemplateParser::parse(Rule::template, template)
        .map_err(|e| format!("Parse error: {e}"))?
        .next()
        .unwrap();

    // Heuristic: reserve enough space for `|`-separated operations but avoid gross
    // over-allocation for medium templates.  Count of `|` is cheap (single pass
    // over the input) and gives an upper bound on the number of operations.
    let pipe_count = template.as_bytes().iter().filter(|&&b| b == b'|').count();

    let estimated_capacity = if template.len() < 50 {
        4 // Simple templates typically have 1-4 operations
    } else if template.len() < 150 {
        8 // Medium templates typically have 4-8 operations
    } else {
        16 // Complex templates might have 8+ operations
    };

    let mut ops = Vec::with_capacity(std::cmp::min(estimated_capacity, pipe_count + 1));
    let mut debug = false;

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::operation_list => {
                for op_pair in pair.into_inner() {
                    let inner = op_pair.into_inner().next().unwrap();
                    ops.push(parse_operation(inner)?);
                }
            }
            Rule::debug_flag => {
                debug = true;
            }
            _ => {}
        }
    }

    Ok((ops, debug))
}

/// Parses a multi-template string containing mixed literal text and template sections.
///
/// This function processes strings that contain both literal text and template operations,
/// creating a sequence of sections that can be processed with caching support.
///
/// # Arguments
///
/// * `template` - The multi-template string to parse
///
/// # Returns
///
/// * `Ok((Vec<TemplateSection>, bool))` - Template sections and debug flag
/// * `Err(String)` - Parse error with detailed description
///
/// # Examples
///
/// ```rust
/// // This is an internal function used by MultiTemplate::parse()
/// // let (sections, debug) = parse_multi_template("Hello {upper} world").unwrap();
/// // assert_eq!(sections.len(), 3); // "Hello ", upper operation, " world"
/// ```
pub fn parse_multi_template(template: &str) -> Result<(Vec<TemplateSection>, bool), String> {
    let mut sections = Vec::new();
    let mut current_literal = String::new();
    let mut chars = template.chars().peekable();
    let mut debug = false;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Found start of template section

            // Save any accumulated literal text
            if !current_literal.is_empty() {
                sections.push(TemplateSection::Literal(std::mem::take(
                    &mut current_literal,
                )));
            }

            // Find the matching closing brace
            let mut brace_count = 1;
            let mut template_content = String::new();

            for inner_ch in chars.by_ref() {
                if inner_ch == '{' {
                    brace_count += 1;
                    template_content.push(inner_ch);
                } else if inner_ch == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        break; // Found matching closing brace
                    } else {
                        template_content.push(inner_ch);
                    }
                } else {
                    template_content.push(inner_ch);
                }
            }

            if brace_count > 0 {
                return Err("Unclosed template brace".to_string());
            }

            // Parse the template content
            let full_template = format!("{{{template_content}}}");
            let (ops, section_debug) = parse_template(&full_template)?;
            if section_debug {
                debug = true; // If any section has debug enabled, enable for the whole multi-template
            }

            sections.push(TemplateSection::Template(ops));
        } else {
            // Regular character, add to current literal
            current_literal.push(ch);
        }
    }

    // Add any remaining literal text
    if !current_literal.is_empty() {
        sections.push(TemplateSection::Literal(std::mem::take(
            &mut current_literal,
        )));
    }

    Ok((sections, debug))
}

/// Parses a single operation from a parse tree node.
///
/// Converts a parsed operation node into the corresponding `StringOp` variant,
/// handling all supported operations and their arguments.
///
/// # Arguments
///
/// * `pair` - Pest parse tree node representing an operation
///
/// # Returns
///
/// * `Ok(StringOp)` - The parsed operation
/// * `Err(String)` - Parse error description
///
/// # Errors
///
/// Returns an error if:
/// - Operation arguments are invalid
/// - Range specifications are malformed
/// - Regex patterns fail to compile
/// - Required arguments are missing
fn parse_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    match pair.as_rule() {
        Rule::shorthand_range => {
            let range = parse_range_spec(pair)?;
            Ok(StringOp::Split {
                sep: SPACE_SEP.to_string(),
                range,
            })
        }
        Rule::shorthand_index => {
            let idx = pair.as_str().parse().unwrap();
            Ok(StringOp::Split {
                sep: SPACE_SEP.to_string(),
                range: RangeSpec::Index(idx),
            })
        }
        Rule::split => {
            let mut parts = pair.into_inner();
            let sep_part = parts.next().unwrap();
            let sep = process_arg(sep_part.as_str());
            let range = if let Some(range_part) = parts.next() {
                parse_range_spec(range_part)?
            } else {
                RangeSpec::Range(None, None, false)
            };
            Ok(StringOp::Split { sep, range })
        }
        Rule::join => Ok(StringOp::Join {
            sep: extract_single_arg(pair)?,
        }),
        Rule::substring => Ok(StringOp::Substring {
            range: extract_range_arg(pair)?,
        }),
        Rule::replace => {
            let sed_parts = parse_sed_string(pair.into_inner().next().unwrap())?;
            Ok(StringOp::Replace {
                pattern: sed_parts.0,
                replacement: sed_parts.1,
                flags: sed_parts.2,
            })
        }
        Rule::upper => Ok(StringOp::Upper),
        Rule::lower => Ok(StringOp::Lower),
        Rule::trim => {
            let chars = parse_trim_chars(pair.clone());
            let direction = parse_trim_direction(pair);
            Ok(StringOp::Trim { chars, direction })
        }
        Rule::append => Ok(StringOp::Append {
            suffix: extract_single_arg(pair)?,
        }),
        Rule::prepend => Ok(StringOp::Prepend {
            prefix: extract_single_arg(pair)?,
        }),
        Rule::surround => Ok(StringOp::Surround {
            text: extract_single_arg(pair)?,
        }),
        Rule::quote => Ok(StringOp::Surround {
            text: extract_single_arg(pair)?,
        }),
        Rule::strip_ansi => Ok(StringOp::StripAnsi),
        Rule::filter => Ok(StringOp::Filter {
            pattern: extract_single_arg_raw(pair)?,
        }),
        Rule::filter_not => Ok(StringOp::FilterNot {
            pattern: extract_single_arg_raw(pair)?,
        }),
        Rule::slice => Ok(StringOp::Slice {
            range: extract_range_arg(pair)?,
        }),
        Rule::sort => Ok(StringOp::Sort {
            direction: parse_sort_direction(pair),
        }),
        Rule::reverse => Ok(StringOp::Reverse),
        Rule::unique => Ok(StringOp::Unique),
        Rule::pad => parse_pad_operation(pair),
        Rule::regex_extract | Rule::map_regex_extract => parse_regex_extract_operation(pair),
        Rule::map => parse_map_operation(pair),
        _ => Err(format!("Unsupported operation: {:?}", pair.as_rule())),
    }
}

/// Extracts and processes a single argument from an operation.
///
/// Takes the first inner node as an argument and processes escape sequences.
///
/// # Arguments
///
/// * `pair` - Parse tree node containing the argument
///
/// # Returns
///
/// * `Ok(String)` - Processed argument with escape sequences resolved
/// * `Err(String)` - Error if argument is missing
fn extract_single_arg(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    let inner = pair.into_inner().next().unwrap();
    Ok(process_arg(inner.as_str()))
}

/// Extracts a single argument without escape sequence processing.
///
/// Used for regex patterns and other contexts where literal strings are needed.
///
/// # Arguments
///
/// * `pair` - Parse tree node containing the argument
///
/// # Returns
///
/// * `Ok(String)` - Raw argument string
/// * `Err(String)` - Error if argument is missing
fn extract_single_arg_raw(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    Ok(pair.into_inner().next().unwrap().as_str().to_string())
}

/// Extracts a range specification argument.
///
/// Parses the range specification from the operation arguments.
///
/// # Arguments
///
/// * `pair` - Parse tree node containing the range
///
/// # Returns
///
/// * `Ok(RangeSpec)` - Parsed range specification
/// * `Err(String)` - Error if range is malformed
fn extract_range_arg(pair: pest::iterators::Pair<Rule>) -> Result<RangeSpec, String> {
    parse_range_spec(pair.into_inner().next().unwrap())
}

/// Parses trim operation characters from arguments.
///
/// Determines which characters to trim based on the operation arguments,
/// distinguishing between character specifications and direction arguments.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the trim operation
///
/// # Returns
///
/// The characters to trim, or empty string for default whitespace trimming.
#[inline(always)]
fn parse_trim_chars(pair: pest::iterators::Pair<Rule>) -> String {
    let mut parts = pair.into_inner();

    // Fast path: if no arguments, return empty string
    let first = match parts.next() {
        Some(p) => p,
        None => return String::new(),
    };

    // Check if there's a second argument
    if let Some(_second) = parts.next() {
        // If there are two arguments, first is chars, second is direction
        // (regardless of what the first argument contains)
        first.as_str().to_string()
    } else {
        // Only one argument - check if it's a direction or chars
        match first.as_str() {
            "left" | "right" | "both" => String::new(), // It's a direction, no chars
            chars => chars.to_string(),                 // It's chars
        }
    }
}

/// Parses trim operation direction from arguments.
///
/// Determines the trimming direction (left, right, or both) from the operation arguments.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the trim operation
///
/// # Returns
///
/// The trim direction, defaulting to `Both` if not specified.
fn parse_trim_direction(pair: pest::iterators::Pair<Rule>) -> TrimDirection {
    let mut parts = pair.into_inner();

    // Check first argument
    if let Some(first) = parts.next() {
        // Check if there's a second argument
        if let Some(second) = parts.next() {
            // If there are two arguments, second is the direction
            match second.as_str() {
                "left" => return TrimDirection::Left,
                "right" => return TrimDirection::Right,
                "both" => return TrimDirection::Both,
                _ => return TrimDirection::Both,
            }
        } else {
            // Only one argument - check if it's a direction
            match first.as_str() {
                "left" => return TrimDirection::Left,
                "right" => return TrimDirection::Right,
                "both" => return TrimDirection::Both,
                _ => return TrimDirection::Both,
            }
        }
    }

    TrimDirection::Both
}

/// Parses sort operation direction from arguments.
///
/// Determines the sort direction (ascending or descending) from the operation arguments.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the sort operation
///
/// # Returns
///
/// The sort direction, defaulting to ascending if not specified.
fn parse_sort_direction(pair: pest::iterators::Pair<Rule>) -> SortDirection {
    if let Some(p) = pair.into_inner().next() {
        match p.as_str() {
            "desc" => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    } else {
        SortDirection::Asc
    }
}

/// Parses a pad operation with width, character, and direction arguments.
///
/// Processes the padding operation arguments to extract width, padding character,
/// and padding direction with appropriate defaults.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the pad operation
///
/// # Returns
///
/// * `Ok(StringOp::Pad)` - Parsed pad operation
/// * `Err(String)` - Error if width is invalid
fn parse_pad_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let mut parts = pair.into_inner();
    let width = parts
        .next()
        .unwrap()
        .as_str()
        .parse()
        .map_err(|_| "Invalid padding width")?;

    let char = if let Some(char_part) = parts.next() {
        let processed = process_arg(char_part.as_str());
        processed.chars().next().unwrap_or(' ')
    } else {
        ' '
    };

    let direction = parts
        .next()
        .map(|p| match p.as_str() {
            "left" => PadDirection::Left,
            "right" => PadDirection::Right,
            "both" => PadDirection::Both,
            _ => PadDirection::Right,
        })
        .unwrap_or(PadDirection::Right);

    Ok(StringOp::Pad {
        width,
        char,
        direction,
    })
}

/// Parses a regex extract operation with pattern and optional group.
///
/// Processes regex extraction arguments to extract the pattern and optional
/// capture group number.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the regex extract operation
///
/// # Returns
///
/// * `Ok(StringOp::RegexExtract)` - Parsed regex extract operation
/// * `Err(String)` - Error if arguments are invalid
fn parse_regex_extract_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let mut parts = pair.into_inner();
    let pattern = parts.next().unwrap().as_str().to_string();
    let group = parts.next().and_then(|p| p.as_str().parse().ok());
    Ok(StringOp::RegexExtract { pattern, group })
}

/// Parses a map operation with nested operation list.
///
/// Processes the map operation to extract the nested operations that should
/// be applied to each list item.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the map operation
///
/// # Returns
///
/// * `Ok(StringOp::Map)` - Parsed map operation with nested operations
/// * `Err(String)` - Error if nested operations are invalid
fn parse_map_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let map_op_pair = pair.into_inner().next().unwrap();
    let operation_list_pair = map_op_pair.into_inner().next().unwrap();

    let mut operations: SmallVec<[StringOp; 8]> = SmallVec::new();
    for op_pair in operation_list_pair.into_inner() {
        let inner_op_pair = op_pair.into_inner().next().unwrap();
        operations.push(parse_map_inner_operation(inner_op_pair)?);
    }

    Ok(StringOp::Map {
        operations: Box::new(operations),
    })
}

/// Parses operations that can be used inside map blocks.
///
/// Handles the subset of operations that are valid within map contexts,
/// including both string operations and list operations that work on individual items.
///
/// # Arguments
///
/// * `pair` - Parse tree node for the map inner operation
///
/// # Returns
///
/// * `Ok(StringOp)` - Parsed operation valid for map context
/// * `Err(String)` - Error if operation is invalid or unsupported in map
fn parse_map_inner_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    match pair.as_rule() {
        // String operations (existing)
        Rule::substring => Ok(StringOp::Substring {
            range: extract_range_arg(pair)?,
        }),
        Rule::replace => {
            let sed_parts = parse_sed_string(pair.into_inner().next().unwrap())?;
            Ok(StringOp::Replace {
                pattern: sed_parts.0,
                replacement: sed_parts.1,
                flags: sed_parts.2,
            })
        }
        Rule::append => Ok(StringOp::Append {
            suffix: extract_single_arg(pair)?,
        }),
        Rule::prepend => Ok(StringOp::Prepend {
            prefix: extract_single_arg(pair)?,
        }),
        Rule::surround => Ok(StringOp::Surround {
            text: extract_single_arg(pair)?,
        }),
        Rule::quote => Ok(StringOp::Surround {
            text: extract_single_arg(pair)?,
        }),
        Rule::upper => Ok(StringOp::Upper),
        Rule::lower => Ok(StringOp::Lower),
        Rule::trim => {
            let chars = parse_trim_chars(pair.clone());
            let direction = parse_trim_direction(pair);
            Ok(StringOp::Trim { chars, direction })
        }
        Rule::pad => parse_pad_operation(pair),
        Rule::reverse => Ok(StringOp::Reverse),
        Rule::strip_ansi => Ok(StringOp::StripAnsi),
        Rule::map_regex_extract => parse_regex_extract_operation(pair),

        // List operations (new)
        Rule::map_split => {
            let mut parts = pair.into_inner();
            let sep_part = parts.next().unwrap();
            let sep = process_arg(sep_part.as_str());
            let range = if let Some(range_part) = parts.next() {
                parse_range_spec(range_part)?
            } else {
                RangeSpec::Range(None, None, false)
            };
            Ok(StringOp::Split { sep, range })
        }
        Rule::map_join => Ok(StringOp::Join {
            sep: extract_single_arg(pair)?,
        }),
        Rule::map_slice => Ok(StringOp::Slice {
            range: extract_range_arg(pair)?,
        }),
        Rule::map_sort => Ok(StringOp::Sort {
            direction: parse_sort_direction(pair),
        }),
        Rule::map_unique => Ok(StringOp::Unique),
        Rule::map_filter => Ok(StringOp::Filter {
            pattern: extract_single_arg_raw(pair)?,
        }),
        Rule::map_filter_not => Ok(StringOp::FilterNot {
            pattern: extract_single_arg_raw(pair)?,
        }),

        _ => Err(format!("Unsupported map operation: {:?}", pair.as_rule())),
    }
}

/// Processes escape sequences in argument strings.
///
/// Converts escape sequences like `\n`, `\t`, `\:`, etc. into their literal
/// characters, enabling special character support in template arguments.
///
/// # Arguments
///
/// * `s` - The raw argument string to process
///
/// # Returns
///
/// The processed string with escape sequences converted to literal characters.
///
/// # Supported Escape Sequences
///
/// - `\n` - Newline
/// - `\t` - Tab
/// - `\r` - Carriage return
/// - `\:` - Literal colon
/// - `\|` - Literal pipe
/// - `\\` - Literal backslash
/// - `\/` - Literal forward slash
/// - `\{` - Literal opening brace
/// - `\}` - Literal closing brace
#[inline(always)]
fn process_arg(s: &str) -> String {
    // Fast path: no escape sequences, return owned string directly
    if !s.contains('\\') {
        return s.to_string();
    }

    // Optimized path: pre-allocate with exact capacity and use efficient iteration
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Handle escape sequence
            match bytes[i + 1] {
                b'n' => result.push('\n'),
                b't' => result.push('\t'),
                b'r' => result.push('\r'),
                b':' => result.push(':'),
                b'|' => result.push('|'),
                b'\\' => result.push('\\'),
                b'/' => result.push('/'),
                b'{' => result.push('{'),
                b'}' => result.push('}'),
                other => result.push(other as char),
            }
            i += 2;
        } else if bytes[i] == b'\\' {
            // Backslash at end of string
            result.push('\\');
            i += 1;
        } else {
            // Regular character
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// Parses sed-style replacement strings.
///
/// Extracts pattern, replacement, and flags from sed-style syntax like `s/pattern/replacement/flags`.
///
/// # Arguments
///
/// * `pair` - Parse tree node containing the sed string
///
/// # Returns
///
/// * `Ok((pattern, replacement, flags))` - Parsed sed components
/// * `Err(String)` - Error if sed syntax is invalid
///
/// # Errors
///
/// Returns an error if the pattern is empty (which would be invalid in regex).
fn parse_sed_string(pair: pest::iterators::Pair<Rule>) -> Result<(String, String, String), String> {
    let mut parts = pair.into_inner();

    let pattern_str = parts.next().unwrap().as_str();
    let replacement_str = parts.next().unwrap().as_str();
    let flags_opt = parts.next();

    if pattern_str.is_empty() {
        return Err("Empty pattern in sed string".to_string());
    }

    Ok((
        pattern_str.to_string(),
        replacement_str.to_string(),
        flags_opt.map_or_else(String::new, |p| p.as_str().to_string()),
    ))
}

/// Parses range specifications from template syntax.
///
/// Converts range syntax like `1..3`, `..5`, `2..`, etc. into `RangeSpec` values
/// with proper handling of inclusive/exclusive bounds and negative indexing.
///
/// # Arguments
///
/// * `pair` - Parse tree node containing the range specification
///
/// # Returns
///
/// * `Ok(RangeSpec)` - Parsed range specification
/// * `Err(String)` - Error if range syntax is invalid
///
/// # Supported Range Types
///
/// - Single index: `1`, `-1`
/// - Exclusive range: `1..3`
/// - Inclusive range: `1..=3`
/// - Open start: `..3`, `..=3`
/// - Open end: `2..`
/// - Full range: `..`
fn parse_range_spec(pair: pest::iterators::Pair<Rule>) -> Result<RangeSpec, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::range_inclusive => {
            let mut parts = inner.into_inner();
            let start = parts.next().and_then(|p| p.as_str().parse().ok());
            let end = parts.next().and_then(|p| p.as_str().parse().ok());
            Ok(RangeSpec::Range(start, end, true))
        }
        Rule::range_exclusive => {
            let mut parts = inner.into_inner();
            let start = parts.next().and_then(|p| p.as_str().parse().ok());
            let end = parts.next().and_then(|p| p.as_str().parse().ok());
            Ok(RangeSpec::Range(start, end, false))
        }
        Rule::range_from => {
            let start = inner.into_inner().next().unwrap().as_str().parse().ok();
            Ok(RangeSpec::Range(start, None, false))
        }
        Rule::range_to => {
            let end = inner.into_inner().next().unwrap().as_str().parse().ok();
            Ok(RangeSpec::Range(None, end, false))
        }
        Rule::range_to_inclusive => {
            let end = inner.into_inner().next().unwrap().as_str().parse().ok();
            Ok(RangeSpec::Range(None, end, true))
        }
        Rule::range_full => Ok(RangeSpec::Range(None, None, false)),
        Rule::index => {
            let idx_str = inner.into_inner().next().unwrap().as_str();
            let idx = idx_str
                .parse()
                .map_err(|_| format!("Invalid index: {idx_str}"))?;
            Ok(RangeSpec::Index(idx))
        }
        _ => Err(format!("Unknown range spec: {:?}", inner.as_rule())),
    }
}
