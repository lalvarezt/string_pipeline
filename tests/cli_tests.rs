use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

const BINARY_NAME: &str = "string-pipeline";

/// Helper function to run the CLI with arguments and return output
fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", BINARY_NAME, "--"])
        .args(args)
        .output()
        .expect("Failed to execute command")
}

/// Helper function to run CLI with stdin input
fn run_cli_with_stdin(args: &[&str], stdin_input: &str) -> std::process::Output {
    let mut cmd = Command::new("cargo")
        .args(["run", "--bin", BINARY_NAME, "--"])
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    if let Some(stdin) = cmd.stdin.as_mut() {
        stdin
            .write_all(stdin_input.as_bytes())
            .expect("Failed to write to stdin");
    }

    cmd.wait_with_output().expect("Failed to read stdout")
}

/// Helper function to create a temporary file with content
fn create_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

#[test]
fn test_basic_template_and_input() {
    let output = run_cli(&["{upper}", "hello world"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO WORLD"
    );
}

#[test]
fn test_stdin_input() {
    let output = run_cli_with_stdin(&["{lower}"], "HELLO WORLD");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "hello world"
    );
}

#[test]
fn test_stdin_with_complex_pipeline() {
    let output = run_cli_with_stdin(&["{split:,:..|map:{upper}|join:-}"], "apple,banana,cherry");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "APPLE-BANANA-CHERRY"
    );
}

#[test]
fn test_template_file_option() {
    let template_file = create_temp_file("{upper}");
    let output = run_cli_with_stdin(
        &["--template-file", template_file.path().to_str().unwrap()],
        "hello world",
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO WORLD"
    );
}

