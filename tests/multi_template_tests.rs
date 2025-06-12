use std::convert::TryInto;
use string_pipeline::MultiTemplate;

#[test]
fn test_multi_template_literal_text_only() {
    // Test with only literal text, no template sections
    let template = MultiTemplate::parse("Just plain text without any templates", None).unwrap();
    let result = template.format("input_ignored").unwrap();
    assert_eq!(result, "Just plain text without any templates");
}

#[test]
fn test_multi_template_single_template_section() {
    // Test with just one template section and literal text
    let template = MultiTemplate::parse("Prefix {upper} Suffix", None).unwrap();
    let result = template.format("hello").unwrap();
    assert_eq!(result, "Prefix HELLO Suffix");
}

#[test]
fn test_multi_template_multiple_template_sections() {
    // Test multiple template sections with different operations
    let template = MultiTemplate::parse("A: {upper} B: {lower} C: {trim}", None).unwrap();
    let result = template.format("  MiXeD  ").unwrap();
    assert_eq!(result, "A:   MIXED   B:   mixed   C: MiXeD");
}

#[test]
fn test_multi_template_complex_operations() {
    // Test complex operations within multi-template sections
    let template = MultiTemplate::parse(
        "Users: {split:,:1..3|join:;} Count: {split:,:..|map:{upper}|join:-}",
        None,
    )
    .unwrap();
    let result = template.format("alice,bob,charlie,dave,eve").unwrap();
    assert_eq!(
        result,
        "Users: bob;charlie Count: ALICE-BOB-CHARLIE-DAVE-EVE"
    );
}

#[test]
fn test_multi_template_caching_optimization() {
    // Test that split operations are cached and reused efficiently
    let template = MultiTemplate::parse(
        "First: {split:,:0} Second: {split:,:1} Third: {split:,:0} Fourth: {split:,:2}",
        None,
    )
    .unwrap();
    let result = template.format("apple,banana,cherry,date").unwrap();
    assert_eq!(
        result,
        "First: apple Second: banana Third: apple Fourth: cherry"
    );
}

#[test]
fn test_multi_template_different_separators() {
    // Test multiple template sections with different separators
    let template = MultiTemplate::parse(
        "Comma: {split:,:0} Space: {split: :1} Pipe: {split:|:0}",
        None,
    )
    .unwrap();
    let result = template.format("a,b c|d").unwrap();
    assert_eq!(result, "Comma: a Space: c|d Pipe: a,b c");
}

#[test]
fn test_multi_template_nested_braces() {
    // Test that nested braces are handled correctly in multi-templates
    let template =
        MultiTemplate::parse("Result: {split:,:..|map:{upper|append:!}|join:-}", None).unwrap();
    let result = template.format("hello,world,test").unwrap();
    assert_eq!(result, "Result: HELLO!-WORLD!-TEST!");
}

#[test]
fn test_multi_template_empty_sections() {
    // Test handling of empty template sections and adjacent braces
    let template = MultiTemplate::parse("Start {upper} Middle {lower} End", None).unwrap();
    let result = template.format("test").unwrap();
    assert_eq!(result, "Start TEST Middle test End");
}

#[test]
fn test_multi_template_debug_mode() {
    // Test debug mode functionality in multi-templates
    let template = MultiTemplate::parse("Debug: {!upper} Normal: {lower}", None).unwrap();
    assert!(template.is_debug());
    let result = template.format("TeSt").unwrap();
    assert_eq!(result, "Debug: TEST Normal: test");
}

#[test]
fn test_multi_template_try_from_trait() {
    // Test TryFrom implementation
    let template: Result<MultiTemplate, _> = "Hello {upper} World!".try_into();
    assert!(template.is_ok());
    let template = template.unwrap();
    let result = template.format("test").unwrap();
    assert_eq!(result, "Hello TEST World!");
}

