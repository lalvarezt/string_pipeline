use string_pipeline::{MultiTemplate, SectionType};

#[test]
fn test_multi_template_literal_text_only() {
    // Test with only literal text, no template sections
    let template = MultiTemplate::parse("Just plain text without any templates").unwrap();
    let result = template.format("input_ignored").unwrap();
    assert_eq!(result, "Just plain text without any templates");
}

#[test]
fn test_multi_template_single_template_section() {
    // Test with just one template section and literal text
    let template = MultiTemplate::parse("Prefix {upper} Suffix").unwrap();
    let result = template.format("hello").unwrap();
    assert_eq!(result, "Prefix HELLO Suffix");
}

#[test]
fn test_multi_template_multiple_template_sections() {
    // Test multiple template sections with different operations
    let template = MultiTemplate::parse("A: {upper} B: {lower} C: {trim}").unwrap();
    let result = template.format("  MiXeD  ").unwrap();
    assert_eq!(result, "A:   MIXED   B:   mixed   C: MiXeD");
}

#[test]
fn test_multi_template_complex_operations() {
    // Test complex operations within multi-template sections
    let template =
        MultiTemplate::parse("Users: {split:,:1..3|join:;} Count: {split:,:..|map:{upper}|join:-}")
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
    let template =
        MultiTemplate::parse("Comma: {split:,:0} Space: {split: :1} Pipe: {split:\\|:0}").unwrap();
    let result = template.format("a,b c|d").unwrap();
    assert_eq!(result, "Comma: a Space: c|d Pipe: a,b c");
}

#[test]
fn test_multi_template_nested_braces() {
    // Test that nested braces are handled correctly in multi-templates
    let template =
        MultiTemplate::parse("Result: {split:,:..|map:{upper|append:!}|join:-}").unwrap();
    let result = template.format("hello,world,test").unwrap();
    assert_eq!(result, "Result: HELLO!-WORLD!-TEST!");
}

#[test]
fn test_multi_template_empty_sections() {
    // Test handling of empty template sections and adjacent braces
    let template = MultiTemplate::parse("Start {upper} Middle {lower} End").unwrap();
    let result = template.format("test").unwrap();
    assert_eq!(result, "Start TEST Middle test End");
}

#[test]
fn test_multi_template_debug_mode() {
    // Test debug mode functionality in multi-templates
    let template =
        MultiTemplate::parse_with_debug("Debug: {!upper} Normal: {lower}", Some(true)).unwrap();
    assert!(template.is_debug());
    let result = template.format("TeSt").unwrap();
    assert_eq!(result, "Debug: TEST Normal: test");
}

#[test]
fn test_multi_template_display_trait() {
    // Test Display implementation shows original template string
    let template = MultiTemplate::parse("Hello {upper} World!").unwrap();
    assert_eq!(format!("{template}"), "Hello {upper} World!");
}

#[test]
fn test_multi_template_template_string_method() {
    // Test template_string method returns original template
    let template_str = "Test {lower} Template";
    let template = MultiTemplate::parse(template_str).unwrap();
    assert_eq!(template.template_string(), template_str);
}

#[test]
fn test_multi_template_section_counts() {
    // Test section counting methods
    let template = MultiTemplate::parse("A {upper} B {lower} C").unwrap();
    assert_eq!(template.section_count(), 5); // "A ", upper, " B ", lower, " C"
    assert_eq!(template.template_section_count(), 2); // upper and lower operations
}

#[test]
fn test_multi_template_special_characters() {
    // Test multi-templates with special characters in literal text
    let template = MultiTemplate::parse("Price: $100 {upper} & more!").unwrap();
    let result = template.format("discount").unwrap();
    assert_eq!(result, "Price: $100 DISCOUNT & more!");
}

#[test]
fn test_multi_template_unicode_support() {
    // Test Unicode support in both literal text and operations
    let template = MultiTemplate::parse("🎉 Result: {upper} 🎊").unwrap();
    let result = template.format("café").unwrap();
    assert_eq!(result, "🎉 Result: CAFÉ 🎊");
}

