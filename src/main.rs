use clap::Parser;
use regex::Regex;

#[derive(Parser)]
struct Cli {
    /// The input string
    input: String,
    /// The template string
    template: String,
}

#[derive(Debug, Clone)]
enum Value {
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
enum StringOp {
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
enum RangeSpec {
    Index(isize),
    Range(Option<isize>, Option<isize>, bool), // (start, end, inclusive)
}

fn parse_range(s: &str) -> Result<RangeSpec, String> {
    let s = s.trim();
    if s.is_empty() {
        // Default to full range
        return Ok(RangeSpec::Range(None, None, false));
    }
    if let Some((start, end)) = s.split_once("..=") {
        let start = if start.is_empty() {
            None
        } else {
            Some(start.parse::<isize>().map_err(|_| "Invalid start")?)
        };
        let end = if end.is_empty() {
            None
        } else {
            Some(end.parse::<isize>().map_err(|_| "Invalid end")?)
        };
        return Ok(RangeSpec::Range(start, end, true));
    }
    if let Some((start, end)) = s.split_once("..") {
        let start = if start.is_empty() {
            None
        } else {
            Some(start.parse::<isize>().map_err(|_| "Invalid start")?)
        };
        let end = if end.is_empty() {
            None
        } else {
            Some(end.parse::<isize>().map_err(|_| "Invalid end")?)
        };
        return Ok(RangeSpec::Range(start, end, false));
    }
    let idx = s.parse::<isize>().map_err(|_| "Invalid index")?;
    Ok(RangeSpec::Index(idx))
}

/// Reads until the next unescaped ':' or end, supporting \: and \\ escaping.
fn read_until(body: &str, pos: &mut usize) -> String {
    let mut s = String::new();
    let bytes = body.as_bytes();
    while *pos < body.len() {
        let c = bytes[*pos] as char;
        if c == ':' {
            // Check if escaped
            if *pos > 0 && bytes[*pos - 1] == b'\\' {
                // Remove the escape
                s.pop();
                s.push(':');
                *pos += 1;
                continue;
            } else {
                break;
            }
        } else if c == '\\' {
            // Look ahead for escape
            if *pos + 1 < body.len() {
                let next = bytes[*pos + 1] as char;
                if next == ':' || next == '\\' {
                    s.push(next);
                    *pos += 2;
                    continue;
                }
            }
            // Lone backslash, treat as literal
            s.push('\\');
            *pos += 1;
            continue;
        } else {
            s.push(c);
            *pos += 1;
        }
    }
    s
}

/// Consumes a colon if present (not escaped).
fn consume_colon(body: &str, pos: &mut usize) {
    if *pos < body.len() && &body[*pos..*pos + 1] == ":" {
        // Check if escaped
        if *pos > 0 && &body[*pos - 1..*pos] == "\\" {
            // Escaped, do not consume
            return;
        }
        *pos += 1;
    }
}

/// Reads until the next unescaped ':' that is followed by a known op, or end.
/// Supports \: and \\ escaping.
fn read_arg_until_next_op(body: &str, pos: &mut usize, ops: &[&str]) -> String {
    let mut s = String::new();
    let bytes = body.as_bytes();
    while *pos < body.len() {
        let c = bytes[*pos] as char;
        if c == ':' {
            // Check if escaped
            if *pos > 0 && bytes[*pos - 1] == b'\\' {
                // Remove the escape
                s.pop();
                s.push(':');
                *pos += 1;
                continue;
            }
            // Check if this colon is followed by a known op
            let after_colon = &body[*pos + 1..];
            for op in ops {
                if after_colon.starts_with(op) {
                    return s;
                }
            }
            // Not followed by op, treat as separator
            break;
        } else if c == '\\' {
            // Look ahead for escape
            if *pos + 1 < body.len() {
                let next = bytes[*pos + 1] as char;
                if next == ':' || next == '\\' {
                    s.push(next);
                    *pos += 2;
                    continue;
                }
            }
            // Lone backslash, treat as literal
            s.push('\\');
            *pos += 1;
            continue;
        } else {
            s.push(c);
            *pos += 1;
        }
    }
    s
}

fn parse_template(template: &str) -> Result<Vec<StringOp>, String> {
    if !template.starts_with('{') || !template.ends_with('}') {
        return Err("Template must start with '{' and end with '}'".to_string());
    }
    let body = &template[1..template.len() - 1];
    let mut pos = 0;
    let len = body.len();
    let mut ops = Vec::new();

    const OPS: &[&str] = &[
        "split", "join", "slice", "replace", "upper", "lower", "trim", "strip", "append", "prepend",
    ];

    let re = Regex::new(r"^s/((?:[^/]|\\/)+)/((?:[^/]|\\/)*?)/([a-zA-Z]*)$")
        .map_err(|e| e.to_string())?;

    while pos < len {
        let op = read_until(body, &mut pos);
        consume_colon(body, &mut pos);

        match op.as_str() {
            "split" => {
                let sep = read_until(body, &mut pos);
                consume_colon(body, &mut pos);
                let range_str = read_until(body, &mut pos);
                consume_colon(body, &mut pos);
                let range = parse_range(&range_str)?;
                ops.push(StringOp::Split { sep, range });
            }
            "join" => {
                let sep = read_until(body, &mut pos);
                consume_colon(body, &mut pos);
                ops.push(StringOp::Join { sep });
            }
            "slice" => {
                let range_str = read_until(body, &mut pos);
                consume_colon(body, &mut pos);
                let range = parse_range(&range_str)?;
                ops.push(StringOp::Slice { range });
            }
            "replace" => {
                let sed = read_arg_until_next_op(body, &mut pos, OPS);
                consume_colon(body, &mut pos);
                let caps = re
                    .captures(&sed)
                    .ok_or("replace sed string must be s/pattern/replacement/flags")?;
                let pattern = caps.get(1).unwrap().as_str().replace("\\/", "/");
                let replacement = caps.get(2).unwrap().as_str().replace("\\/", "/");
                let flags = caps.get(3).unwrap().as_str().to_string();
                ops.push(StringOp::Replace {
                    pattern,
                    replacement,
                    flags,
                });
            }
            "upper" => ops.push(StringOp::Upper),
            "lower" => ops.push(StringOp::Lower),
            "trim" => ops.push(StringOp::Trim),
            "strip" => {
                let chars_arg = read_until(body, &mut pos);
                consume_colon(body, &mut pos);
                ops.push(StringOp::Strip { chars: chars_arg });
            }
            "append" => {
                let suffix = read_arg_until_next_op(body, &mut pos, OPS);
                consume_colon(body, &mut pos);
                ops.push(StringOp::Append { suffix });
            }
            "prepend" => {
                let prefix = read_arg_until_next_op(body, &mut pos, OPS);
                consume_colon(body, &mut pos);
                ops.push(StringOp::Prepend { prefix });
            }
            "" => continue, // skip empty
            unknown => return Err(format!("Unknown operation: {}", unknown)),
        }
    }
    Ok(ops)
}

fn resolve_index(idx: isize, len: usize) -> usize {
    let len = len as isize;
    let mut i = if idx < 0 { len + idx } else { idx };
    if i < 0 {
        i = 0;
    }
    if i > len {
        i = len;
    }
    i as usize
}

fn apply_range<T: Clone>(items: &[T], range: &RangeSpec) -> Vec<T> {
    let len = items.len();
    match range {
        RangeSpec::Index(idx) => {
            // Handle empty collections
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
            let s_idx = resolve_index(start.unwrap_or(0), len);
            let mut e_idx = match end {
                Some(e) => resolve_index(*e, len),
                None => len,
            };
            if *inclusive && end.is_some() {
                e_idx += 1;
                if e_idx > len {
                    e_idx = len;
                }
            }
            if s_idx > e_idx {
                vec![]
            } else {
                items[s_idx..e_idx].to_vec()
            }
        }
    }
}

fn apply_ops(input: &str, ops: &[StringOp]) -> Result<String, String> {
    let mut val = Value::Str(input.to_string());
    let mut last_split_sep: Option<String> = None;
    for op in ops {
        match op {
            StringOp::Split { sep, range } => {
                let parts: Vec<String> = match &val {
                    Value::Str(s) => s.split(sep).map(|s| s.to_string()).collect(),
                    Value::List(list) => list
                        .iter()
                        .flat_map(|s| s.split(sep).map(|s| s.to_string()))
                        .collect(),
                };
                last_split_sep = Some(sep.clone());
                let result = apply_range(&parts, range);
                val = Value::List(result);
            }
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
                    val = Value::Str(list.join(sep));
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
                let re = Regex::new(pattern).map_err(|e| e.to_string())?;
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
                let chars: Vec<char> = chars.chars().collect();
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
                    val = Value::List(list.iter().map(|s| format!("{}{}", s, suffix)).collect())
                }
            },
            StringOp::Prepend { prefix } => match &val {
                Value::Str(s) => val = Value::Str(format!("{}{}", prefix, s)),
                Value::List(list) => {
                    val = Value::List(list.iter().map(|s| format!("{}{}", prefix, s)).collect())
                }
            },
        }
    }