#[test]
fn test_multi_template_display_trait() {
    // Test Display implementation shows original template string
    let template = MultiTemplate::parse("Hello {upper} World!", None).unwrap();
    assert_eq!(format!("{}", template), "Hello {upper} World!");
}

#[test]
fn test_multi_template_template_string_method() {
    // Test template_string method returns original template
    let template_str = "Test {lower} Template";
    let template = MultiTemplate::parse(template_str, None).unwrap();
    assert_eq!(template.template_string(), template_str);
}

#[test]
fn test_multi_template_section_counts() {
    // Test section counting methods
    let template = MultiTemplate::parse("A {upper} B {lower} C", None).unwrap();
    assert_eq!(template.section_count(), 5); // "A ", upper, " B ", lower, " C"
    assert_eq!(template.template_section_count(), 2); // upper and lower operations
}

#[test]
fn test_multi_template_special_characters() {
    // Test multi-templates with special characters in literal text
    let template = MultiTemplate::parse("Price: $100 {upper} & more!", None).unwrap();
    let result = template.format("discount").unwrap();
    assert_eq!(result, "Price: $100 DISCOUNT & more!");
}

#[test]
fn test_multi_template_unicode_support() {
    // Test Unicode support in both literal text and operations
    let template = MultiTemplate::parse("🎉 Result: {upper} 🎊", None).unwrap();
    let result = template.format("café").unwrap();
    assert_eq!(result, "🎉 Result: CAFÉ 🎊");
}

#[test]
fn test_multi_template_whitespace_preservation() {
    // Test that whitespace in literal sections is preserved
    let template = MultiTemplate::parse("   Before   {trim}   After   ", None).unwrap();
    let result = template.format("  test  ").unwrap();
    assert_eq!(result, "   Before   test   After   ");
}

#[test]
fn test_multi_template_consecutive_templates() {
    // Test consecutive template sections without literal text between them
    let template = MultiTemplate::parse("{upper}{lower}", None).unwrap();
    let result = template.format("TeSt").unwrap();
    assert_eq!(result, "TESTtest");
}

#[test]
fn test_multi_template_shorthand_syntax() {
    // Test shorthand syntax within multi-templates
    let template = MultiTemplate::parse("First: {0} Last: {-1}", None).unwrap();
    let result = template.format("apple banana cherry").unwrap();
    assert_eq!(result, "First: apple Last: cherry");
}

#[test]
fn test_multi_template_range_operations() {
    // Test range operations in multi-templates
    let template = MultiTemplate::parse("Range: {1..3} Substring: {substring:2..5}", None).unwrap();
    let result = template.format("one two three four five").unwrap();
    assert_eq!(result, "Range: two three Substring: e t");
}

#[test]
fn test_multi_template_filter_operations() {
    // Test filter operations in multi-templates
    let template = MultiTemplate::parse(
        "Original: {split:,:..|join:,} Filtered: {split:,:..|filter:test|join:,}",
        None,
    )
    .unwrap();
    let result = template.format("apple,test1,banana,test2").unwrap();
    assert_eq!(
        result,
        "Original: apple,test1,banana,test2 Filtered: test1,test2"
    );
}

#[test]
fn test_multi_template_regex_operations() {
    // Test regex operations in multi-templates
    let template = MultiTemplate::parse(
        "Numbers: {regex_extract:\\d+} Letters: {regex_extract:[a-z]+}",
        None,
    )
    .unwrap();
    let result = template.format("abc123def456").unwrap();
    assert_eq!(result, "Numbers: 123 Letters: abc");
}

// Error handling tests