#[test]
fn test_multi_template_whitespace_preservation() {
    // Test that whitespace in literal sections is preserved
    let template = MultiTemplate::parse("   Before   {trim}   After   ").unwrap();
    let result = template.format("  test  ").unwrap();
    assert_eq!(result, "   Before   test   After   ");
}

#[test]
fn test_multi_template_consecutive_templates() {
    // Test consecutive template sections without literal text between them
    let template = MultiTemplate::parse("{upper}{lower}").unwrap();
    let result = template.format("TeSt").unwrap();
    assert_eq!(result, "TESTtest");
}

#[test]
fn test_multi_template_shorthand_syntax() {
    // Test shorthand syntax within multi-templates
    let template = MultiTemplate::parse("First: {0} Last: {-1}").unwrap();
    let result = template.format("apple banana cherry").unwrap();
    assert_eq!(result, "First: apple Last: cherry");
}

#[test]
fn test_multi_template_range_operations() {
    // Test range operations in multi-templates
    let template = MultiTemplate::parse("Range: {1..3} Substring: {substring:2..5}").unwrap();
    let result = template.format("one two three four five").unwrap();
    assert_eq!(result, "Range: two three Substring: e t");
}

#[test]
fn test_multi_template_filter_operations() {
    // Test filter operations in multi-templates
    let template = MultiTemplate::parse(
        "Original: {split:,:..|join:,} Filtered: {split:,:..|filter:test|join:,}",
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
    let template =
        MultiTemplate::parse("Numbers: {regex_extract:\\d+} Letters: {regex_extract:[a-z]+}")
            .unwrap();
    let result = template.format("abc123def456").unwrap();
    assert_eq!(result, "Numbers: 123 Letters: abc");
}

// Error handling tests

#[test]
fn test_multi_template_unclosed_brace_error() {
    // Test error when template section is not closed
    let result = MultiTemplate::parse("Hello {upper world");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unclosed template brace"));
}

#[test]
fn test_multi_template_invalid_operation_error() {
    // Test error when template section contains invalid operation
    let result = MultiTemplate::parse("Hello {invalid_operation} World");
    assert!(result.is_err());
}

#[test]
fn test_multi_template_malformed_nested_braces() {
    // Test error handling for malformed nested braces
    let result = MultiTemplate::parse("Test {split:,:0|map:{upper} incomplete");
    assert!(result.is_err());
}

#[test]
fn test_multi_template_format_error_propagation() {
    // Test that format errors from individual sections are properly propagated
    let template = MultiTemplate::parse("Valid: {upper} Invalid: {regex_extract:[}").unwrap();
    let result = template.format("test");
    assert!(result.is_err());
}

// Integration tests with complex scenarios

#[test]
fn test_multi_template_csv_processing() {
    // Test processing CSV-like data with multi-templates
    let template =
        MultiTemplate::parse("Name: {split:,:0} | Age: {split:,:1} | Email: {split:,:2}").unwrap();
    let result = template.format("John Doe,25,john@example.com").unwrap();
    assert_eq!(result, "Name: John Doe | Age: 25 | Email: john@example.com");
}

#[test]
fn test_multi_template_log_processing() {
    // Test log processing scenario
    let template =
        MultiTemplate::parse("[{split: :0}] Level: {split: :1} Message: {split: :2..}").unwrap();
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
    let template = MultiTemplate::parse("Dir: {split:/:0..-1|join:/} File: {split:/:-1}").unwrap();
    let result = template.format("/home/user/documents/file.txt").unwrap();
    assert_eq!(result, "Dir: /home/user/documents File: file.txt");
}

#[test]
fn test_multi_template_complex_formatting() {
    // Test complex formatting scenario combining multiple operations
    let template = MultiTemplate::parse("Summary: {split:,:..|map:{trim|upper}|join:-} (Count: {split:,:..|map:{trim}|unique|join:,})").unwrap();
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

    let template = MultiTemplate::parse(&sections).unwrap();
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
    let template = MultiTemplate::parse("Before: {upper} After: {lower}").unwrap();
    let result = template.format("").unwrap();
    assert_eq!(result, "Before:  After: ");
}

#[test]
fn test_multi_template_only_template_section() {
    // Test when entire string is just one template section
    let template = MultiTemplate::parse("{split:,:..|map:{upper}|join:-}").unwrap();
    let result = template.format("hello,world,test").unwrap();
    assert_eq!(result, "HELLO-WORLD-TEST");
}

#[test]
fn test_multi_template_mixed_operations() {
    // Test that split optimization doesn't interfere with other operations
    let template = MultiTemplate::parse("First: {0} Upper: {upper} Last: {2}").unwrap();
    let result = template.format("hello world test").unwrap();
    assert_eq!(result, "First: hello Upper: HELLO WORLD TEST Last: test");
}

// Tests for the structured template functionality

#[test]
fn test_format_with_inputs_basic() {
    // Test basic usage of format_with_inputs (single inputs)
    let template = MultiTemplate::parse("User: {upper} | Email: {lower}").unwrap();
    let result = template
        .format_with_inputs(&[&["john doe"], &["JOHN@EXAMPLE.COM"]], &[" ", " "])
        .unwrap();
    assert_eq!(result, "User: JOHN DOE | Email: john@example.com");
}

#[test]
fn test_format_with_inputs_redirect() {
    // Test basic with multiple operations
    let template = MultiTemplate::parse("bat {strip_ansi|lower} > {}.txt").unwrap();
    let result = template
        .format_with_inputs(&[&["MyFile.log"], &["output"]], &[" ", " "])
        .unwrap();
    assert_eq!(result, "bat myfile.log > output.txt");
}

#[test]
fn test_format_with_inputs_multiple_values() {
    // Test multiple inputs per template section
    let template = MultiTemplate::parse("Users: {upper} | Files: {lower}").unwrap();
    let result = template
        .format_with_inputs(
            &[&["john doe", "peter parker"], &["FILE1.TXT", "FILE2.TXT"]],
            &[" ", ","],
        )
        .unwrap();
    assert_eq!(
        result,
        "Users: JOHN DOE PETER PARKER | Files: file1.txt,file2.txt"
    );
}

#[test]
fn test_format_with_inputs_multiple_values_quoted() {
    // Test multiple inputs per template section
    let template = MultiTemplate::parse("Users: {upper} | Files: {lower}").unwrap();
    let result = template
        .format_with_inputs(
            &[
                &["john doe", "peter parker"],
                &["'FILE1.TXT'", "'FILE2.TXT'"],
            ],
            &[" ", " "],
        )
        .unwrap();
    assert_eq!(
        result,
        "Users: JOHN DOE PETER PARKER | Files: 'file1.txt' 'file2.txt'"
    );
}

#[test]
fn test_format_with_inputs_complex_operations() {
    // Test with complex operations in each section
    let template = MultiTemplate::parse(
        "Files: {split:,:..|filter:\\.txt$|join: \\| } Count: {split:,:..|map:{upper}|join:-}",
    )
    .unwrap();
    let result = template
        .format_with_inputs(
            &[&["file1.txt,doc.pdf,file2.txt,readme.md"], &["a,b,c"]],
            &[" ", " "],
        )
        .unwrap();
    assert_eq!(result, "Files: file1.txt | file2.txt Count: A-B-C");
}

#[test]
fn test_format_with_inputs_single_template() {
    // Test with just one template section
    let template = MultiTemplate::parse("Result: {upper}").unwrap();
    let result = template
        .format_with_inputs(&[&["hello world"]], &[" "])
        .unwrap();
    assert_eq!(result, "Result: HELLO WORLD");
}

#[test]
fn test_format_with_inputs_no_templates() {
    // Test with no template sections (only literals)
    let template = MultiTemplate::parse("Just literal text").unwrap();
    let result = template.format_with_inputs(&[], &[]).unwrap();
    assert_eq!(result, "Just literal text");
}

#[test]
fn test_format_with_inputs_multiple_sections() {
    // Test with multiple template sections
    let template = MultiTemplate::parse("A: {upper} B: {lower} C: {trim} D: {append:!}").unwrap();
    let result = template
        .format_with_inputs(
            &[&["hello"], &["WORLD"], &["  test  "], &["done"]],
            &[" ", " ", " ", " "],
        )
        .unwrap();
    assert_eq!(result, "A: HELLO B: world C: test D: done!");
}

#[test]
fn test_format_with_inputs_input_count_handling() {
    // Test graceful handling when input count doesn't match template section count
    let template = MultiTemplate::parse("A: {upper} B: {lower}").unwrap();

    // Too few inputs - should use empty string for missing inputs
    let result = template.format_with_inputs(&[&["only_one"]], &[" ", " "]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: ONLY_ONE B: ");

    // Too many inputs - should truncate excess inputs
    let result = template.format_with_inputs(&[&["one"], &["two"], &["three"]], &[" ", " "]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: ONE B: two");
}

#[test]
fn test_format_with_inputs_excess_inputs() {
    // Test that excess inputs are truncated gracefully
    let template = MultiTemplate::parse("diff {} {}").unwrap();
    let result = template.format_with_inputs(
        &[
            &["file1.txt"],
            &["file2.txt"],
            &["file3.txt"], // This should be ignored
            &["file4.txt"], // This should also be ignored
        ],
        &[" ", " "],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "diff file1.txt file2.txt");
}

#[test]
fn test_format_with_inputs_insufficient_inputs() {
    // Test that missing inputs are treated as empty strings
    let template = MultiTemplate::parse("cmd {upper} {lower} {trim}").unwrap();
    let result = template.format_with_inputs(
        &[
            &["arg1"],
            &["ARG2"],
            // Missing third input
        ],
        &[" ", " ", " "],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cmd ARG1 arg2 ");
}

#[test]
fn test_format_with_inputs_empty_inputs_array() {
    // Test with completely empty inputs array
    let template = MultiTemplate::parse("start {upper} middle {lower} end").unwrap();
    let result = template.format_with_inputs(&[], &[" ", " "]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "start  middle  end");
}

#[test]
fn test_format_with_inputs_mixed_empty_and_filled() {
    // Test with mix of empty slices and filled slices
    let template = MultiTemplate::parse("A:{upper} B:{lower} C:{trim}").unwrap();
    let result = template.format_with_inputs(
        &[
            &[],        // Empty slice for first section
            &["hello"], // Normal input for second section
            &[],        // Empty slice for third section
        ],
        &[" ", " ", " "],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: B:hello C:");
}

#[test]
fn test_format_with_inputs_one_to_one_mode_scenario() {
    // Test the specific scenario from the original issue
    let template = MultiTemplate::parse("diff {} {}").unwrap();

    // Simulating OneToOne mode: individual slices for each input
    let inputs = ["file1.txt", "file2.txt", "file3.txt"];
    let input_arrays: Vec<&[&str]> = inputs.iter().map(std::slice::from_ref).collect();

    let result = template.format_with_inputs(&input_arrays, &[" ", " "]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "diff file1.txt file2.txt");
    // file3.txt should be ignored (truncated)
}

#[test]
fn test_format_with_inputs_separator_defaults() {
    // Test that missing separators default to space " "
    let template = MultiTemplate::parse("files: {} | items: {} | values: {}").unwrap();

    // Provide separators for only first section
    let result = template.format_with_inputs(
        &[
            &["a", "b", "c"], // First section gets comma separator
            &["x", "y", "z"], // Second section gets default space separator
            &["1", "2", "3"], // Third section gets default space separator
        ],
        &[","],
    );

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "files: a,b,c | items: x y z | values: 1 2 3"
    );
}

#[test]
fn test_format_with_inputs_no_separators_provided() {
    // Test with no separators provided - all should default to space
    let template = MultiTemplate::parse("A: {} B: {}").unwrap();

    let result = template.format_with_inputs(&[&["one", "two", "three"], &["a", "b", "c"]], &[]); // No separators provided

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: one two three B: a b c");
}

#[test]
fn test_format_with_inputs_separator_count_handling() {
    // Test graceful handling when separator count doesn't match template section count
    let template = MultiTemplate::parse("A: {upper} B: {lower}").unwrap();

    // Too few separators - should use default space for missing separators
    let result = template.format_with_inputs(&[&["one"], &["two"]], &[","]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: ONE B: two");

    // Too many separators - should truncate excess separators
    let result = template.format_with_inputs(&[&["one"], &["two"]], &[",", ";", ":"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A: ONE B: two");
}

#[test]
fn test_format_with_inputs_consecutive_templates() {
    // Test consecutive template sections without literal text
    let template = MultiTemplate::parse("{upper}{lower}{trim}").unwrap();
    let result = template
        .format_with_inputs(&[&["Hello"], &["WORLD"], &["  test  "]], &[" ", " ", " "])
        .unwrap();
    assert_eq!(result, "HELLOworldtest");
}

#[test]
fn test_format_with_inputs_empty_sections() {
    // Test empty input sections
    let template = MultiTemplate::parse("A: {upper} B: {lower}").unwrap();
    let result = template
        .format_with_inputs(&[&[], &["test"]], &[" ", " "])
        .unwrap();
    assert_eq!(result, "A:  B: test");
}

#[test]
fn test_format_with_inputs_custom_separators() {
    // Test different separators for each section
    let template = MultiTemplate::parse("List1: {} | List2: {}").unwrap();
    let result = template
        .format_with_inputs(&[&["a", "b", "c"], &["x", "y", "z"]], &["-", "|"])
        .unwrap();
    assert_eq!(result, "List1: a-b-c | List2: x|y|z");
}

#[test]
fn test_format_with_inputs_processing_error() {
    // Test that processing errors are properly propagated
    let template = MultiTemplate::parse("Valid: {upper} Invalid: {regex_extract:[}").unwrap();
    let result = template.format_with_inputs(&[&["test"], &["input"]], &[" ", " "]);
    assert!(result.is_err());
}

#[test]
fn test_get_template_sections() {
    // Test introspection method for template sections
    let template = MultiTemplate::parse("Hello {upper} world {lower|trim} end").unwrap();
    let sections = template.get_template_sections();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].0, 0); // First template section at position 0
    assert_eq!(sections[1].0, 1); // Second template section at position 1
    assert_eq!(sections[0].1.len(), 1); // {upper} has 1 operation
    assert_eq!(sections[1].1.len(), 2); // {lower|trim} has 2 operations
}

#[test]
fn test_get_template_sections_empty() {
    // Test with no template sections
    let template = MultiTemplate::parse("Just literal text with no templates").unwrap();
    let sections = template.get_template_sections();
    assert_eq!(sections.len(), 0);
}

#[test]
fn test_get_section_info() {
    // Test detailed section info method
    let template = MultiTemplate::parse("Start {upper} middle {lower} end").unwrap();
    let info = template.get_section_info();

    assert_eq!(info.len(), 5);

    // Check first section (literal)
    assert_eq!(info[0].section_type, SectionType::Literal);
    assert_eq!(info[0].overall_position, 0);
    assert_eq!(info[0].template_position, None);
    assert_eq!(info[0].content.as_ref().unwrap(), "Start ");
    assert!(info[0].operations.is_none());

    // Check second section (template)
    assert_eq!(info[1].section_type, SectionType::Template);
    assert_eq!(info[1].overall_position, 1);
    assert_eq!(info[1].template_position, Some(0));
    assert!(info[1].content.is_none());
    assert_eq!(info[1].operations.as_ref().unwrap().len(), 1);

    // Check third section (literal)
    assert_eq!(info[2].section_type, SectionType::Literal);
    assert_eq!(info[2].content.as_ref().unwrap(), " middle ");

    // Check fourth section (template)
    assert_eq!(info[3].section_type, SectionType::Template);
    assert_eq!(info[3].template_position, Some(1));

    // Check fifth section (literal)
    assert_eq!(info[4].section_type, SectionType::Literal);
    assert_eq!(info[4].content.as_ref().unwrap(), " end");
}

#[test]
fn test_get_section_info_only_templates() {
    // Test section info with only template sections
    let template = MultiTemplate::parse("{upper}{lower}").unwrap();
    let info = template.get_section_info();

    assert_eq!(info.len(), 2);
    assert_eq!(info[0].section_type, SectionType::Template);
    assert_eq!(info[0].template_position, Some(0));
    assert_eq!(info[1].section_type, SectionType::Template);
    assert_eq!(info[1].template_position, Some(1));
}

#[test]
fn test_backwards_compatibility_maintained() {
    // Test that existing format() method still works exactly as before
    let template = MultiTemplate::parse("Hello {upper} world {lower}!").unwrap();
    let result_old = template.format("test").unwrap();
    assert_eq!(result_old, "Hello TEST world test!");

    // Verify section counting methods work
    assert_eq!(template.template_section_count(), 2);
    assert_eq!(template.section_count(), 5);
}

#[test]
fn test_structured_template_complex_scenario() {
    // Test a complex real-world scenario
    let template =
        MultiTemplate::parse("cp {split:/:-1} /backup/{split:/:-1|replace:s/\\.txt$/.bak/}")
            .unwrap();
    let result = template
        .format_with_inputs(
            &[
                &[
                    "/home/user/documents/important1.txt",
                    "/home/user/documents/important2.txt",
                ],
                &["/home/user/documents/important.txt"],
            ],
            &[" ", " "],
        )
        .unwrap();
    assert_eq!(
        result,
        "cp important1.txt important2.txt /backup/important.bak"
    );
}

#[test]
fn test_structured_template_data_processing() {
    // Test structured processing for data transformation
    let template = MultiTemplate::parse("Name: {split:,:..|slice:0..1|join:} Age: {split:,:..|slice:1..2|join:} Email: {split:,:..|slice:2..3|join:}").unwrap();
    let csv_data = "John Doe,30,john@example.com";
    let result = template
        .format_with_inputs(&[&[csv_data], &[csv_data], &[csv_data]], &[" ", " ", " "])
        .unwrap();
    assert_eq!(result, "Name: John Doe Age: 30 Email: john@example.com");
}

#[test]
fn test_structured_template_file_operations() {
    // Test file operation template
    let template = MultiTemplate::parse("mkdir -p {split:/:..-1|join:/} && touch {}.tmp").unwrap();
    let result = template
        .format_with_inputs(
            &[
                &["/home/user/projects/new/file.txt"],
                &["/home/user/projects/new/file.txt"],
            ],
            &[" ", " "],
        )
        .unwrap();
    assert_eq!(
        result,
        "mkdir -p /home/user/projects/new && touch /home/user/projects/new/file.txt.tmp"
    );
}

// Tests for shell variable support (${...} patterns)

#[test]
fn test_multi_template_shell_variable_basic() {
    // Test basic shell variable pattern ${VAR}
    let template = MultiTemplate::parse("${HOME}/projects/{upper}").unwrap();
    let result = template.format("readme").unwrap();
    assert_eq!(result, "${HOME}/projects/README");
}

#[test]
fn test_multi_template_shell_variable_with_default() {
    // Test shell variable with default value ${VAR:-default}
    let template = MultiTemplate::parse("${EDITOR:-vim} {upper}.txt").unwrap();
    let result = template.format("config").unwrap();
    assert_eq!(result, "${EDITOR:-vim} CONFIG.txt");
}

#[test]
fn test_multi_template_shell_variable_specific_case() {
    // Test the specific case that was failing: ${EDITOR:-vim} {}
    let template = MultiTemplate::parse("${EDITOR:-vim} {}").unwrap();
    let result = template.format("file.txt").unwrap();
    assert_eq!(result, "${EDITOR:-vim} file.txt");
}

#[test]
fn test_multi_template_multiple_shell_variables() {
    // Test multiple shell variables in one template
    let template = MultiTemplate::parse("${USER}@${HOST}: {upper}").unwrap();
    let result = template.format("hello world").unwrap();
    assert_eq!(result, "${USER}@${HOST}: HELLO WORLD");
}

#[test]
fn test_multi_template_shell_variable_complex() {
    // Test complex shell variable expressions
    let template = MultiTemplate::parse("${PATH:+/usr/bin:}${HOME}/bin {lower}").unwrap();
    let result = template.format("SCRIPT").unwrap();
    assert_eq!(result, "${PATH:+/usr/bin:}${HOME}/bin script");
}

#[test]
fn test_multi_template_shell_variable_empty() {
    // Test empty shell variable ${}
    let template = MultiTemplate::parse("${} prefix {upper}").unwrap();
    let result = template.format("test").unwrap();
    assert_eq!(result, "${} prefix TEST");
}

#[test]
fn test_multi_template_shell_variable_nested_braces() {
    // Test shell variables with nested braces
    let template = MultiTemplate::parse("${CONFIG_DIR:-${HOME}/.config} {lower}").unwrap();
    let result = template.format("APP").unwrap();
    assert_eq!(result, "${CONFIG_DIR:-${HOME}/.config} app");
}

#[test]
fn test_multi_template_shell_variable_mixed_with_templates() {
    // Test mixing shell variables with multiple template sections
    let template = MultiTemplate::parse("cp {upper} ${BACKUP_DIR:-/backup}/{lower}.bak").unwrap();
    let result = template.format("important.txt").unwrap();
    assert_eq!(
        result,
        "cp IMPORTANT.TXT ${BACKUP_DIR:-/backup}/important.txt.bak"
    );
}

#[test]
fn test_multi_template_shell_variable_at_boundaries() {
    // Test shell variables at start/end of template
    let template = MultiTemplate::parse("${PREFIX} middle {upper} ${SUFFIX}").unwrap();
    let result = template.format("test").unwrap();
    assert_eq!(result, "${PREFIX} middle TEST ${SUFFIX}");
}

#[test]
fn test_multi_template_shell_variable_consecutive() {
    // Test consecutive shell variables
    let template = MultiTemplate::parse("${VAR1}${VAR2} {upper}").unwrap();
    let result = template.format("hello").unwrap();
    assert_eq!(result, "${VAR1}${VAR2} HELLO");
}

#[test]
fn test_multi_template_shell_variable_special_characters() {
    // Test shell variables with special characters
    let template = MultiTemplate::parse("${HOME}/some-dir/sub_dir {upper}").unwrap();
    let result = template.format("file name").unwrap();
    assert_eq!(result, "${HOME}/some-dir/sub_dir FILE NAME");
}

#[test]
fn test_multi_template_shell_variable_real_world_example() {
    // Test real-world shell command example
    let template = MultiTemplate::parse("${EDITOR:-nano} ${HOME}/.config/{lower}.conf").unwrap();
    let result = template.format("MYAPP").unwrap();
    assert_eq!(result, "${EDITOR:-nano} ${HOME}/.config/myapp.conf");
}

// Error handling tests for shell variables

#[test]
fn test_multi_template_unclosed_shell_variable_error() {
    // Test error when shell variable is not closed
    let result = MultiTemplate::parse("${HOME unclosed {upper}");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Unclosed shell variable brace")
    );
}

#[test]
fn test_multi_template_shell_variable_complex_nesting() {
    // Test complex nesting of shell variables and templates
    let template = MultiTemplate::parse(
        "${DIR:-${HOME}/default} contains {split:,:..|filter:\\.txt$|join: and }",
    )
    .unwrap();
    let result = template
        .format("file1.txt,doc.pdf,file2.txt,readme.md")
        .unwrap();
    assert_eq!(
        result,
        "${DIR:-${HOME}/default} contains file1.txt and file2.txt"
    );
}
