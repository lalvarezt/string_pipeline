use pest::Parser;
use pest_derive::Parser;

use super::{PadDirection, RangeSpec, SortDirection, StringOp, TrimDirection};

#[derive(Parser)]
#[grammar = "pipeline/template.pest"]
struct TemplateParser;

pub fn parse_template(template: &str) -> Result<(Vec<StringOp>, bool), String> {
    let pairs = TemplateParser::parse(Rule::template, template)
        .map_err(|e| format!("Parse error: {}", e))?
        .next()
        .unwrap();

    // Pre-allocate operations vector with estimated capacity
    let mut ops = Vec::with_capacity(16);
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

fn parse_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    match pair.as_rule() {
        Rule::shorthand_range => {
            let range = parse_range_spec(pair)?;
            Ok(StringOp::Split {
                sep: " ".to_string(),
                range,
            })
        }
        Rule::shorthand_index => {
            let idx = pair.as_str().parse().unwrap();
            Ok(StringOp::Split {
                sep: " ".to_string(),
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

fn extract_single_arg(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    let inner = pair.into_inner().next().unwrap();
    Ok(process_arg(inner.as_str()))
}

fn extract_single_arg_raw(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    Ok(pair.into_inner().next().unwrap().as_str().to_string())
}

fn extract_range_arg(pair: pest::iterators::Pair<Rule>) -> Result<RangeSpec, String> {
    parse_range_spec(pair.into_inner().next().unwrap())
}

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

fn parse_regex_extract_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let mut parts = pair.into_inner();
    let pattern = parts.next().unwrap().as_str().to_string();
    let group = parts.next().and_then(|p| p.as_str().parse().ok());
    Ok(StringOp::RegexExtract { pattern, group })
}

fn parse_map_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let map_op_pair = pair.into_inner().next().unwrap();
    let operation_list_pair = map_op_pair.into_inner().next().unwrap();

    let mut operations = Vec::with_capacity(16);
    for op_pair in operation_list_pair.into_inner() {
        let inner_op_pair = op_pair.into_inner().next().unwrap();
        operations.push(parse_map_inner_operation(inner_op_pair)?);
    }

    Ok(StringOp::Map { operations })
}

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

fn process_arg(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_string();
    }

    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some(':') => result.push(':'),
                Some('|') => result.push('|'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('{') => result.push('{'),
                Some('}') => result.push('}'),
                Some(next) => result.push(next),
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_sed_string(pair: pest::iterators::Pair<Rule>) -> Result<(String, String, String), String> {
    let mut parts = pair.into_inner();

    let pattern_str = parts.next().unwrap().as_str();
    let replacement_str = parts.next().unwrap().as_str();
    let flags_str = parts.next().map(|p| p.as_str()).unwrap_or("");

    if pattern_str.is_empty() {
        return Err("Empty pattern in sed string".to_string());
    }

    Ok((
        pattern_str.to_string(),
        replacement_str.to_string(),
        flags_str.to_string(),
    ))
}

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
                .map_err(|_| format!("Invalid index: {}", idx_str))?;
            Ok(RangeSpec::Index(idx))
        }
        _ => Err(format!("Unknown range spec: {:?}", inner.as_rule())),
    }
}
