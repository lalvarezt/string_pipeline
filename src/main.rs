use clap::{CommandFactory, Parser};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use string_pipeline::{MultiTemplate, Template};

#[derive(Parser)]
#[command(
    name = "string-pipeline",
    version,
    about = "Powerful CLI tool and Rust library for chainable string transformations using intuitive template syntax",
    long_about = "A powerful string transformation CLI tool and Rust library that makes complex text processing \
        simple. Transform data using intuitive template syntax — chain operations like split, join, replace, filter, \
        and others in a single readable expression. Supports both single templates (e.g., '{upper}') and multi-templates \
        with mixed text and operations (e.g., 'Name: {split: :0} Age: {split: :1}') with intelligent caching."
)]
struct Cli {
    /// The template string to apply
    #[arg(value_name = "TEMPLATE")]
    template: Option<String>,

    /// The input string (if not provided, reads from stdin)
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    /// Read template from file instead of command line
    #[arg(short = 't', long = "template-file", value_name = "FILE")]
    template_file: Option<PathBuf>,

    /// Read input from file instead of stdin/argument
    #[arg(short = 'f', long = "input-file", value_name = "FILE")]
    input_file: Option<PathBuf>,

    /// Force debug mode (equivalent to adding ! to template start)
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// Validate template syntax without processing input
    #[arg(long = "validate")]
    validate: bool,

    /// Output format for results
    #[arg(short = 'o', long = "output", value_enum, default_value = "raw")]
    output_format: OutputFormat,

    /// Suppress all output except the final result
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Show available operations and exit
    #[arg(long = "list-operations")]
    list_operations: bool,

    /// Show template syntax help and exit
    #[arg(long = "syntax-help")]
    syntax_help: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// Output as raw string (default behavior)
    Raw,
    /// Output each item on separate lines
    Lines,
    /// Output as JSON array/string
    Json,
}

fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from stdin: {}", e))?;
    Ok(buffer)
}

fn is_stdin_available() -> bool {
    use std::io::IsTerminal;
    !io::stdin().is_terminal()
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))
}

fn show_operations_help() {
    println!("Available Operations:");
    println!(
        "
  split:SEP:RANGE          - Split text into parts
  slice:RANGE              - Extract range of items
  join:SEP                 - Combine items with separator
  substring:RANGE          - Extract characters from string
  trim[:CHARS][:DIR]       - Remove characters from ends
  pad:WIDTH[:CHAR][:DIR]   - Add padding to reach width
  upper                    - Convert to uppercase
  lower                    - Convert to lowercase
  append:TEXT              - Add text to end
  prepend:TEXT             - Add text to beginning
  replace:s/PAT/REP/FLAGS  - Find and replace with regex
  regex_extract:PAT[:GRP]  - Extract with regex pattern
  sort[:DIR]               - Sort items alphabetically
  reverse                  - Reverse order or characters
  unique                   - Remove duplicates
  filter:PATTERN           - Keep items matching pattern
  filter_not:PATTERN       - Remove items matching pattern
  strip_ansi               - Remove ANSI color codes
  map:{{operations}}       - Apply operations to each item

Use 'string-pipeline --syntax-help' for detailed syntax information.
"
    );
}

fn show_syntax_help() {
    println!("Template Syntax Help:");
    println!(
        "
BASIC SYNTAX:
  {{operation1|operation2|operation3}}

MULTI-TEMPLATE SYNTAX:
  literal text {{operation}} more text {{operation}}

RANGE SYNTAX:
  N        - Single index (5 = 6th item, 0-indexed)
  N..M     - Range exclusive (1..3 = items 1,2)
  N..=M    - Range inclusive (1..=3 = items 1,2,3)
  N..      - From N to end (2.. = from 3rd item)
  ..M      - From start to M-1 (..3 = first 3 items)
  ..       - All items

SINGLE TEMPLATE EXAMPLES:
  {{split:,:..|map:{{upper}}|join:-}}
  {{trim|split: :..|filter:^[A-Z]|sort}}
  {{!split:,:..|slice:1..3}}  (debug mode)

MULTI-TEMPLATE EXAMPLES:
  'Name: {{split: :0}} Age: {{split: :1}}'
  'First: {{split:,:0}} Second: {{split:,:1}}'
  'some string {{split:,:1}} some string {{split:,:2}}'

CACHING:
  Multi-templates automatically cache repeated operations for efficiency.
  In 'A: {{split:,:0}} B: {{split:,:1}} C: {{split:,:0}}', split is
  calculated only once, with subsequent operations reusing the cached result.

ESCAPING:
  \\:  - Literal colon
  \\|  - Literal pipe
  \\}} - Literal closing brace
  \\n  - Newline
  \\t  - Tab

For complete documentation, visit:
https://github.com/lalvarezt/string_pipeline/blob/main/docs/template-system.md
"
    );
}

fn remove_debug_markers(template: &str) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    let mut depth: usize = 0;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if depth == 0 && chars.peek() == Some(&'!') {
                // Skip the '!' debug marker at top-level
                result.push(ch);
                chars.next(); // consume the '!'
                depth += 1;
            } else {
                result.push(ch);
                depth += 1;
            }
        } else if ch == '}' {
            result.push(ch);
            depth = depth.saturating_sub(1);
        } else {
            result.push(ch);
        }
    }
    result
}