#[test]
fn test_multi_template_unclosed_brace_error() {
    // Test error when template section is not closed
    let result = MultiTemplate::parse("Hello {upper world", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unclosed template brace"));
}

#[test]
fn test_multi_template_invalid_operation_error() {
    // Test error when template section contains invalid operation
    let result = MultiTemplate::parse("Hello {invalid_operation} World", None);
    assert!(result.is_err());
}

#[test]
fn test_multi_template_malformed_nested_braces() {
    // Test error handling for malformed nested braces
    let result = MultiTemplate::parse("Test {split:,:0|map:{upper} incomplete", None);
    assert!(result.is_err());
}

#[test]
fn test_multi_template_format_error_propagation() {
    // Test that format errors from individual sections are properly propagated
    let template = MultiTemplate::parse("Valid: {upper} Invalid: {regex_extract:[}", None).unwrap();
    let result = template.format("test");
    assert!(result.is_err());
}

// Integration tests with complex scenarios

#[test]
fn test_multi_template_csv_processing() {
    // Test processing CSV-like data with multi-templates
    let template = MultiTemplate::parse(
        "Name: {split:,:0} | Age: {split:,:1} | Email: {split:,:2}",
        None,
    )
    .unwrap();
    let result = template.format("John Doe,25,john@example.com").unwrap();
    assert_eq!(result, "Name: John Doe | Age: 25 | Email: john@example.com");
}

#[test]
fn test_multi_template_log_processing() {
    // Test log processing scenario
    let template = MultiTemplate::parse(
        "[{split: :0}] Level: {split: :1} Message: {split: :2..}",
        None,
    )
    .unwrap();
    let result = template
        .format("2023-01-01 ERROR Database connection failed")
        .unwrap();
    assert_eq!(
        result,
        "[2023-01-01] Level: ERROR Message: Database connection failed"
    );
}

#[test]
fn test_multi_template_path_processing() {
    // Test file path processing
    let template =
        MultiTemplate::parse("Dir: {split:/:0..-1|join:/} File: {split:/:-1}", None).unwrap();
    let result = template.format("/home/user/documents/file.txt").unwrap();
    assert_eq!(result, "Dir: /home/user/documents File: file.txt");
}

#[test]
fn test_multi_template_complex_formatting() {
    // Test complex formatting scenario combining multiple operations
    let template = MultiTemplate::parse("Summary: {split:,:..|map:{trim|upper}|join:-} (Count: {split:,:..|map:{trim}|unique|join:,})", None).unwrap();
    let result = template
        .format("  apple  , banana ,  apple  , cherry ")
        .unwrap();
    assert_eq!(
        result,
        "Summary: APPLE-BANANA-APPLE-CHERRY (Count: apple,banana,cherry)"
    );
}

#[test]
fn test_multi_template_performance_with_many_sections() {
    // Test performance with many template sections (should use caching effectively)
    let sections = (0..10)
        .map(|i| format!("S{}: {{split:,:{}}}", i, i % 3))
        .collect::<Vec<_>>()
        .join(" ");

    let template = MultiTemplate::parse(&sections, None).unwrap();
    let result = template.format("a,b,c,d,e").unwrap();

    // Verify the template works and produces expected output
    assert!(result.contains("S0: a"));
    assert!(result.contains("S1: b"));
    assert!(result.contains("S2: c"));
    assert!(result.contains("S3: a")); // Should reuse cached split for index 0
}

#[test]
fn test_multi_template_empty_input() {
    // Test behavior with empty input
    let template = MultiTemplate::parse("Before: {upper} After: {lower}", None).unwrap();
    let result = template.format("").unwrap();
    assert_eq!(result, "Before:  After: ");
}

#[test]
fn test_multi_template_only_template_section() {
    // Test when entire string is just one template section
    let template = MultiTemplate::parse("{split:,:..|map:{upper}|join:-}", None).unwrap();
    let result = template.format("hello,world,test").unwrap();
    assert_eq!(result, "HELLO-WORLD-TEST");
}

#[test]
fn test_multi_template_mixed_operations() {
    // Test that split optimization doesn't interfere with other operations
    let template = MultiTemplate::parse("First: {0} Upper: {upper} Last: {2}", None).unwrap();
    let result = template.format("hello world test").unwrap();
    assert_eq!(result, "First: hello Upper: HELLO WORLD TEST Last: test");
}