    // Note: If the final value is a List, we join using the last split separator
    // or a space if no split operation was performed
    Ok(match val {
        Value::Str(s) => s,
        Value::List(list) => list.join(last_split_sep.as_deref().unwrap_or(" ")),
    })
}

fn process(input: &str, template: &str) -> Result<String, String> {
    let ops = parse_template(template)?;
    apply_ops(input, &ops)
}

fn main() {
    let cli = Cli::parse();
    match process(&cli.input, &cli.template) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(process(input, "{split:,:1..3:join:-}").unwrap(), "b-c");
        assert_eq!(process(input, "{split:,:-2..:join:_}").unwrap(), "d_e");
        assert_eq!(process(input, "{split:,:1:join:-}").unwrap(), "b");
        assert_eq!(process(input, "{split:,:..:join:-}").unwrap(), "a-b-c-d-e");
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
            process(input, "{split:,:..:trim:append:!}").unwrap(),
            "a!,b!,c!,d!,e!"
        );
        assert_eq!(
            process(input, "{split:,:..:trim:prepend:_}").unwrap(),
            "_a,_b,_c,_d,_e"
        );
    }

    #[test]
    fn test_chain() {
        let input = "first,second,third";
        // Original test
        let template = "{split:,:1:replace:s/second/hello/:upper}";
        assert_eq!(process(input, template).unwrap(), "HELLO");

        // Split, replace, lower
        let template = "{split:,:1:replace:s/second/hello/:lower}";
        assert_eq!(process(input, template).unwrap(), "hello");

        // Split, replace, trim (no effect, but test chain)
        let template = "{split:,:1:replace:s/second/ hello /:trim}";
        assert_eq!(process(input, template).unwrap(), "hello");

        // Split, upper, append
        let template = "{split:,:2:upper:append:!}";
        assert_eq!(process(input, template).unwrap(), "THIRD!");

        // Split, lower, prepend
        let template = r"{split:,:0:lower:prepend:word\: }";
        assert_eq!(process(input, template).unwrap(), "word: first");

        // Split range, join, upper
        let template = "{split:,:0..2:join:_:upper}";
        assert_eq!(process(input, template).unwrap(), "FIRST_SECOND");

        // Split range, join, replace, lower
        let template = "{split:,:0..2:join:-:replace:s/first/1/:lower}";
        assert_eq!(process(input, template).unwrap(), "1-second");

        // Split, replace, slice (get first 2 chars)
        let template = "{split:,:1:replace:s/second/hello/:slice:0..2}";
        assert_eq!(process(input, template).unwrap(), "he");

        // Split, replace, slice (last 2 chars)
        let template = "{split:,:1:replace:s/second/hello/:slice:-2..}";
        assert_eq!(process(input, template).unwrap(), "lo");

        // Split, strip, upper
        let input = "  first , second , third  ";
        let template = "{split:,:1:strip: :upper}";
        assert_eq!(process(input, template).unwrap(), "SECOND");

        // Split, join, append, upper
        let input = "a,b,c";
        let template = "{split:,:..:join:-:append:! :upper}";
        assert_eq!(process(input, template).unwrap(), "A-B-C! ");

        // Split, join, prepend, lower
        let template = r"{split:,:..:join:_:prepend:joined\: }";
        assert_eq!(process(input, template).unwrap(), "joined: a_b_c");

        // Split, trim, join, replace, upper
        let input = "  x, y ,z ";
        let template = "{split:,:..:trim:join: :replace:s/ /_/g:upper}";
        assert_eq!(process(input, template).unwrap(), "X_Y_Z");

        // Split, join, replace, slice
        let input = "foo,bar,baz";
        let template = "{split:,:..:join:-:replace:s/bar/xxx/:slice:0..7}";
        assert_eq!(process(input, template).unwrap(), "foo-xxx");

        // Split, join, replace, slice, lower
        let template = "{split:,:..:join:-:replace:s/bar/XXX/:slice:0..7:lower}";
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
            process(input, r"{split:,:..:append:\:x\:y\:z}").unwrap(),
            "a:x:y:z,b:x:y:z,c:x:y:z"
        );
        // Prepend with colons to list
        assert_eq!(
            process(input, r"{split:,:..:prepend:x\:y\:z\:}").unwrap(),
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
            process(input, r"{prepend:\:start\::append:\:end\::upper}").unwrap(),
            ":START:FOO:END:"
        );
        // On a list
        let input = "a,b";
        assert_eq!(
            process(input, r"{split:,:..:prepend:x\::append:\:y}").unwrap(),
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
            process(input, r"{split:,:..:prepend:\::append:\\\\}").unwrap(),
            r":a\\,:b\\"
        );
    }

    #[test]
    fn test_escaped_colon_in_replace() {
        let input = "foo:bar:baz";
        // Replace literal colon with literal backslash
        assert_eq!(
            process(input, r"{replace:s/\:/\\/g}").unwrap(),
            r"foo\bar\baz"
        );
        // Replace literal backslash with colon
        let input = r"foo\bar\baz";
        assert_eq!(
            process(input, r"{replace:s/\\\\/\:/g}").unwrap(),
            "foo:bar:baz"
        );
    }

    // New edge case tests
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
        assert_eq!(process("", "{split:,:..:slice:0}").unwrap(), "");
        assert_eq!(process("", "{split:,:..:slice:1..3}").unwrap(), "");
    }

    #[test]
    fn test_invalid_regex() {
        // Should handle invalid regex gracefully
        assert!(process("test", "{replace:s/[/replacement/}").is_err());
        assert!(process("test", "{replace:s/*/replacement/}").is_err());
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
        assert_eq!(process(input, "{split:,:..:upper}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..:lower}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..:trim}").unwrap(), "");
        assert_eq!(process(input, "{split:,:..:append:!}").unwrap(), "!");
        assert_eq!(process(input, "{split:,:..:prepend:_}").unwrap(), "_");
    }

    #[test]
    fn test_final_output_behavior() {
        // Test documented behavior: List joins with last split separator or space
        let input = "a,b,c";

        // With split operation - should use comma
        assert_eq!(process(input, "{split:,:..:upper}").unwrap(), "A,B,C");

        // Without split operation - should use space (no split occurred)
        assert_eq!(process("hello world", "{upper}").unwrap(), "HELLO WORLD");

        // Multiple splits - should use last split separator
        assert_eq!(
            process(input, "{split:,:..:join:-:split:-:..:upper}").unwrap(),
            "A-B-C"
        );
    }
}