fn main() {
    let cli = Cli::parse();

    // Handle help commands
    if cli.list_operations {
        show_operations_help();
        return;
    }

    if cli.syntax_help {
        show_syntax_help();
        return;
    }

    // Show help if no arguments and no stdin
    if cli.template.is_none() && cli.template_file.is_none() && !is_stdin_available() {
        Cli::command().print_help().unwrap();
        return;
    }

    // Get template from argument or file
    let template_str = match (&cli.template, &cli.template_file) {
        (Some(template), None) => template.clone(),
        (None, Some(file)) => match read_file(file) {
            Ok(content) => content.trim().to_string(),
            Err(e) => {
                eprintln!("Error reading template file: {}", e);
                std::process::exit(1);
            }
        },
        (Some(_), Some(_)) => {
            eprintln!("Error: Cannot specify both template argument and template file");
            std::process::exit(1);
        }
        (None, None) => {
            eprintln!("Error: Must provide either template argument or --template-file");
            std::process::exit(1);
        }
    };

    // Apply debug flags to template
    let final_template = if cli.debug && !template_str.contains("{!") {
        // Check if this will be a multi-template
        let is_multi_template_debug =
            if template_str.starts_with('{') && template_str.ends_with('}') {
                // Count top-level braces (not nested ones)
                let mut brace_count = 0;
                let mut depth = 0;
                let chars = template_str.chars();

                for ch in chars {
                    if ch == '{' {
                        depth += 1;
                        if depth == 1 {
                            brace_count += 1;
                        }
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }

                brace_count > 1
            } else {
                // Has text outside of braces, definitely multi-template
                true
            };

        if is_multi_template_debug {
            // Multi-template: add debug to each individual template section (top-level only)
            let mut result = String::new();
            let mut chars = template_str.chars().peekable();
            let mut depth: usize = 0;

            while let Some(ch) = chars.next() {
                if ch == '{' {
                    if depth == 0 && chars.peek() != Some(&'!') {
                        // Only add debug flag to top-level braces
                        result.push_str("{!");
                        depth += 1;
                    } else {
                        result.push(ch);
                        depth += 1;
                    }
                } else if ch == '}' {
                    result.push(ch);
                    depth = depth.saturating_sub(1);
                } else {
                    result.push(ch);
                }
            }
            result
        } else {
            // Single template: wrap with {!...}
            if let Some(stripped) = template_str.strip_prefix('{') {
                format!("{{!{}", stripped)
            } else {
                format!("{{!{}}}", template_str)
            }
        }
    } else {
        template_str
    };

    // Detect if this is a multi-template or single template
    // Multi-template: has literal text outside of braces OR multiple top-level template sections
    let is_multi_template = if final_template.starts_with('{') && final_template.ends_with('}') {
        // Count top-level braces (not nested ones)
        let mut brace_count = 0;
        let mut depth = 0;
        let chars = final_template.chars();

        for ch in chars {
            if ch == '{' {
                depth += 1;
                if depth == 1 {
                    brace_count += 1;
                }
            } else if ch == '}' {
                depth -= 1;
            }
        }

        brace_count > 1
    } else {
        // Has text outside of braces, definitely multi-template
        true
    };

    // Parse and validate template
    if is_multi_template {
        let _multi_template = MultiTemplate::parse(&final_template).unwrap_or_else(|e| {
            eprintln!("Error parsing template: {}", e);
            std::process::exit(1);
        });
    } else {
        let _template = Template::parse(&final_template).unwrap_or_else(|e| {
            eprintln!("Error parsing template: {}", e);
            std::process::exit(1);
        });
    }

    // If just validating, exit here
    if cli.validate {
        if !cli.quiet {
            println!("Template syntax is valid");
        }
        return;
    }

    // Get input from argument, file, or stdin
    let input = match (&cli.input, &cli.input_file) {
        (Some(input), None) => input.clone(),
        (None, Some(file)) => match read_file(file) {
            Ok(content) => content.trim_end().to_string(),
            Err(e) => {
                eprintln!("Error reading input file: {}", e);
                std::process::exit(1);
            }
        },
        (None, None) => match read_stdin() {
            Ok(input) => input.trim_end().to_string(),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        (Some(_), Some(_)) => {
            eprintln!("Error: Cannot specify both input argument and input file");
            std::process::exit(1);
        }
    };

    // For quiet mode, disable debug output by removing debug markers
    let final_template_for_processing = if cli.quiet {
        // Remove all debug markers from template to suppress debug output
        remove_debug_markers(&final_template)
    } else {
        final_template.clone()
    };

    // Process input with appropriate template type
    let result = if is_multi_template {
        // Parse and process with MultiTemplate
        let multi_template =
            MultiTemplate::parse(&final_template_for_processing).unwrap_or_else(|e| {
                eprintln!("Error parsing template: {}", e);
                std::process::exit(1);
            });

        match multi_template.format(&input) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error formatting input: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Parse and process with single Template
        let processing_template =
            Template::parse(&final_template_for_processing).unwrap_or_else(|e| {
                eprintln!("Error parsing template: {}", e);
                std::process::exit(1);
            });

        match processing_template.format(&input) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error formatting input: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Output result based on format
    match cli.output_format {
        OutputFormat::Raw => {
            print!("{}", result);
        }
        OutputFormat::Lines => {
            for line in result.split(',') {
                println!("{}", line);
            }
        }
        OutputFormat::Json => {
            if result.contains(',') && !result.starts_with('"') {
                let items: Vec<&str> = result.split(',').collect();
                println!(
                    "{}",
                    serde_json::to_string(&items)
                        .unwrap_or_else(|_| format!("[\"{}\"]", result.replace('"', "\\\"")))
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&result)
                        .unwrap_or_else(|_| format!("\"{}\"", result.replace('"', "\\\"")))
                );
            }
        }
    }
}
