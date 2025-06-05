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

    let mut ops = Vec::new();
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
            let sep = process_arg(parts.next().unwrap().as_str());
            let range = parts
                .next()
                .map_or_else(|| Ok(RangeSpec::Range(None, None, false)), parse_range_spec)?;
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
        Rule::trim => Ok(StringOp::Trim {
            direction: parse_trim_direction(pair),
        }),
        Rule::strip => Ok(StringOp::Strip {
            chars: extract_single_arg_raw(pair)?,
        }),
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

// Helper functions to reduce repetition

fn extract_single_arg(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    Ok(process_arg(pair.into_inner().next().unwrap().as_str()))
}

fn extract_single_arg_raw(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
    Ok(pair.into_inner().next().unwrap().as_str().to_string())
}

fn extract_range_arg(pair: pest::iterators::Pair<Rule>) -> Result<RangeSpec, String> {
    parse_range_spec(pair.into_inner().next().unwrap())
}

fn parse_trim_direction(pair: pest::iterators::Pair<Rule>) -> TrimDirection {
    pair.into_inner()
        .next()
        .map(|p| match p.as_str() {
            "left" => TrimDirection::Left,
            "right" => TrimDirection::Right,
            "both" => TrimDirection::Both,
            _ => TrimDirection::Both,
        })
        .unwrap_or(TrimDirection::Both)
}

fn parse_sort_direction(pair: pest::iterators::Pair<Rule>) -> SortDirection {
    pair.into_inner()
        .next()
        .map(|p| match p.as_str() {
            "desc" => SortDirection::Desc,
            "asc" => SortDirection::Asc,
            _ => SortDirection::Asc,
        })
        .unwrap_or(SortDirection::Asc)
}

fn parse_pad_operation(pair: pest::iterators::Pair<Rule>) -> Result<StringOp, String> {
    let mut parts = pair.into_inner();
    let width = parts
        .next()
        .unwrap()
        .as_str()
        .parse()
        .map_err(|_| "Invalid padding width")?;
    let char = parts
        .next()
        .map(|p| process_arg(p.as_str()).chars().next().unwrap_or(' '))
        .unwrap_or(' ');
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

    let mut operations = Vec::new();
    for op_pair in operation_list_pair.into_inner() {
        let inner_op_pair = op_pair.into_inner().next().unwrap();
        operations.push(parse_operation(inner_op_pair)?);
    }

    Ok(StringOp::Map { operations })
}

fn process_arg(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => {
                    result.push('\n');
                    chars.next();
                }
                Some('t') => {
                    result.push('\t');
                    chars.next();
                }
                Some('r') => {
                    result.push('\r');
                    chars.next();
                }
                Some(':') => {
                    result.push(':');
                    chars.next();
                }
                Some('|') => {
                    result.push('|');
                    chars.next();
                }
                Some('\\') => {
                    result.push('\\');
                    chars.next();
                }
                Some('/') => {
                    result.push('/');
                    chars.next();
                }
                Some('{') => {
                    result.push('{');
                    chars.next();
                }
                Some('}') => {
                    result.push('}');
                    chars.next();
                }
                Some(&next) => {
                    result.push(next);
                    chars.next();
                }
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

    let pattern = parts.next().unwrap().as_str().to_string();
    let replacement = parts.next().unwrap().as_str().to_string();
    let flags = parts
        .next()
        .map_or_else(String::new, |p| p.as_str().to_string());

    if pattern.is_empty() {
        return Err("Empty pattern in sed string".to_string());
    }

    Ok((pattern, replacement, flags))
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
            let idx = inner.into_inner().next().unwrap().as_str().parse().unwrap();
            Ok(RangeSpec::Index(idx))
        }
        _ => Err(format!("Unknown range spec: {:?}", inner.as_rule())),
    }
}
