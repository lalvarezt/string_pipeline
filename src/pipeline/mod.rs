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

pub fn parse_template(template: &str) -> Result<Vec<StringOp>, String> {
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

pub fn apply_ops(input: &str, ops: &[StringOp]) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut default_sep = " ".to_string(); // Clear default
    for op in ops {
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
                let pattern_to_use = pattern.clone();

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
    let ops = parse_template(template)?;
    apply_ops(input, &ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_newline() {
        let input = "a,b,c";
        assert_eq!(process(input, r"{split:,:..|join:\\n}").unwrap(), "a\nb\nc");
    }

    #[test]
    fn test_split_index() {
        let input = "a,b,c";
        assert_eq!(process(input, "{split:,:1}").unwrap(), "b");
        assert_eq!(process(input, "{split:,:-1}").unwrap(), "c");
        assert_eq!(process(input, "{split:,:-3}").unwrap(), "a");
        assert_eq!(process(input, "{split:,:-4}").unwrap(), "a"); // clamped to 0
        assert_eq!(process(input, "{split:,:4}").unwrap(), "c"); // clamped to last
    }

    #[test]
    fn test_split_range() {
        let input = "a,b,c,d,e";
        assert_eq!(process(input, "{split:,:1..3}").unwrap(), "b,c");
        assert_eq!(process(input, "{split:,:1..=3}").unwrap(), "b,c,d");
        assert_eq!(process(input, "{split:,:..2}").unwrap(), "a,b");
        assert_eq!(process(input, "{split:,:2..}").unwrap(), "c,d,e");
        assert_eq!(process(input, "{split:,:..=2}").unwrap(), "a,b,c");
        assert_eq!(process(input, "{split:,:..}").unwrap(), "a,b,c,d,e");
        assert_eq!(process(input, "{split:,:-2..}").unwrap(), "d,e");
        assert_eq!(process(input, "{split:,:-3..-1}").unwrap(), "c,d");
        assert_eq!(process(input, "{split:,:-3..=-1}").unwrap(), "c,d,e");
    }

    #[test]
    fn test_split_range_and_join() {
        let input = "a,b,c,d,e";
        assert_eq!(process(input, "{split:,:1..3|join:-}").unwrap(), "b-c");
        assert_eq!(process(input, "{split:,:-2..|join:_}").unwrap(), "d_e");
        assert_eq!(process(input, "{split:,:1|join:-}").unwrap(), "b");
        assert_eq!(process(input, "{split:,:..|join:-}").unwrap(), "a-b-c-d-e");
    }

    #[test]
    fn test_replace() {
        let input = "foo bar foo";
        let template = "{replace:s/foo/baz/g}";
        assert_eq!(process(input, template).unwrap(), "baz bar baz");
    }

    #[test]
    fn test_upper() {
        let input = "hello";
        let template = "{upper}";
        assert_eq!(process(input, template).unwrap(), "HELLO");
    }

    #[test]
    fn test_lower() {
        let input = "HELLO";
        let template = "{lower}";
        assert_eq!(process(input, template).unwrap(), "hello");
    }

    #[test]
    fn test_trim() {
        let input = "  hello  ";
        let template = "{trim}";
        assert_eq!(process(input, template).unwrap(), "hello");
    }

    #[test]
    fn test_slice_range() {
        let input = "abcdef";
        assert_eq!(process(input, "{slice:1..3}").unwrap(), "bc");
        assert_eq!(process(input, "{slice:1..=3}").unwrap(), "bcd");
        assert_eq!(process(input, "{slice:..2}").unwrap(), "ab");
        assert_eq!(process(input, "{slice:2..}").unwrap(), "cdef");
        assert_eq!(process(input, "{slice:..=2}").unwrap(), "abc");
        assert_eq!(process(input, "{slice:..}").unwrap(), "abcdef");
        assert_eq!(process(input, "{slice:-3..}").unwrap(), "def");
        assert_eq!(process(input, "{slice:-3..-1}").unwrap(), "de");
        assert_eq!(process(input, "{slice:-3..=-1}").unwrap(), "def");
        assert_eq!(process(input, "{slice:2}").unwrap(), "c");
    }

    #[test]
    fn test_strip() {
        let input = "xyhelloxy";
        let template = "{strip:xy}";
        assert_eq!(process(input, template).unwrap(), "hello");
    }

    #[test]
    fn test_append() {
        let input = "foo";
        let template = "{append:bar}";
        assert_eq!(process(input, template).unwrap(), "foobar");
    }

    #[test]
    fn test_prepend() {
        let input = "bar";
        let template = "{prepend:foo}";
        assert_eq!(process(input, template).unwrap(), "foobar");
    }

    #[test]
    fn test_append_prepend_list() {
        let input = " a, b,c , d , e ";
        assert_eq!(
            process(input, "{split:,:..|trim|append:!}").unwrap(),
            "a!,b!,c!,d!,e!"
        );
        assert_eq!(
            process(input, "{split:,:..|trim|prepend:_}").unwrap(),
            "_a,_b,_c,_d,_e"
        );
    }

    #[test]
    fn test_chain() {
        let input = "first,second,third";
        // Original test
        let template = "{split:,:1|replace:s/second/hello/|upper}";
        assert_eq!(process(input, template).unwrap(), "HELLO");

        // Split, replace, lower
        let template = "{split:,:1|replace:s/second/hello/|lower}";
        assert_eq!(process(input, template).unwrap(), "hello");

        // Split, replace, trim (no effect, but test chain)
        let template = "{split:,:1|replace:s/second/ hello /|trim}";
        assert_eq!(process(input, template).unwrap(), "hello");

        // Split, upper, append
        let template = "{split:,:2|upper|append:!}";
        assert_eq!(process(input, template).unwrap(), "THIRD!");

        // Split, lower, prepend
        let template = r"{split:,:0|lower|prepend:word\: }";
        assert_eq!(process(input, template).unwrap(), "word: first");

        // Split range, join, upper
        let template = "{split:,:0..2|join:_|upper}";
        assert_eq!(process(input, template).unwrap(), "FIRST_SECOND");

        // Split range, join, replace, lower
        let template = "{split:,:0..2|join:-|replace:s/first/1/|lower}";
        assert_eq!(process(input, template).unwrap(), "1-second");

        // Split, replace, slice (get first 2 chars)
        let template = "{split:,:1|replace:s/second/hello/|slice:0..2}";
        assert_eq!(process(input, template).unwrap(), "he");

        // Split, replace, slice (last 2 chars)
        let template = "{split:,:1|replace:s/second/hello/|slice:-2..}";
        assert_eq!(process(input, template).unwrap(), "lo");

        // Split, strip, upper
        let input = "  first , second , third  ";
        let template = "{split:,:1|strip: |upper}";
        assert_eq!(process(input, template).unwrap(), "SECOND");

        // Split, join, append, upper
        let input = "a,b,c";
        let template = "{split:,:..|join:-|append:! |upper}";
        assert_eq!(process(input, template).unwrap(), "A-B-C! ");

        // Split, join, prepend, lower
        let template = r"{split:,:..|join:_|prepend:joined\: }";
        assert_eq!(process(input, template).unwrap(), "joined: a_b_c");

        // Split, trim, join, replace, upper
        let input = "  x, y ,z ";
        let template = "{split:,:..|trim|join:_|upper}";
        assert_eq!(process(input, template).unwrap(), "X_Y_Z");

        // Split, join, replace, slice
        let input = "foo,bar,baz";
        let template = "{split:,:..|join:-|replace:s/bar/xxx/|slice:0..7}";
        assert_eq!(process(input, template).unwrap(), "foo-xxx");

        // Split, join, replace, slice, lower
        let template = "{split:,:..|join:-|replace:s/bar/XXX/|slice:0..7|lower}";
        assert_eq!(process(input, template).unwrap(), "foo-xxx");
    }

    #[test]
    fn test_append_colons() {
        let input = "foo";
        // Colon at start
        assert_eq!(process(input, r"{append:\:bar}").unwrap(), "foo:bar");
        // Colon at end
        assert_eq!(process(input, r"{append:bar\:}").unwrap(), "foobar:");
        // Colon in middle
        assert_eq!(process(input, r"{append:ba\:r}").unwrap(), "fooba:r");
        // Multiple colons
        assert_eq!(
            process(input, r"{append:\:bar\:baz\:qux}").unwrap(),
            "foo:bar:baz:qux"
        );
    }

    #[test]
    fn test_prepend_colons() {
        let input = "bar";
        // Colon at start
        assert_eq!(process(input, r"{prepend:\:foo}").unwrap(), ":foobar");
        // Colon at end
        assert_eq!(process(input, r"{prepend:foo\:}").unwrap(), "foo:bar");
        // Colon in middle
        assert_eq!(process(input, r"{prepend:fo\:o}").unwrap(), "fo:obar");
        // Multiple colons
        assert_eq!(
            process(input, r"{prepend:foo\:bar\:baz\:}").unwrap(),
            "foo:bar:baz:bar"
        );
    }

    #[test]
    fn test_append_prepend_colons_list() {
        let input = "a,b,c";
        // Append with colons to list
        assert_eq!(
            process(input, r"{split:,:..|append:\:x\:y\:z}").unwrap(),
            "a:x:y:z,b:x:y:z,c:x:y:z"
        );
        // Prepend with colons to list
        assert_eq!(
            process(input, r"{split:,:..|prepend:x\:y\:z\:}").unwrap(),
            "x:y:z:a,x:y:z:b,x:y:z:c"
        );
    }

    #[test]
    fn test_replace_colons() {
        let input = "foo:bar:baz";
        // Replace colon with dash
        assert_eq!(
            process(input, r"{replace:s/\:/-/g}").unwrap(),
            "foo-bar-baz"
        );
        // Replace 'bar:baz' with 'qux:quux'
        assert_eq!(
            process(input, r"{replace:s/bar\:baz/qux\:quux/}").unwrap(),
            "foo:qux:quux"
        );
        // Replace with colons in both pattern and replacement
        let input = "a:b:c";
        assert_eq!(process(input, r"{replace:s/a\:b/c\:d/}").unwrap(), "c:d:c");
    }

    #[test]
    fn test_chain_colon_args() {
        let input = "foo";
        // Prepend and append with colons, then upper
        assert_eq!(
            process(input, r"{prepend:\:start\:|append:\:end\:|upper}").unwrap(),
            ":START:FOO:END:"
        );
        // On a list
        let input = "a,b";
        assert_eq!(
            process(input, r"{split:,:..|prepend:x\:|append:\:y}").unwrap(),
            "x:a:y,x:b:y"
        );
    }

    #[test]
    fn test_escaped_colon_append() {
        let input = "foo";
        // Append a literal colon
        assert_eq!(process(input, r"{append:\:}").unwrap(), "foo:");
        // Append a literal backslash
        assert_eq!(process(input, r"{append:\\}").unwrap(), r"foo\");
        // Append colon and backslash
        assert_eq!(process(input, r"{append:\:\\}").unwrap(), r"foo:\");
        // Append multiple colons and backslashes
        assert_eq!(process(input, r"{append:\:\:\\\:\\}").unwrap(), r"foo::\:\");
    }

    #[test]
    fn test_escaped_colon_prepend() {
        let input = "bar";
        // Prepend a literal colon
        assert_eq!(process(input, r"{prepend:\:}").unwrap(), ":bar");
        // Prepend a literal backslash
        assert_eq!(process(input, r"{prepend:\\}").unwrap(), r"\bar");
        // Prepend colon and backslash
        assert_eq!(process(input, r"{prepend:\:\\}").unwrap(), r":\bar");
        // Prepend multiple colons and backslashes
        assert_eq!(
            process(input, r"{prepend:\:\:\\\:\\}").unwrap(),
            r"::\:\bar"
        );
    }

    #[test]
    fn test_escaped_colon_in_list() {
        let input = "a,b";
        // Append and prepend with escaped colons and backslashes
        assert_eq!(
            process(input, r"{split:,:..|prepend:\:|append:\\\\}").unwrap(),
            r":a\\,:b\\"
        );
    }

    #[test]
    fn test_escaped_pipe() {
        let input = "foo|bar";
        // Replace pipe with dash
        assert_eq!(process(input, r"{replace:s/\|/-/}").unwrap(), "foo-bar");
        // Replace with escaped pipe in replacement
        assert_eq!(
            process(input, r"{replace:s/\|/\\\|/}").unwrap(),
            r"foo\|bar"
        );
        // Replace text containing pipe with another text containing pipe
        assert_eq!(
            process(input, r"{replace:s/foo\|bar/baz\|qux/}").unwrap(),
            "baz|qux"
        );
    }

    #[test]
    fn test_escaped_pipe_in_args() {
        let input = "a|b|c";
        // Split by pipe and join with dash
        assert_eq!(process(input, r"{split:\|:..|join:-}").unwrap(), "a-b-c");
        // Split by pipe and join with pipe
        assert_eq!(process(input, r"{split:\|:..|join:\|}").unwrap(), "a|b|c");
        // Split by pipe and append/prepend with pipes
        assert_eq!(
            process(input, r"{split:\|:..|append:\|y|join:,}").unwrap(),
            "a|y,b|y,c|y"
        );
    }

    #[test]
    fn test_empty_operations() {
        // Empty template should return the input as-is
        assert_eq!(process("test", "{}").unwrap(), "test");
    }

    #[test]
    fn test_invalid_range_edge_cases() {
        // Test what happens with very large indices
        assert_eq!(process("a,b,c", "{split:,:100}").unwrap(), "c");
        // Test empty range (start > end)
        assert_eq!(process("a,b,c", "{split:,:3..1}").unwrap(), "");
        // Test range that starts beyond bounds
        assert_eq!(process("a,b,c", "{split:,:10..20}").unwrap(), "");
    }

    #[test]
    fn test_join_without_prior_split() {
        // What happens when you join a string?
        assert_eq!(process("hello", "{join:-}").unwrap(), "hello");
    }

    #[test]
    fn test_strip_empty_chars() {
        // Edge case: strip with empty character set
        assert_eq!(process("hello", "{strip:}").unwrap(), "hello");
    }

    #[test]
    fn test_slice_empty_string() {
        assert_eq!(process("", "{slice:0}").unwrap(), "");
        assert_eq!(process("", "{slice:-1}").unwrap(), "");
        assert_eq!(process("", "{slice:1..3}").unwrap(), "");
        assert_eq!(process("", "{slice:..}").unwrap(), "");
    }

    #[test]
    fn test_slice_empty_list() {
        // Split an empty string creates empty list
        assert_eq!(process("", "{split:,:..|slice:0}").unwrap(), "");
        assert_eq!(process("", "{split:,:..|slice:1..3}").unwrap(), "");
    }

    #[test]
    fn test_malformed_sed_strings() {
        // Missing closing slash
        assert!(process("test", "{replace:s/pattern/replacement}").is_err());
        // No pattern
        assert!(process("test", "{replace:s//replacement/}").is_err());
        // Wrong format entirely
        assert!(process("test", "{replace:pattern/replacement}").is_err());
    }

    #[test]
    fn test_invalid_template_format() {
        // Missing braces
        assert!(process("test", "split:,:0").is_err());
        // Missing opening brace
        assert!(process("test", "split:,:0}").is_err());
        // Missing closing brace
        assert!(process("test", "{split:,:0").is_err());
    }

    #[test]
    fn test_unknown_operation() {
        assert!(process("test", "{unknown}").is_err());
        assert!(process("test", "{badop:arg}").is_err());
    }

    #[test]
    fn test_invalid_range_strings() {
        // Invalid range formats
        assert!(process("a,b,c", "{split:,:abc}").is_err());
        assert!(process("a,b,c", "{split:,:1..abc}").is_err());
        assert!(process("hello", "{slice:xyz}").is_err());
    }

    #[test]
    fn test_large_indices_handling() {
        let input = "a,b,c";
        // Very large positive index should clamp to last element
        assert_eq!(process(input, "{split:,:999999}").unwrap(), "c");
        // Very large negative index should clamp to first element
        assert_eq!(process(input, "{split:,:-999999}").unwrap(), "a");
    }

    #[test]
    fn test_operations_on_empty_list() {
        // Create empty list and apply operations
        let input = "";
        assert_eq!(process(input, "{split:,:..|upper}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..|lower}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..|trim}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..|append:!}").unwrap(), "!");
        assert_eq!(process(input, "{split:,:..|prepend:_}").unwrap(), "_");
    }

    #[test]
    fn test_final_output_behavior() {
        // Test documented behavior: List joins with last split separator or space
        let input = "a,b,c";

        // With split operation - should use comma
        assert_eq!(process(input, "{split:,:..|upper}").unwrap(), "A,B,C");

        // Without split operation - should use space (no split occurred)
        assert_eq!(process("hello world", "{upper}").unwrap(), "HELLO WORLD");

        // Multiple splits - should use last split separator
        assert_eq!(
            process(input, "{split:,:..|join:-|split:-:..|upper}").unwrap(),
            "A-B-C"
        );
    }

    #[test]
    fn test_shorthand_index() {
        let input = "a b c d e";
        // Test shorthand index
        assert_eq!(process(input, "{1}").unwrap(), "b");
        assert_eq!(process(input, "{-1}").unwrap(), "e");
        assert_eq!(process(input, "{0}").unwrap(), "a");

        // Test shorthand ranges
        assert_eq!(process(input, "{1..3}").unwrap(), "b c");
        assert_eq!(process(input, "{1..=3}").unwrap(), "b c d");
        assert_eq!(process(input, "{..2}").unwrap(), "a b");
        assert_eq!(process(input, "{2..}").unwrap(), "c d e");
        assert_eq!(process(input, "{..=2}").unwrap(), "a b c");
        assert_eq!(process(input, "{..}").unwrap(), "a b c d e");
        assert_eq!(process(input, "{-2..}").unwrap(), "d e");
        assert_eq!(process(input, "{-3..-1}").unwrap(), "c d");
        assert_eq!(process(input, "{-3..=-1}").unwrap(), "c d e");

        // Test with empty input
        assert_eq!(process("", "{1}").unwrap(), "");
        assert_eq!(process("", "{1..3}").unwrap(), "");
        assert_eq!(process("", "{..}").unwrap(), "");

        // Test with single word
        assert_eq!(process("word", "{0}").unwrap(), "word");
        assert_eq!(process("word", "{1}").unwrap(), "word");
        assert_eq!(process("word", "{..}").unwrap(), "word");
        assert_eq!(process("word", "{0..}").unwrap(), "word");
        assert_eq!(process("word", "{..1}").unwrap(), "word");
    }

    #[test]
    fn test_empty_list_append_consistency() {
        // Create empty list through split of empty string
        let result = process("", "{split:,:..|append:!}").unwrap();
        assert_eq!(result, "!");

        // Create empty list through split with no matches
        let result = process("abc", "{split:xyz:..|append:!}").unwrap();
        assert_eq!(result, "abc!");
    }

    #[test]
    fn test_empty_list_prepend_consistency() {
        // Create empty list through split of empty string
        let result = process("", "{split:,:..|prepend:!}").unwrap();
        assert_eq!(result, "!");

        // Create empty list through split with no matches
        let result = process("abc", "{split:xyz:..|prepend:!}").unwrap();
        assert_eq!(result, "!abc");
    }

    #[test]
    fn test_empty_list_vs_other_operations_consistency() {
        // Test how other operations handle empty lists for comparison

        // Upper on empty list
        let upper_result = process("", "{split:,:..|upper}").unwrap();
        assert_eq!(upper_result, ""); // Consistent: empty string

        // Lower on empty list
        let lower_result = process("", "{split:,:..|lower}").unwrap();
        assert_eq!(lower_result, ""); // Consistent: empty string

        // Trim on empty list
        let trim_result = process("", "{split:,:..|trim}").unwrap();
        assert_eq!(trim_result, ""); // Consistent: empty string

        // Strip on empty list
        let strip_result = process("", "{split:,:..|strip:x}").unwrap();
        assert_eq!(strip_result, ""); // Consistent: empty string

        // Replace on empty list
        let replace_result = process("", "{split:,:..|replace:s/a/b/}").unwrap();
        assert_eq!(replace_result, ""); // Consistent: empty string

        // Slice on empty list
        let slice_result = process("", "{split:,:..|slice:0..1}").unwrap();
        assert_eq!(slice_result, ""); // Consistent: empty string
    }

    #[test]
    fn test_empty_list_chain_with_append_prepend() {
        // Test chaining operations after append/prepend on empty list

        // Chain after append on empty list
        let chain_after_append = process("", "{split:,:..|append:!|upper}").unwrap();
        assert_eq!(chain_after_append, "!");

        // Chain after prepend on empty list
        let chain_after_prepend = process("", "{split:,:..|prepend:_|lower}").unwrap();
        assert_eq!(chain_after_prepend, "_");

        // But what if we try to split again after append/prepend?
        let split_after_append = process("", "{split:,:..|append:a,b|split:,:..|join:-}").unwrap();
        assert_eq!(split_after_append, "a-b");
    }

    #[test]
    fn test_empty_list_multiple_appends_prepends() {
        // Test multiple append/prepend operations on empty list

        let multiple_appends = process("", "{split:,:..|append:!|append:?}").unwrap();
        assert_eq!(multiple_appends, "!?");

        let multiple_prepends = process("", "{split:,:..|prepend:_|prepend:#}").unwrap();
        assert_eq!(multiple_prepends, "#_");

        let mixed = process("", "{split:,:..|append:!|prepend:_}").unwrap();
        assert_eq!(mixed, "_!");
    }

    #[test]
    fn test_empty_list_join_behavior() {
        // Test join operation on empty list
        let join_empty = process("", "{split:,:..|join:-}").unwrap();
        assert_eq!(join_empty, ""); // Should return empty string

        // Test join after operations that might create empty list
        let join_after_range = process("a,b,c", "{split:,:10..20|join:-}").unwrap();
        assert_eq!(join_after_range, ""); // Should return empty string
    }

    #[test]
    fn test_expected_consistent_behavior_for_empty_lists() {
        // EXPECTED: append/prepend on empty list should maintain list type consistency
        // but since it's empty, the final join should use the operation result appropriately
        let result1 = process("", "{split:,:..|append:a|append:b}").unwrap();
        let result2 = process("a", "{append:b}").unwrap();
        assert_eq!(result1, "ab"); // Both should be same
        assert_eq!(result2, "ab"); // if behavior is consistent
    }

    #[test]
    fn test_edge_case_empty_string_vs_empty_list() {
        // Test difference between empty string and empty list

        // Empty string input
        let empty_string_append = process("", "{append:!}").unwrap();
        assert_eq!(empty_string_append, "!"); // String operation

        // Empty list (from split) append
        let empty_list_append = process("", "{split:,:..|append:!}").unwrap();
        assert_eq!(empty_list_append, "!"); // List operation -> String

        // These should potentially behave the same way for consistency
        assert_eq!(empty_string_append, empty_list_append);
    }

    #[test]
    fn test_empty_list_with_different_separators() {
        // Test if the separator tracking works correctly with empty lists

        let result1 = process("", "{split:,:..|append:a|append:b}").unwrap();
        assert_eq!(result1, "ab");

        let result2 = process("", "{split:-:..|append:a|append:b}").unwrap();
        assert_eq!(result2, "ab");

        // Both should be the same since we're not creating actual lists
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_empty_list_operation_order_dependency() {
        // Test if the order of operations affects empty list handling

        // Append then join
        let append_then_join = process("", "{split:,:..|append:test|join:-}").unwrap();
        assert_eq!(append_then_join, "test");

        // Join then append (should not be possible, but test error handling)
        let join_then_append = process("", "{split:,:..|join:-|append:test}").unwrap();
        assert_eq!(join_then_append, "test");

        // These results expose the internal type conversions
    }

    #[test]
    fn test_append_prepend_consistent_behavior() {
        // All operations on empty lists should return empty results
        // except when the operation itself provides content
        // All operations should treat empty list as single empty string element:
        assert_eq!(process("", "{split:,:..|upper}").unwrap(), "");
        assert_eq!(process("", "{split:,:..|append:!}").unwrap(), "!");
        assert_eq!(process("", "{split:,:..|prepend:_}").unwrap(), "_");
    }

    // ============================================================================
    // POSITIVE TESTING - Expected Behavior
    // ============================================================================

    mod positive_tests {
        use super::*;

        // ------------------------------------------------------------------------
        // Basic Operations
        // ------------------------------------------------------------------------

        #[test]
        fn test_split_basic() {
            assert_eq!(process("a,b,c", "{split:,:..|join:-}").unwrap(), "a-b-c");
            assert_eq!(
                process("hello world", "{split: :..|join:_}").unwrap(),
                "hello_world"
            );
            assert_eq!(
                process("a::b::c", r"{split:\:\::..|join:\|}").unwrap(),
                "a|b|c"
            );
        }

        #[test]
        fn test_split_with_ranges() {
            let input = "a,b,c,d,e,f";
            // Index access
            assert_eq!(process(input, "{split:,:0}").unwrap(), "a");
            assert_eq!(process(input, "{split:,:2}").unwrap(), "c");
            assert_eq!(process(input, "{split:,:-1}").unwrap(), "f");
            assert_eq!(process(input, "{split:,:-2}").unwrap(), "e");

            // Exclusive ranges
            assert_eq!(process(input, "{split:,:1..3}").unwrap(), "b,c");
            assert_eq!(process(input, "{split:,:0..2}").unwrap(), "a,b");
            assert_eq!(process(input, "{split:,:-3..-1}").unwrap(), "d,e");

            // Inclusive ranges
            assert_eq!(process(input, "{split:,:1..=3}").unwrap(), "b,c,d");
            assert_eq!(process(input, "{split:,:-3..=-1}").unwrap(), "d,e,f");

            // Open ranges
            assert_eq!(process(input, "{split:,:2..}").unwrap(), "c,d,e,f");
            assert_eq!(process(input, "{split:,:..3}").unwrap(), "a,b,c");
            assert_eq!(process(input, "{split:,:..=2}").unwrap(), "a,b,c");
            assert_eq!(process(input, "{split:,:..}").unwrap(), "a,b,c,d,e,f");
        }

        #[test]
        fn test_join_various_separators() {
            let input = "a,b,c";
            assert_eq!(process(input, r"{split:,:..|join:\\n}").unwrap(), "a\nb\nc");
            assert_eq!(process(input, r"{split:,:..|join:\\t}").unwrap(), "a\tb\tc");
            assert_eq!(process(input, r"{split:,:..|join:\\r}").unwrap(), "a\rb\rc");
            assert_eq!(
                process(input, "{split:,:..|join: - }").unwrap(),
                "a - b - c"
            );
            assert_eq!(process(input, "{split:,:..|join:}").unwrap(), "abc");
        }

        #[test]
        fn test_replace_patterns() {
            // Sed-style replace
            assert_eq!(
                process("foo bar foo", "{replace:s/foo/baz/}").unwrap(),
                "baz bar foo"
            );
            assert_eq!(
                process("foo bar foo", "{replace:s/foo/baz/g}").unwrap(),
                "baz bar baz"
            );

            // Regex patterns in sed style
            assert_eq!(
                process("abc123def", r"{replace:s/[0-9]+/XXX/}").unwrap(),
                "abcXXXdef"
            );
            assert_eq!(
                process("test123test456", "{replace:s/[0-9]+/NUM/g}").unwrap(),
                "testNUMtestNUM"
            );

            // Word boundaries
            assert_eq!(
                process("cat catch", r"{replace:s/\bcat\b/dog/g}").unwrap(),
                "dog catch"
            );
        }

        #[test]
        fn test_case_operations() {
            assert_eq!(process("Hello World", "{upper}").unwrap(), "HELLO WORLD");
            assert_eq!(process("Hello World", "{lower}").unwrap(), "hello world");

            // On lists
            assert_eq!(process("a,B,c", "{split:,:..|upper}").unwrap(), "A,B,C");
            assert_eq!(process("A,b,C", "{split:,:..|lower}").unwrap(), "a,b,c");
        }

        #[test]
        fn test_trim_operations() {
            assert_eq!(process("  hello  ", "{trim}").unwrap(), "hello");
            assert_eq!(process("\t\nhello\r\n", "{trim}").unwrap(), "hello");

            // On lists
            assert_eq!(
                process(" a , b , c ", "{split:,:..|trim}").unwrap(),
                "a,b,c"
            );
        }

        #[test]
        fn test_strip_operations() {
            assert_eq!(process("xyhelloxy", "{strip:xy}").unwrap(), "hello");
            assert_eq!(process("!!!hello!!!", "{strip:!}").unwrap(), "hello");
            assert_eq!(process("abchelloabc", "{strip:abc}").unwrap(), "hello");
            assert_eq!(process("  hello  ", "{strip: }").unwrap(), "hello");
            assert_eq!(process("hello", "{strip:}").unwrap(), "hello"); // default whitespace
        }

        #[test]
        fn test_slice_operations() {
            let input = "abcdefgh";
            assert_eq!(process(input, "{slice:0}").unwrap(), "a");
            assert_eq!(process(input, "{slice:2}").unwrap(), "c");
            assert_eq!(process(input, "{slice:-1}").unwrap(), "h");
            assert_eq!(process(input, "{slice:1..4}").unwrap(), "bcd");
            assert_eq!(process(input, "{slice:1..=4}").unwrap(), "bcde");
            assert_eq!(process(input, "{slice:2..}").unwrap(), "cdefgh");
            assert_eq!(process(input, "{slice:..3}").unwrap(), "abc");
            assert_eq!(process(input, "{slice:..=3}").unwrap(), "abcd");
            assert_eq!(process(input, "{slice:-3..}").unwrap(), "fgh");
            assert_eq!(process(input, "{slice:-3..-1}").unwrap(), "fg");
            assert_eq!(process(input, "{slice:-3..=-1}").unwrap(), "fgh");
        }

        #[test]
        fn test_append_prepend_operations() {
            assert_eq!(process("hello", "{append: world}").unwrap(), "hello world");
            assert_eq!(process("world", "{prepend:hello }").unwrap(), "hello world");

            // On lists
            assert_eq!(
                process("a,b,c", "{split:,:..|append:!}").unwrap(),
                "a!,b!,c!"
            );
            assert_eq!(
                process("a,b,c", "{split:,:..|prepend:_}").unwrap(),
                "_a,_b,_c"
            );
        }

        #[test]
        fn test_shorthand_syntax() {
            let input = "one two three four five";
            assert_eq!(process(input, "{0}").unwrap(), "one");
            assert_eq!(process(input, "{2}").unwrap(), "three");
            assert_eq!(process(input, "{-1}").unwrap(), "five");
            assert_eq!(process(input, "{1..3}").unwrap(), "two three");
            assert_eq!(process(input, "{1..=3}").unwrap(), "two three four");
            assert_eq!(process(input, "{..2}").unwrap(), "one two");
            assert_eq!(process(input, "{2..}").unwrap(), "three four five");
            assert_eq!(process(input, "{..=2}").unwrap(), "one two three");
            assert_eq!(process(input, "{..}").unwrap(), "one two three four five");
        }

        // ------------------------------------------------------------------------
        // Complex Chaining
        // ------------------------------------------------------------------------

        #[test]
        fn test_complex_chains() {
            // Multi-step transformations
            assert_eq!(
                process("foo,bar,baz", "{split:,:..|upper|join:-|append:!}").unwrap(),
                "FOO-BAR-BAZ!"
            );

            // Range then transform
            assert_eq!(
                process("a,b,c,d,e", "{split:,:1..4|join:_|replace:s/_/-/g|upper}").unwrap(),
                "B-C-D"
            );

            // Complex slicing and replacement
            assert_eq!(
                process(
                    "hello world test",
                    "{split: :..|slice:0..=1|join:|replace:s/o/0/g}"
                )
                .unwrap(),
                "hew0te"
            );
        }

        #[test]
        fn test_nested_operations() {
            // Split, transform each element, rejoin with different separator
            assert_eq!(
                process(
                    "First,Second,Third",
                    r"{split:,:..|lower|prepend:item_|join: \| }"
                )
                .unwrap(),
                "item_first | item_second | item_third"
            );

            // Extract, transform, and format
            assert_eq!(
                process(
                    "data1,data2,data3,data4",
                    "{split:,:1..3|replace:s/data/item/g|upper|join: -> }"
                )
                .unwrap(),
                "ITEM2 -> ITEM3"
            );
        }

        // ------------------------------------------------------------------------
        // Special Characters and Escaping
        // ------------------------------------------------------------------------

        #[test]
        fn test_escape_sequences() {
            // Newlines, tabs, carriage returns
            assert_eq!(process("a,b", r"{split:,:..|join:\\n}").unwrap(), "a\nb");
            assert_eq!(process("a,b", r"{split:,:..|join:\\t}").unwrap(), "a\tb");
            assert_eq!(process("a,b", r"{split:,:..|join:\\r}").unwrap(), "a\rb");

            // Escaped special characters
            assert_eq!(process("test", r"{append:\:}").unwrap(), "test:");
            assert_eq!(process("test", r"{append:\\}").unwrap(), r"test\");
            assert_eq!(process("test", r"{append:\|}").unwrap(), "test|");

            // Complex escaping
            assert_eq!(process("test", r"{append:\:\\\|\:}").unwrap(), r"test:\|:");
        }

        #[test]
        fn test_unicode_support() {
            // Unicode characters
            assert_eq!(
                process("café,naïve", "{split:,:..|upper}").unwrap(),
                "CAFÉ,NAÏVE"
            );
            assert_eq!(process("BJÖRK", "{lower}").unwrap(), "björk");

            // Emoji and symbols
            assert_eq!(
                process("🚀,⭐,🌟", "{split:,:..|join: }").unwrap(),
                "🚀 ⭐ 🌟"
            );

            // Unicode in separators
            assert_eq!(process("a→b→c", "{split:→:..|join:←}").unwrap(), "a←b←c");
        }

        #[test]
        fn test_special_separators() {
            // Multi-character separators
            assert_eq!(
                process("a::b::c", r"{split:\:\::..|join:--}").unwrap(),
                "a--b--c"
            );
            assert_eq!(
                process("a<>b<>c", "{split:<>:..|join:><}").unwrap(),
                "a><b><c"
            );

            // Regex-special characters as separators
            assert_eq!(process("a.b.c", "{split:.:..|join:*}").unwrap(), "a*b*c");
            assert_eq!(process("a+b+c", "{split:+:..|join:=}").unwrap(), "a=b=c");
        }

        // ------------------------------------------------------------------------
        // Edge Cases and Boundary Conditions
        // ------------------------------------------------------------------------

        #[test]
        fn test_empty_inputs() {
            assert_eq!(process("", "{}").unwrap(), "");
            assert_eq!(process("", "{upper}").unwrap(), "");
            assert_eq!(process("", "{split:,:..|join:-}").unwrap(), "");
            assert_eq!(process("", "{slice:0}").unwrap(), "");
            assert_eq!(process("", "{append:test}").unwrap(), "test");
            assert_eq!(process("", "{prepend:test}").unwrap(), "test");
        }

        #[test]
        fn test_single_character_inputs() {
            assert_eq!(process("a", "{upper}").unwrap(), "A");
            assert_eq!(process("A", "{lower}").unwrap(), "a");
            assert_eq!(process("a", "{slice:0}").unwrap(), "a");
            assert_eq!(process("a", "{append:b}").unwrap(), "ab");
            assert_eq!(process("b", "{prepend:a}").unwrap(), "ab");
        }

        #[test]
        fn test_boundary_indices() {
            let input = "a,b,c";
            // Large positive indices (should clamp)
            assert_eq!(process(input, "{split:,:1000}").unwrap(), "c");
            assert_eq!(process(input, "{split:,:999..1001}").unwrap(), "");

            // Large negative indices (should clamp)
            assert_eq!(process(input, "{split:,:-1000}").unwrap(), "a");
            assert_eq!(process(input, "{split:,:-1000..-999}").unwrap(), "");
        }

        #[test]
        fn test_no_separator_found() {
            // Split with separator not in string
            assert_eq!(process("hello", "{split:,:..|join:-}").unwrap(), "hello");
            assert_eq!(process("hello", "{split:xyz:..|upper}").unwrap(), "HELLO");
        }

        #[test]
        fn test_consecutive_separators() {
            // Multiple consecutive separators create empty elements
            assert_eq!(process("a,,b", "{split:,:..|join:-}").unwrap(), "a--b");
            assert_eq!(process("a,,,b", r"{split:,:..|join:\|}").unwrap(), "a|||b");
        }

        #[test]
        fn test_operations_preserve_structure() {
            // Operations on lists should preserve list nature until final output
            let input = "a,b,c";
            assert_eq!(process(input, "{split:,:..|upper|lower}").unwrap(), "a,b,c");
            assert_eq!(process(input, "{split:,:..|trim|strip:}").unwrap(), "a,b,c");
        }

        // ------------------------------------------------------------------------
        // Performance and Stress Testing
        // ------------------------------------------------------------------------

        #[test]
        fn test_large_inputs() {
            // Large number of elements
            let large_input = (0..1000)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let result = process(&large_input, "{split:,:0..10|join:-}").unwrap();
            assert_eq!(result, "0-1-2-3-4-5-6-7-8-9");
        }

        #[test]
        fn test_long_strings() {
            let long_string = "a".repeat(10000);
            assert_eq!(process(&long_string, "{slice:0..5}").unwrap(), "aaaaa");
            assert_eq!(process(&long_string, "{slice:-5..}").unwrap(), "aaaaa");
        }

        #[test]
        fn test_deep_chaining() {
            let input = "test";
            let result = process(
                input,
                "{upper|lower|upper|lower|upper|lower|append:!|prepend:_}",
            )
            .unwrap();
            assert_eq!(result, "_test!");
        }
    }

    // ============================================================================
    // NEGATIVE TESTING - Error Handling and Edge Cases
    // ============================================================================

    mod negative_tests {
        use super::*;

        // ------------------------------------------------------------------------
        // Malformed Templates
        // ------------------------------------------------------------------------

        #[test]
        fn test_malformed_template_syntax() {
            // Missing braces
            assert!(process("test", "split:,:0").is_err());
            assert!(process("test", "upper").is_err());

            // Unmatched braces
            assert!(process("test", "{split:,:0").is_err());
            assert!(process("test", "split:,:0}").is_err());
            assert!(process("test", "{{split:,:0}").is_err());

            // Empty braces with invalid content
            assert!(process("test", "{ }").is_err());
            assert!(process("test", r"{\|}").is_err());
        }

        #[test]
        fn test_invalid_operations() {
            // Non-existent operations
            assert!(process("test", "{unknown}").is_err());
            assert!(process("test", "{badop:arg}").is_err());
            assert!(process("test", "{invalid_operation}").is_err());

            // Misspelled operations
            assert!(process("test", "{splt:,:0}").is_err());
            assert!(process("test", "{joinn:-}").is_err());
            assert!(process("test", "{repplace:s/a/b/}").is_err());
        }

        #[test]
        fn test_invalid_operation_arguments() {
            // Missing required arguments
            assert!(process("test", "{split}").is_err());
            assert!(process("test", "{join}").is_err());
            assert!(process("test", "{replace}").is_err());
            assert!(process("test", "{slice}").is_err());
            assert!(process("test", "{strip}").is_err());
            assert!(process("test", "{append}").is_err());
            assert!(process("test", "{prepend}").is_err());

            // Invalid argument formats
            assert!(process("test", "{split:}").is_err());
        }

        // ------------------------------------------------------------------------
        // Invalid Range Specifications
        // ------------------------------------------------------------------------

        #[test]
        fn test_invalid_range_formats() {
            // Non-numeric ranges
            assert!(process("a,b,c", "{split:,:abc}").is_err());
            assert!(process("a,b,c", "{split:,:1..abc}").is_err());
            assert!(process("hello", "{slice:xyz}").is_err());
            assert!(process("hello", "{slice:1..xyz}").is_err());

            // Invalid range syntax
            assert!(process("a,b,c", "{split:,:1...3}").is_err()); // Triple dots
            assert!(process("a,b,c", "{split:,:1.=3}").is_err()); // Wrong inclusive syntax
            assert!(process("hello", "{slice:1.2}").is_err()); // Single dot

            // Invalid shorthand ranges
            assert!(process("a b c", "{abc}").is_err());
            assert!(process("a b c", "{1.2.3}").is_err());
        }

        // ------------------------------------------------------------------------
        // Invalid Regular Expressions
        // ------------------------------------------------------------------------

        #[test]
        fn test_malformed_sed_strings() {
            // Missing closing slash
            assert!(process("test", "{replace:s/pattern/replacement}").is_err());
            assert!(process("test", "{replace:s/pattern}").is_err());

            // Missing middle slash
            assert!(process("test", "{replace:s/patternreplacement/}").is_err());

            // No pattern
            assert!(process("test", "{replace:s//replacement/}").is_err());

            // Wrong prefix
            assert!(process("test", "{replace:r/pattern/replacement/}").is_err());
            assert!(process("test", "{replace:pattern/replacement}").is_err());

            // Invalid flags
            assert!(process("test", "{replace:s/a/b/xyz}").is_ok()); // Should work but ignore invalid flags
        }

        #[test]
        fn test_invalid_regex_patterns() {
            // Invalid regex syntax in sed strings
            assert!(process("test", "{replace:s/[/replacement/}").is_err()); // Unclosed bracket
            assert!(process("test", "{replace:s/(/replacement/}").is_err()); // Unclosed paren
            assert!(process("test", "{replace:s/*/replacement/}").is_err()); // Invalid quantifier
            assert!(process("test", "{replace:s/+/replacement/}").is_err()); // Invalid quantifier
            assert!(process("test", "{replace:s/?/replacement/}").is_err()); // Invalid quantifier
            assert!(process("test", "{replace:s/{/replacement/}").is_err()); // Unclosed brace
        }

        // ------------------------------------------------------------------------
        // Chain Validation
        // ------------------------------------------------------------------------

        #[test]
        fn test_invalid_operation_chains() {
            // Operations that don't make sense in sequence (these should still work, testing for crashes)
            assert!(process("test", r"{split:,:..|split:-:..|split: :..|join:\|}").is_ok());
        }

        // ------------------------------------------------------------------------
        // Resource Exhaustion
        // ------------------------------------------------------------------------

        #[test]
        fn test_extremely_large_operations() {
            // Very large separator (should work but be expensive)
            let large_sep = "x".repeat(1000);
            let template = format!("{{split:{}:..|join:-}}", large_sep);
            assert!(process("test", &template).is_ok());

            // Very large replacement string
            let large_replacement = "y".repeat(1000);
            let template = format!("{{replace:s/test/{}/}}", large_replacement);
            assert!(process("test", &template).is_ok());
        }

        #[test]
        fn test_pathological_splitting() {
            // String with many consecutive separators
            let input = ",".repeat(1000);
            assert!(process(&input, "{split:,:..|join:-}").is_ok());

            // Every character is a separator
            let input = "a,a,a,a,a".repeat(200);
            assert!(process(&input, r"{split:,:..|join:\|\|}").is_ok());
        }

        // ------------------------------------------------------------------------
        // Unicode Edge Cases
        // ------------------------------------------------------------------------

        #[test]
        fn test_unicode_edge_cases() {
            // Splitting on unicode characters
            let input = "café→naïve→résumé";
            assert!(process(input, "{split:→:..|join:←}").is_ok());

            // Slicing through multi-byte characters (should work correctly)
            assert!(process("🚀🌟⭐", "{slice:1}").is_ok());

            // Very long unicode strings
            let long_unicode = "🌟".repeat(1000);
            assert!(process(&long_unicode, "{slice:0..10}").is_ok());
        }

        // ------------------------------------------------------------------------
        // Error Message Quality
        // ------------------------------------------------------------------------

        #[test]
        fn test_error_message_content() {
            // Check that error messages are informative

            let result = process("test", "{unknown}");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.contains("unknown") || error.contains("Unknown"));

            let result = process("test", "{replace:s/pattern/replacement}");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error.contains("slash") || error.contains("sed") || error.contains("Malformed")
            );

            let result = process("test", "{split:,:abc}");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error.contains("range") || error.contains("parse") || error.contains("invalid")
            );
        }

        // ------------------------------------------------------------------------
        // Boundary Input Validation
        // ------------------------------------------------------------------------

        #[test]
        fn test_null_and_control_characters() {
            // Null bytes in input
            let input_with_null = "hello\0world";
            assert!(process(input_with_null, "{upper}").is_ok());

            // Control characters
            let input_with_control = "hello\x01\x02world";
            assert!(process(input_with_control, "{split:\x01:..|join:\\|}").is_ok());
        }

        #[test]
        fn test_very_long_templates() {
            // Extremely long template (testing parser limits)
            let long_op = "upper|lower|".repeat(500);
            let template = format!("{{{}}}", long_op.trim_end_matches('|'));

            // This might be slow but shouldn't crash
            assert!(process("test", &template).is_ok());
        }

        #[test]
        fn test_nested_escape_sequences() {
            // Complex nested escaping that might confuse the parser
            assert!(process("test", r"{append:\\\\}").is_ok()); // Should append two backslashes
            assert!(process("test", r"{append:\\\:}").is_ok()); // Should append backslash and colon
        }
    }
}
