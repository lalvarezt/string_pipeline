use regex::Regex;
mod parser;

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum StringOp {
    Split {
        sep: String,
        range: RangeSpec,
    },
    Join {
        sep: String,
    },
    Replace {
        pattern: String,
        replacement: String,
        flags: String,
    },
    Upper,
    Lower,
    Trim,
    Slice {
        range: RangeSpec,
    },
    Strip {
        chars: String,
    },
    Append {
        suffix: String,
    },
    Prepend {
        prefix: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum RangeSpec {
    Index(isize),
    Range(Option<isize>, Option<isize>, bool), // (start, end, inclusive)
}

pub fn parse_template(template: &str) -> Result<(Vec<StringOp>, bool), String> {
    parser::parse_template(template)
}

fn resolve_index(idx: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    let len_i = len as isize;
    let resolved = if idx < 0 { len_i + idx } else { idx };

    if resolved < 0 {
        0
    } else if resolved > len_i {
        len - 1
    } else {
        resolved as usize
    }
}

fn apply_range<T: Clone>(items: &[T], range: &RangeSpec) -> Vec<T> {
    let len = items.len();
    match range {
        RangeSpec::Index(idx) => {
            if len == 0 {
                return vec![];
            }
            let mut i = resolve_index(*idx, len);
            if i >= len {
                i = len - 1;
            }
            items.get(i).cloned().map_or(vec![], |v| vec![v])
        }
        RangeSpec::Range(start, end, inclusive) => {
            if len == 0 {
                return vec![];
            }
            let s_idx = start.map_or(0, |s| resolve_index(s, len));
            let e_idx = match end {
                Some(e) => {
                    let mut idx = resolve_index(*e, len);
                    if *inclusive {
                        idx = idx.saturating_add(1);
                    }
                    idx
                }
                None => len,
            };
            if s_idx >= len {
                vec![]
            } else {
                items[s_idx..e_idx.min(len)].to_vec()
            }
        }
    }
}

/// Unescape \n, \t, \r, in argument strings
fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => {
                    out.push('\n');
                    chars.next();
                }
                Some('t') => {
                    out.push('\t');
                    chars.next();
                }
                Some('r') => {
                    out.push('\r');
                    chars.next();
                }
                Some(':') => {
                    out.push(':');
                    chars.next();
                }
                Some('\\') => {
                    out.push('\\');
                    chars.next();
                }
                Some('/') => {
                    out.push('/');
                    chars.next();
                }
                Some('|') => {
                    out.push('|');
                    chars.next();
                }
                Some(&next) => {
                    out.push(next);
                    chars.next();
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn apply_ops(input: &str, ops: &[StringOp], debug: bool) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut default_sep = " ".to_string();

    if debug {
        eprintln!("DEBUG: Initial value: {:?}", val);
    }

    for (i, op) in ops.iter().enumerate() {
        if debug {
            eprintln!("DEBUG: Applying operation {}: {:?}", i + 1, op);
        }
        match op {
            StringOp::Split { sep, range } => match &val {
                Value::Str(s) => {
                    let parts: Vec<String> = s.split(sep).map(|s| s.to_string()).collect();
                    default_sep = sep.clone();
                    let result = apply_range(&parts, range);
                    val = Value::List(result);
                }
                Value::List(list) => {
                    let result: Vec<String> = list
                        .iter()
                        .flat_map(|s| {
                            let parts: Vec<String> = s.split(sep).map(|s| s.to_string()).collect();
                            apply_range(&parts, range)
                        })
                        .collect();
                    default_sep = sep.clone();
                    val = Value::List(result);
                }
            },
            StringOp::Slice { range } => match &val {
                Value::Str(s) => {
                    let chars: Vec<char> = s.chars().collect();
                    let result = apply_range(&chars, range);
                    val = Value::Str(result.into_iter().collect());
                }
                Value::List(list) => {
                    let sliced: Vec<String> = list
                        .iter()
                        .map(|s| {
                            let chars: Vec<char> = s.chars().collect();
                            let result = apply_range(&chars, range);
                            result.into_iter().collect()
                        })
                        .collect();
                    val = Value::List(sliced);
                }
            },
            StringOp::Join { sep } => match &val {
                Value::List(list) => {
                    let unescaped_sep = unescape(sep);
                    let joined = if list.is_empty() {
                        String::new()
                    } else {
                        list.join(&unescaped_sep)
                    };
                    val = Value::Str(joined);
                    default_sep = unescaped_sep.clone(); // Update default
                }
                Value::Str(s) => {
                    val = Value::Str(s.clone());
                }
            },
            StringOp::Replace {
                pattern,
                replacement,
                flags,
            } => {
                let mut pattern_to_use = pattern.clone();

                // Build inline flags based on the provided flags
                let mut inline_flags = String::new();
                if flags.contains('i') {
                    inline_flags.push('i');
                }
                if flags.contains('m') {
                    inline_flags.push('m');
                }
                if flags.contains('s') {
                    inline_flags.push('s');
                }
                if flags.contains('x') {
                    inline_flags.push('x');
                }

                // Add inline flags if any are present
                if !inline_flags.is_empty() {
                    pattern_to_use = format!("(?{}){}", inline_flags, pattern_to_use);
                }

                // Compile the regex for use
                let re = match Regex::new(&pattern_to_use) {
                    Ok(re) => re,
                    Err(e) => return Err(format!("Invalid regex pattern: {}", e)),
                };

                let replacement = unescape(replacement);

                match &val {
                    Value::Str(s) => {
                        let s = if flags.contains('g') {
                            re.replace_all(s, replacement.as_str()).to_string()
                        } else {
                            re.replace(s, replacement.as_str()).to_string()
                        };
                        val = Value::Str(s);
                    }
                    Value::List(list) => {
                        let replaced: Vec<String> = list
                            .iter()
                            .map(|s| {
                                if flags.contains('g') {
                                    re.replace_all(s, replacement.as_str()).to_string()
                                } else {
                                    re.replace(s, replacement.as_str()).to_string()
                                }
                            })
                            .collect();
                        val = Value::List(replaced);
                    }
                }
            }
            StringOp::Upper => match &val {
                Value::Str(s) => val = Value::Str(s.to_uppercase()),
                Value::List(list) => {
                    val = Value::List(list.iter().map(|s| s.to_uppercase()).collect())
                }
            },
            StringOp::Lower => match &val {
                Value::Str(s) => val = Value::Str(s.to_lowercase()),
                Value::List(list) => {
                    val = Value::List(list.iter().map(|s| s.to_lowercase()).collect())
                }
            },
            StringOp::Trim => match &val {
                Value::Str(s) => val = Value::Str(s.trim().to_string()),
                Value::List(list) => {
                    val = Value::List(list.iter().map(|s| s.trim().to_string()).collect())
                }
            },
            StringOp::Strip { chars } => {
                let chars: Vec<char> = if chars.trim().is_empty() {
                    vec![' ', '\t', '\n', '\r']
                } else {
                    chars.chars().collect()
                };
                match &val {
                    Value::Str(s) => {
                        val = Value::Str(s.trim_matches(|c| chars.contains(&c)).to_string())
                    }
                    Value::List(list) => {
                        val = Value::List(
                            list.iter()
                                .map(|s| s.trim_matches(|c| chars.contains(&c)).to_string())
                                .collect(),
                        )
                    }
                }
            }
            StringOp::Append { suffix } => match &val {
                Value::Str(s) => val = Value::Str(format!("{}{}", s, suffix)),
                Value::List(list) => {
                    if list.is_empty() {
                        val = Value::List(vec![suffix.clone()]); // Create single-item list
                    } else {
                        val =
                            Value::List(list.iter().map(|s| format!("{}{}", s, suffix)).collect());
                    }
                }
            },
            StringOp::Prepend { prefix } => match &val {
                Value::Str(s) => val = Value::Str(format!("{}{}", prefix, s)),
                Value::List(list) => {
                    if list.is_empty() {
                        val = Value::List(vec![prefix.clone()]); // Create single-item list
                    } else {
                        val =
                            Value::List(list.iter().map(|s| format!("{}{}", prefix, s)).collect());
                    }
                }
            },
        }
        if debug {
            match &val {
                Value::Str(s) => eprintln!("DEBUG: Result: String({:?})", s),
                Value::List(list) => {
                    eprintln!("DEBUG: Result: List with {} items:", list.len());
                    for (idx, item) in list.iter().enumerate() {
                        eprintln!("DEBUG:   [{}]: {:?}", idx, item);
                    }
                }
            }
            eprintln!("DEBUG: ---");
        }
    }

    // Note: If the final value is a List, we join using the last split separator
    // or a space if no split operation was performed
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

pub fn process(input: &str, template: &str) -> Result<String, String> {
    let (ops, debug) = parse_template(template)?;
    apply_ops(input, &ops, debug)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single Operation Tests - Organized by Operation Type
    mod single_operations {
        use super::*;

        mod positive_tests {
            use super::*;

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

            // Strip operation tests
            #[test]
            fn test_strip_basic() {
                assert_eq!(process("xyhelloxy", "{strip:xy}").unwrap(), "hello");
            }

            #[test]
            fn test_strip_single_char() {
                assert_eq!(process("aaahelloaaa", "{strip:a}").unwrap(), "hello");
            }

            #[test]
            fn test_strip_multiple_chars() {
                assert_eq!(process("xyzhellopqr", "{strip:xyzpqr}").unwrap(), "hello");
            }

            #[test]
            fn test_strip_no_chars_to_strip() {
                assert_eq!(process("hello", "{strip:xyz}").unwrap(), "hello");
            }

            #[test]
            fn test_strip_all_chars() {
                assert_eq!(process("aaaa", "{strip:a}").unwrap(), "");
            }

            #[test]
            fn test_strip_empty_chars() {
                assert_eq!(process("hello", "{strip:}").unwrap(), "hello");
            }

            #[test]
            fn test_strip_unicode() {
                assert_eq!(process("🔥hello🔥", "{strip:🔥}").unwrap(), "hello");
            }

            // Slice operation tests
            #[test]
            fn test_slice_index() {
                assert_eq!(process("hello", "{slice:1}").unwrap(), "e");
            }

            #[test]
            fn test_slice_negative_index() {
                assert_eq!(process("hello", "{slice:-1}").unwrap(), "o");
            }

            #[test]
            fn test_slice_range_exclusive() {
                assert_eq!(process("hello", "{slice:1..3}").unwrap(), "el");
            }

            #[test]
            fn test_slice_range_inclusive() {
                assert_eq!(process("hello", "{slice:1..=3}").unwrap(), "ell");
            }

            #[test]
            fn test_slice_range_from() {
                assert_eq!(process("hello", "{slice:2..}").unwrap(), "llo");
            }

            #[test]
            fn test_slice_range_to() {
                assert_eq!(process("hello", "{slice:..3}").unwrap(), "hel");
            }

            #[test]
            fn test_slice_range_to_inclusive() {
                assert_eq!(process("hello", "{slice:..=2}").unwrap(), "hel");
            }

            #[test]
            fn test_slice_full_range() {
                assert_eq!(process("hello", "{slice:..}").unwrap(), "hello");
            }

            #[test]
            fn test_slice_empty_string() {
                assert_eq!(process("", "{slice:0}").unwrap(), "");
            }

            #[test]
            fn test_slice_out_of_bounds() {
                assert_eq!(process("hi", "{slice:5}").unwrap(), "i");
            }

            #[test]
            fn test_slice_unicode() {
                assert_eq!(process("café", "{slice:1..3}").unwrap(), "af");
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
        }

        mod negative_tests {
            use super::*;

            // Split operation negative tests
            #[test]
            fn test_split_invalid_range() {
                assert!(process("a,b,c", "{split:,:abc}").is_err());
            }

            #[test]
            fn test_split_malformed_range() {
                assert!(process("a,b,c", "{split:,:1..abc}").is_err());
            }

            // Join operation negative tests
            #[test]
            fn test_join_on_string_should_work() {
                // Join should work on strings too, treating them as single item lists
                assert_eq!(process("hello", "{join:-}").unwrap(), "hello");
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

            // Slice operation negative tests
            #[test]
            fn test_slice_invalid_range() {
                assert!(process("hello", "{slice:abc}").is_err());
            }

            #[test]
            fn test_slice_malformed_range() {
                assert!(process("hello", "{slice:1..abc}").is_err());
            }

            // Strip operation negative tests
            #[test]
            fn test_strip_missing_argument() {
                // This should be handled gracefully or error
                let result = process("hello", "{strip}");
                assert!(result.is_err());
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
        }
    }

    // Two-Step Pipeline Tests
    mod two_step_pipelines {
        use super::*;

        mod positive_tests {
            use super::*;

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
                assert_eq!(
                    process("a\nb\nc", "{split:\\n:..|join: }").unwrap(),
                    "a b c"
                );
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
                    process("hello,world", "{split:,:..|upper}").unwrap(),
                    "HELLO,WORLD"
                );
            }

            #[test]
            fn test_split_lower() {
                assert_eq!(
                    process("HELLO,WORLD", "{split:,:..|lower}").unwrap(),
                    "hello,world"
                );
            }

            #[test]
            fn test_split_upper_with_index() {
                assert_eq!(
                    process("hello,world,test", "{split:,:1|upper}").unwrap(),
                    "WORLD"
                );
            }

            // Split + Trim operations
            #[test]
            fn test_split_trim() {
                assert_eq!(
                    process(" a , b , c ", "{split:,:..|trim}").unwrap(),
                    "a,b,c"
                );
            }

            #[test]
            fn test_split_trim_with_range() {
                assert_eq!(
                    process(" a , b , c , d ", "{split:,:1..3|trim}").unwrap(),
                    "b,c"
                );
            }

            // Split + Strip operations
            #[test]
            fn test_split_strip() {
                assert_eq!(
                    process("xa,yb,zc", "{split:,:..|strip:xyz}").unwrap(),
                    "a,b,c"
                );
            }

            // Split + Append/Prepend operations
            #[test]
            fn test_split_append() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|append:!}").unwrap(),
                    "a!,b!,c!"
                );
            }

            #[test]
            fn test_split_prepend() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|prepend:->}").unwrap(),
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
                    process("hello,world,test", "{split:,:..|replace:s/l/L/g}").unwrap(),
                    "heLLo,worLd,test"
                );
            }

            #[test]
            fn test_split_replace_with_range() {
                assert_eq!(
                    process("hello,world,test", "{split:,:0..2|replace:s/o/0/g}").unwrap(),
                    "hell0,w0rld"
                );
            }

            // Case + Join operations
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

            // Slice + operations
            #[test]
            fn test_slice_upper() {
                assert_eq!(process("hello", "{slice:1..3|upper}").unwrap(), "EL");
            }

            #[test]
            fn test_slice_append() {
                assert_eq!(
                    process("hello", "{slice:0..3|append:...}").unwrap(),
                    "hel..."
                );
            }

            #[test]
            fn test_slice_replace() {
                assert_eq!(
                    process("hello world", "{slice:6..|replace:s/world/universe/}").unwrap(),
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
            fn test_strip_upper() {
                assert_eq!(process("xyhelloxy", "{strip:xy|upper}").unwrap(), "HELLO");
            }

            #[test]
            fn test_strip_split() {
                assert_eq!(
                    process("xya,b,cxy", "{strip:xy|split:,:..}").unwrap(),
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
                    process("a\tb\tc", "{split:\\t:..|join:\\n}").unwrap(),
                    "a\nb\nc"
                );
            }
        }

        mod negative_tests {
            use super::*;

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
                assert_eq!(process("", "{split:,:..|upper}").unwrap(), "");
            }

            // Invalid range combinations
            #[test]
            fn test_invalid_range_in_pipeline() {
                assert!(process("a,b,c", "{split:,:abc|upper}").is_err());
            }
        }
    }

    // Three-Step Pipeline Tests
    mod three_step_pipelines {
        use super::*;

        mod positive_tests {
            use super::*;

            // Split + Transform + Join patterns
            #[test]
            fn test_split_upper_join() {
                assert_eq!(
                    process("hello,world,test", "{split:,:..|upper|join:-}").unwrap(),
                    "HELLO-WORLD-TEST"
                );
            }

            #[test]
            fn test_split_lower_join() {
                assert_eq!(
                    process("HELLO,WORLD,TEST", "{split:,:..|lower|join:_}").unwrap(),
                    "hello_world_test"
                );
            }

            #[test]
            fn test_split_trim_join() {
                assert_eq!(
                    process(" a , b , c ", r"{split:,:..|trim|join:\|}").unwrap(),
                    "a|b|c"
                );
            }

            #[test]
            fn test_split_append_join() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|append:!|join: }").unwrap(),
                    "a! b! c!"
                );
            }

            #[test]
            fn test_split_prepend_join() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|prepend:->|join:\\n}").unwrap(),
                    "->a\n->b\n->c"
                );
            }

            #[test]
            fn test_split_replace_join() {
                assert_eq!(
                    process("hello,world,test", "{split:,:..|replace:s/l/L/g|join:;}").unwrap(),
                    "heLLo;worLd;test"
                );
            }

            #[test]
            fn test_split_strip_join() {
                assert_eq!(
                    process("xa,yb,zc", "{split:,:..|strip:xyz|join:-}").unwrap(),
                    "a-b-c"
                );
            }

            // Split with range + Transform + Join
            #[test]
            fn test_split_range_upper_join() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:1..3|upper|join:-}").unwrap(),
                    "B-C"
                );
            }

            #[test]
            fn test_split_range_append_join() {
                assert_eq!(
                    process("a,b,c,d,e", "{split:,:0..3|append:_item|join: }").unwrap(),
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
                    process("  hello,world  ", "{trim|split:,:..|upper}").unwrap(),
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
            fn test_slice_split_join() {
                assert_eq!(
                    process("prefix:a,b,c", "{slice:7..|split:,:..|join:-}").unwrap(),
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
                    process(" a , b , c ", "{split:,:..|trim|upper}").unwrap(),
                    "A,B,C"
                );
            }

            #[test]
            fn test_split_strip_lower() {
                assert_eq!(
                    process("XA,YB,ZC", "{split:,:..|strip:XYZ|lower}").unwrap(),
                    "a,b,c"
                );
            }

            #[test]
            fn test_split_replace_append() {
                assert_eq!(
                    process("hello,world", "{split:,:..|replace:s/l/L/g|append:!}").unwrap(),
                    "heLLo!,worLd!"
                );
            }

            // Complex range operations
            #[test]
            fn test_split_range_trim_join() {
                assert_eq!(
                    process(" a , b , c , d ", r"{split:,:1..3|trim|join:\|}").unwrap(),
                    "b|c"
                );
            }

            #[test]
            fn test_slice_append_slice() {
                assert_eq!(
                    process("hello", "{slice:1..4|append:_test|slice:0..5}").unwrap(),
                    "ell_t"
                );
            }

            // Unicode and special character handling
            #[test]
            fn test_unicode_three_step() {
                assert_eq!(
                    process("café,naïve,résumé", "{split:,:..|upper|join:🔥}").unwrap(),
                    "CAFÉ🔥NAÏVE🔥RÉSUMÉ"
                );
            }

            #[test]
            fn test_special_chars_pipeline() {
                assert_eq!(
                    process("a\tb\tc", "{split:\\t:..|prepend:[|append:]|join: }").unwrap(),
                    "[a] [b] [c]"
                );
            }

            // Escape sequence handling
            #[test]
            fn test_escaped_colons_pipeline() {
                assert_eq!(
                    process("a,b,c", "{split:,:..|append:\\:value|join: }").unwrap(),
                    "a:value b:value c:value"
                );
            }

            #[test]
            fn test_escaped_pipes_pipeline() {
                let result = process("test", "{replace:s/test/a\\|b/|split:\\|:..|join:-}");
                assert_eq!(result.unwrap(), "a-b");
            }

            // Complex real-world scenarios
            #[test]
            fn test_csv_processing() {
                assert_eq!(
                    process("Name,Age,City", "{split:,:..|lower|prepend:col_}").unwrap(),
                    "col_name,col_age,col_city"
                );
            }

            #[test]
            fn test_path_processing() {
                assert_eq!(
                    process(
                        "/home/user/documents/file.txt",
                        "{split:/:-1|split:.:..|append:_backup}"
                    )
                    .unwrap(),
                    "file_backup.txt_backup"
                );
            }

            #[test]
            fn test_log_processing() {
                assert_eq!(
                    process("2023-01-01 ERROR Failed", "{split: :1..|join:_|lower}").unwrap(),
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
                let result = process(&input, "{split:,:0..5|append:_num|join:-}").unwrap();
                assert_eq!(result, "0_num-1_num-2_num-3_num-4_num");
            }

            // Deep transformations
            #[test]
            fn test_nested_transformations() {
                assert_eq!(
                    process("  HELLO,WORLD  ", "{trim|split:,:..|lower|prepend:item_}").unwrap(),
                    "item_hello,item_world"
                );
            }
        }

        mod negative_tests {
            use super::*;

            // Invalid three-step combinations
            #[test]
            fn test_invalid_middle_operation() {
                assert!(process("test", "{split:,:..|invalid_op|join:-}").is_err());
            }

            #[test]
            fn test_invalid_final_operation() {
                assert!(process("test", "{split:,:..|upper|invalid_op}").is_err());
            }

            #[test]
            fn test_malformed_three_step() {
                assert!(process("test", "{split:,|upper|}").is_err());
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
                assert!(process("test", "{split:,:..|replace:s/[/invalid/|upper}").is_err());
            }

            #[test]
            fn test_invalid_range_in_three_step() {
                assert!(process("a,b,c", "{split:,:abc|upper|join:-}").is_err());
            }

            #[test]
            fn test_empty_results_propagation() {
                assert_eq!(process("", "{split:,:..|upper|join:-}").unwrap(), "");
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
            let result = process("hello,world", "{!split:,:..|upper}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "HELLO,WORLD");
        }

        #[test]
        fn test_debug_flag_three_step() {
            let result = process("hello,world", "{!split:,:..|upper|join:-}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "HELLO-WORLD");
        }

        #[test]
        fn test_debug_flag_complex_pipeline() {
            let result = process("  a , b , c  ", "{!trim|split:,:..|trim|upper|join:_}");
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
        fn test_debug_flag_with_slice() {
            let result = process("hello", "{!slice:1..3}");
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
        fn test_debug_flag_with_strip() {
            let result = process("xyhelloxy", "{!strip:xy}");
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
            let result = process("hello world test", "{!split: :..|upper|join:_|lower}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello_world_test");
        }

        #[test]
        fn test_debug_flag_regex_operations() {
            let result = process("test123", "{!replace:s/\\d+/XXX/}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "testXXX");
        }

        #[test]
        fn test_debug_flag_boundary_conditions() {
            let result = process("a", "{!slice:-1}");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "a");
        }
    }
}