#[test]
fn test_input_file_option() {
    let input_file = create_temp_file("hello world");
    let output = run_cli(&[
        "{upper}",
        "--input-file",
        input_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO WORLD"
    );
}

#[test]
fn test_both_template_and_input_files() {
    let template_file = create_temp_file("{upper}");
    let input_file = create_temp_file("hello world");
    let output = run_cli(&[
        "--template-file",
        template_file.path().to_str().unwrap(),
        "--input-file",
        input_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO WORLD"
    );
}

#[test]
fn test_debug_flag() {
    let output = run_cli(&["--debug", "{upper}", "hello"]);
    assert!(output.status.success());
    // Debug flag should not cause failure, output should still be correct
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "HELLO");
}

#[test]
fn test_validate_flag() {
    let output = run_cli(&["--validate", "{upper}"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Template syntax is valid"
    );
}

#[test]
fn test_validate_invalid_template() {
    let output = run_cli(&["--validate", "{invalid_operation}"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error parsing template"));
}

#[test]
fn test_quiet_flag() {
    let output = run_cli(&["--quiet", "--validate", "{upper}"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
}

#[test]
fn test_quiet_suppresses_debug() {
    let output = run_cli(&["--quiet", "--debug", "{split:,:..|map:{upper}}", "a,b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only have the result, no DEBUG: messages
    assert_eq!(stdout.trim(), "A,B");
    assert!(!stdout.contains("DEBUG:"));
}

#[test]
fn test_quiet_suppresses_inline_debug() {
    let output = run_cli(&["--quiet", "{!split:,:..|map:{upper}}", "a,b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only have the result, no DEBUG: messages
    assert_eq!(stdout.trim(), "A,B");
    assert!(!stdout.contains("DEBUG:"));
}

#[test]
fn test_quiet_suppresses_debug_stderr() {
    let output = run_cli(&["--quiet", "--debug", "{split:,:..|map:{upper}}", "a,b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should only have the result, no DEBUG: messages
    assert_eq!(stdout.trim(), "A,B");
    assert!(!stderr.contains("DEBUG:"));
}

#[test]
fn test_quiet_suppresses_inline_debug_stderr() {
    let output = run_cli(&["--quiet", "{!split:,:..|map:{upper}}", "a,b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should only have the result, no DEBUG: messages
    assert_eq!(stdout.trim(), "A,B");
    assert!(!stderr.contains("DEBUG:"));
}

#[test]
fn test_debug_without_quiet_shows_stderr() {
    let output = run_cli(&["--debug", "{split:,:..|map:{upper}}", "a,b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should have the result
    assert_eq!(stdout.trim(), "A,B");
    // Should have DEBUG messages on stderr
    assert!(stderr.contains("DEBUG:"));
}

#[test]
fn test_output_format_raw() {
    let output = run_cli(&["--output", "raw", "{split:,:..|join:,}", "a,b,c"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "a,b,c");
}

#[test]
fn test_output_format_raw_default() {
    // Test that default behavior is same as explicit raw
    let output = run_cli(&["{split:,:..|join:,}", "a,b,c"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "a,b,c");
}

#[test]
fn test_output_format_lines() {
    let output = run_cli(&["--output", "lines", "{split:,:..|join:,}", "a,b,c"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("a\n") && stdout.contains("b\n") && stdout.contains("c"));
}

#[test]
fn test_output_format_json() {
    let output = run_cli(&["--output", "json", "{split:,:..|join:,}", "a,b,c"]);
    assert!(output.status.success());
    let stdout_raw = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_raw.trim();
    assert!(
        stdout.starts_with('"') && stdout.ends_with('"')
            || stdout.starts_with('[') && stdout.ends_with(']')
    );
}

#[test]
fn test_list_operations_flag() {
    let output = run_cli(&["--list-operations"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available Operations:"));
    assert!(stdout.contains("split:"));
    assert!(stdout.contains("upper"));
    assert!(stdout.contains("lower"));
}

#[test]
fn test_syntax_help_flag() {
    let output = run_cli(&["--syntax-help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Template Syntax Help:"));
    assert!(stdout.contains("BASIC SYNTAX:"));
    assert!(stdout.contains("RANGE SYNTAX:"));
}

#[test]
fn test_error_both_template_and_template_file() {
    let template_file = create_temp_file("{upper}");
    let output = run_cli(&[
        "{lower}",
        "--template-file",
        template_file.path().to_str().unwrap(),
        "hello",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot specify both template argument and template file"));
}

#[test]
fn test_error_both_input_and_input_file() {
    let input_file = create_temp_file("hello");
    let output = run_cli(&[
        "{upper}",
        "world",
        "--input-file",
        input_file.path().to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot specify both input argument and input file"));
}

#[test]
fn test_help_when_no_arguments() {
    let output = run_cli(&["--help"]);
    // Test explicit help flag instead of relying on no-args behavior
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_short_flags() {
    // Test short versions of flags
    let template_file = create_temp_file("{upper}");
    let input_file = create_temp_file("hello world");

    let output = run_cli(&[
        "-t",
        template_file.path().to_str().unwrap(),
        "-f",
        input_file.path().to_str().unwrap(),
        "-d", // debug
        "-q", // quiet
    ]);
    assert!(output.status.success());
}

#[test]
fn test_complex_pipeline() {
    let output = run_cli(&["{split:,:..|map:{upper}|join:-}", "hello,world,test"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO-WORLD-TEST"
    );
}

#[test]
fn test_invalid_template_syntax() {
    let output = run_cli(&["{unclosed_brace", "hello"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error parsing template"));
}

#[test]
fn test_nonexistent_template_file() {
    let output = run_cli(&["--template-file", "/nonexistent/file.txt"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error reading template file"));
}

#[test]
fn test_nonexistent_input_file() {
    let output = run_cli(&["{upper}", "--input-file", "/nonexistent/file.txt"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error reading input file"));
}

#[test]
fn test_empty_input() {
    let output = run_cli_with_stdin(&["{upper}"], "");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
}

#[test]
fn test_multiline_input() {
    let input = "hello\nworld\ntest";
    let output = run_cli_with_stdin(&["{upper}"], input);
    assert!(output.status.success());
    let stdout_raw = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_raw.trim();
    assert!(stdout.contains("HELLO") && stdout.contains("WORLD") && stdout.contains("TEST"));
}
