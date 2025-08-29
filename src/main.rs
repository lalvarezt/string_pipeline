use clap::{CommandFactory, Parser};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use string_pipeline::MultiTemplate;

#[derive(Parser)]
#[command(
    name = "string-pipeline",
    version,
    about = "Powerful CLI tool and Rust library for chainable string transformations using intuitive template syntax",
    long_about = "A powerful string transformation CLI tool and Rust library that makes complex text processing \
        simple. Transform data using intuitive template syntax — chain operations like split, join, replace, filter, \
        and others in a single readable expression. Supports templates with mixed text and operations \
        (e.g., 'Name: {split: :0} Age: {split: :1}') with intelligent caching for efficiency."
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

/// Processed configuration from CLI arguments
struct Config {
    template: String,
    input: Option<String>,
    validate: bool,
    quiet: bool,
    debug: bool,
}

/// Read content from a file with proper error handling
fn read_file(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))
}

/// Read from stdin with proper error handling
fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from stdin: {e}"))?;
    Ok(buffer)
}

/// Check if stdin is available (not a terminal)
fn is_stdin_available() -> bool {
    use std::io::IsTerminal;
    !io::stdin().is_terminal()
}

/// Get template string from CLI arguments
fn get_template(cli: &Cli) -> Result<String, String> {
    match (&cli.template, &cli.template_file) {
        (Some(template), None) => Ok(template.clone()),
        (None, Some(file)) => read_file(file)
            .map(|content| content.trim().to_string())
            .map_err(|e| format!("Error reading template file: {e}")),
        (Some(_), Some(_)) => {
            Err("Error: Cannot specify both template argument and template file".to_string())
        }
        (None, None) => {
            Err("Error: Must provide either template argument or --template-file".to_string())
        }
    }
}

/// Get input string from CLI arguments
fn get_input(cli: &Cli) -> Result<String, String> {
    match (&cli.input, &cli.input_file) {
        (Some(input), None) => Ok(input.clone()),
        (None, Some(file)) => read_file(file)
            .map(|content| content.trim_end().to_string())
            .map_err(|e| format!("Error reading input file: {e}")),
        (None, None) => read_stdin().map(|input| input.trim_end().to_string()),
        (Some(_), Some(_)) => {
            Err("Error: Cannot specify both input argument and input file".to_string())
        }
    }
}

/// Build configuration from CLI arguments
fn build_config(cli: Cli) -> Result<Config, String> {
    let template = get_template(&cli)?;

    // Skip input collection if we're only validating the template
    let input = if cli.validate {
        None
    } else {
        Some(get_input(&cli)?)
    };

    Ok(Config {
        template,
        input,
        validate: cli.validate,
        quiet: cli.quiet,
        debug: cli.debug,
    })
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
  surround:CHARS           - Add characters to both ends
  quote:CHARS              - Add characters to both ends (alias)
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

MIXED TEXT SYNTAX:
  literal text {{operation}} more text {{operation}}

RANGE SYNTAX:
  N        - Single index (5 = 6th item, 0-indexed)
  N..M     - Range exclusive (1..3 = items 1,2)
  N..=M    - Range inclusive (1..=3 = items 1,2,3)
  N..      - From N to end (2.. = from 3rd item)
  ..M      - From start to M-1 (..3 = first 3 items)
  ..       - All items

OPERATION-ONLY EXAMPLES:
  {{split:,:..|map:{{upper}}|join:-}}
  {{trim|split: :..|filter:^[A-Z]|sort}}
  {{!split:,:..|slice:1..3}}  (debug mode)

MIXED TEXT EXAMPLES:
  'Name: {{split: :0}} Age: {{split: :1}}'
  'First: {{split:,:0}} Second: {{split:,:1}}'
  'some string {{split:,:1}} some string {{split:,:2}}'

CACHING:
  Templates automatically cache split results for efficiency.
  In 'A: {{split:,:0}} B: {{split:,:1}} C: {{split:,:0}}', the input is
  split only once, with subsequent operations reusing the cached split result.

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

fn main() {
    let cli = Cli::parse();

    // Handle help commands first
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

    // Build configuration from CLI arguments
    let config = build_config(cli).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    // Parse template and handle debug mode from both template prefix and CLI flag
    let template = MultiTemplate::parse_with_debug(&config.template, None).unwrap_or_else(|e| {
        eprintln!("Error parsing template: {e}");
        std::process::exit(1);
    });

    // Enable debug if either the template has ! prefix OR the CLI debug flag is set
    // Disable debug only if quiet mode is enabled
    let should_debug = (template.is_debug() || config.debug) && !config.quiet;
    let template = template.with_debug(should_debug);

    // If just validating, exit here
    if config.validate {
        if !config.quiet {
            println!("Template syntax is valid");
        }
        return;
    }

    // For non-validation, input is required
    let input = config
        .input
        .expect("Input should be available for non-validation operations");

    // Process input with template
    let result = template.format(&input).unwrap_or_else(|e| {
        eprintln!("Error formatting input: {e}");
        std::process::exit(1);
    });

    // Output result as string
    print!("{result}");
}
