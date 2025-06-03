use clap::Parser;
use std::io::{self, Read};
use string_pipeline::Template;

#[derive(Parser)]
struct Cli {
    /// The template string
    template: String,
    /// The input string (if not provided, reads from stdin)
    input: Option<String>,
}

fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from stdin: {}", e))?;
    Ok(buffer)
}

fn main() {
    let cli = Cli::parse();

    // Get input from argument or stdin
    let input = match cli.input {
        Some(input) => input,
        None => match read_stdin() {
            Ok(input) => input.trim_end().to_string(),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    };

    let template = Template::parse(&cli.template).unwrap_or_else(|e| {
        eprintln!("Error parsing template: {}", e);
        std::process::exit(1);
    });

    match template.format(&input) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("Error formatting input: {}", e);
            std::process::exit(1);
        }
    }
}
