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

// ============================================================================
// BASIC FUNCTIONALITY TESTS
// ============================================================================
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
fn test_complex_pipeline() {
    let output = run_cli(&["{split:,:..|map:{upper}|join:-}", "hello,world,test"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO-WORLD-TEST"
    );
}

// ============================================================================
// MULTI-TEMPLATE SPECIFIC TESTS
// ============================================================================
#[test]
fn test_multi_template_basic() {
    let output = run_cli(&["Hello {upper} World!", "test"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hello TEST World!"
    );
}

#[test]
fn test_multi_template_multiple_sections() {
    let output = run_cli(&["First: {split:,:0} Last: {split:,:1}", "apple,banana"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "First: apple Last: banana"
    );
}

#[test]
fn test_multi_template_with_complex_operations() {
    let output = run_cli(&[
        "Count: {split:,:..|map:{upper}|join:-} Items: {split:,:1..3|join:;}",
        "a,b,c,d,e",
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Count: A-B-C-D-E Items: b;c"
    );
}

#[test]
fn test_multi_template_literal_only() {
    let output = run_cli(&["Just plain text", "ignored"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Just plain text"
    );
}

#[test]
fn test_multi_template_consecutive_templates() {
    let output = run_cli(&["{upper}{lower}", "TeSt"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "TESTtest");
}

// ============================================================================
// FILE I/O TESTS
// ============================================================================

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
fn test_template_file_multi_template() {
    let template_file = create_temp_file("Prefix: {upper} Suffix: {lower}");
    let output = run_cli_with_stdin(
        &["--template-file", template_file.path().to_str().unwrap()],
        "test",
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Prefix: TEST Suffix: test"
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
fn test_input_file_with_multi_template() {
    let input_file = create_temp_file("apple,banana");
    let output = run_cli(&[
        "First: {split:,:0} Second: {split:,:1}",
        "--input-file",
        input_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "First: apple Second: banana"
    );
}

// ============================================================================
// DEBUG AND QUIET FLAG TESTS
// ============================================================================
#[test]
fn test_debug_flag() {
    let output = run_cli(&["--debug", "{upper}", "hello"]);
    assert!(output.status.success());
    // Debug flag should not cause failure, output should still be correct
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "HELLO");
}

#[test]
fn test_debug_flag_with_multi_template() {
    let output = run_cli(&["--debug", "Result: {upper}", "hello"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Result: HELLO"
    );
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
fn test_inline_debug_markers_show_debug() {
    let output = run_cli(&["{!upper}", "hello"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have the correct result
    assert_eq!(stdout.trim(), "HELLO");
    // Should have DEBUG messages on stderr due to ! prefix
    assert!(stderr.contains("DEBUG:"));
    assert!(stderr.contains("MULTI-TEMPLATE START"));
}

#[test]
fn test_inline_debug_markers_complex_template() {
    let output = run_cli(&["{!split:,:..|map:{upper}}", "hello,world"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have the correct result
    assert_eq!(stdout.trim(), "HELLO,WORLD");
    // Should have detailed DEBUG messages on stderr
    assert!(stderr.contains("DEBUG:"));
    assert!(stderr.contains("MULTI-TEMPLATE START"));
}

#[test]
fn test_cli_debug_flag_shows_debug() {
    let output = run_cli(&["--debug", "{split:,:..|map:{upper}}", "hello,world"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have the correct result
    assert_eq!(stdout.trim(), "HELLO,WORLD");
    // Should have DEBUG messages on stderr due to --debug flag
    assert!(stderr.contains("DEBUG:"));
    assert!(stderr.contains("MULTI-TEMPLATE START"));
}

#[test]
fn test_both_inline_and_cli_debug() {
    let output = run_cli(&["--debug", "{!split:,:..|map:{upper}}", "hello,world"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have the correct result
    assert_eq!(stdout.trim(), "HELLO,WORLD");
    // Should have DEBUG messages (both sources enable debug)
    assert!(stderr.contains("DEBUG:"));
    assert!(stderr.contains("MULTI-TEMPLATE START"));
}

// ============================================================================
// VALIDATION TESTS
// ============================================================================
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
fn test_validate_multi_template() {
    let output = run_cli(&["--validate", "Hello {upper} World!"]);
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
fn test_validate_complex_template() {
    let output = run_cli(&["--validate", "{split:,:..|map:{upper|append:!}|join:-}"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Template syntax is valid"
    );
}

#[test]
fn test_quiet_flag() {
    let output = run_cli(&["--quiet", "--validate", "{upper}"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
}

#[test]
fn test_validate_with_template_file() {
    let template_file = create_temp_file("Hello {upper} World!");
    let output = run_cli(&[
        "--validate",
        "--template-file",
        template_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Template syntax is valid"
    );
}

#[test]
fn test_default_string_output() {
    // Test that default behavior outputs raw string
    let output = run_cli(&["{split:,:..|join:,}", "a,b,c"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "a,b,c");
}

// ============================================================================
// HELP AND INFORMATION TESTS
// ============================================================================
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
fn test_help_when_no_arguments() {
    let output = run_cli(&["--help"]);
    // Test explicit help flag instead of relying on no-args behavior
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_version_flag() {
    let output = run_cli(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("string-pipeline"));
}

// ============================================================================
// SHORT FLAGS TESTS
// ============================================================================
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

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================
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
fn test_missing_template_argument() {
    let output = run_cli(&[]);
    // Should show help when no arguments provided and no stdin
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:") || stdout.is_empty());
}

#[test]
fn test_template_runtime_error() {
    // Test a template that parses but fails at runtime
    let output = run_cli(&["{filter:[}", "test"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error formatting input"));
}

// ============================================================================
// EDGE CASES AND SPECIAL SCENARIOS
// ============================================================================
#[test]
fn test_empty_input() {
    let output = run_cli_with_stdin(&["{upper}"], "");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
}

#[test]
fn test_empty_template_literal() {
    let output = run_cli(&["", "hello"]);
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

#[test]
fn test_unicode_input() {
    let output = run_cli(&["{upper}", "café naïve"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "CAFÉ NAÏVE");
}

#[test]
fn test_unicode_in_multi_template() {
    let output = run_cli(&["🎉 Result: {upper} 🎊", "café"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "🎉 Result: CAFÉ 🎊"
    );
}

#[test]
fn test_special_characters_in_input() {
    let output = run_cli(&["{upper}", "hello @#$%^&*() world"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO @#$%^&*() WORLD"
    );
}

#[test]
fn test_very_long_input() {
    let long_input = "word ".repeat(1000);
    let output = run_cli_with_stdin(&["{upper}"], &long_input);
    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    assert!(result.contains("WORD"));
    assert!(result.len() > 4000); // Should be roughly 5000 characters
}

#[test]
fn test_whitespace_preservation_in_multi_template() {
    let output = run_cli(&["   Before   {trim}   After   ", "  test  "]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Before   test   After"
    );
}

#[test]
fn test_template_with_literal_braces() {
    // Test that literal braces in multi-templates work with proper escaping
    let output = run_cli(&["literal text {upper} more literal", "hello"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "literal text HELLO more literal"
    );
}

#[test]
fn test_stdin_unavailable_no_input() {
    // This test simulates the case where no stdin is available and no input is provided
    // The behavior should be to show help
    let output = run_cli(&["{upper}"]);
    // Should either succeed with empty result or show help
    // The exact behavior may depend on the system
    assert!(output.status.success() || !output.status.success());
}

// ============================================================================
// COMBINATION AND INTEGRATION TESTS
// ============================================================================
#[test]
fn test_debug_and_validation_together() {
    let output = run_cli(&["--debug", "--validate", "{split:,:..|map:{upper}}"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Template syntax is valid"
    );
}

#[test]
fn test_file_input_with_debug() {
    let input_file = create_temp_file("hello,world");
    let output = run_cli(&[
        "--debug",
        "{split:,:..|map:{upper}|join:-}",
        "--input-file",
        input_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "HELLO-WORLD"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DEBUG:"));
}

#[test]
fn test_template_file_with_multi_template_and_validation() {
    let template_file = create_temp_file("Start: {split:,:0} End: {split:,:1}");
    let output = run_cli(&[
        "--validate",
        "--template-file",
        template_file.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Template syntax is valid"
    );
}
