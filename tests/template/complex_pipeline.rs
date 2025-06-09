use super::process;

// Split + Join combinations
#[test]
fn test_split_join_different_separators() {
    assert_eq!(process("a,b,c", "{split:,:..|join:-}").unwrap(), "a-b-c");
}

#[test]
fn test_split_join_with_range() {
    assert_eq!(
        process("a,b,c,d,e", "{split:,:1..3|join:;}").unwrap(),
        "b;c"
    );
}

#[test]
fn test_split_join_empty_separator() {
    assert_eq!(process("a,b,c", "{split:,:..|join:}").unwrap(), "abc");
}

#[test]
fn test_split_append_with_index() {
    assert_eq!(
        process("a,b,c", "{split:,:1|append:_test}").unwrap(),
        "b_test"
    );
}

#[test]
fn test_split_index_transform_append() {
    assert_eq!(
        process("hello,world,test", "{split:,:1|upper|append:!}").unwrap(),
        "WORLD!"
    );
}

#[test]
fn test_multichar_separator_operations() {
    assert_eq!(
        process("a::b::c", r"{split:\:\::..|join:-}").unwrap(),
        "a-b-c"
    );
}

#[test]
fn test_escape_sequences_in_pipeline() {
    assert_eq!(
        process("a\tb\tc", "{split:\t:..|join:\n}").unwrap(),
        "a\nb\nc"
    );
}

#[test]
fn test_substring_split_join() {
    assert_eq!(
        process("prefix:a,b,c", "{substring:7..|split:,:..|join:-}").unwrap(),
        "a-b-c"
    );
}

#[test]
fn test_many_elements() {
    let input = (0..100)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let result = process(&input, "{split:,:0..5|map:{append:_num}|join:-}").unwrap();
    assert_eq!(result, "0_num-1_num-2_num-3_num-4_num");
}

#[test]
fn test_invalid_operation_in_pipeline() {
    assert!(process("test", "{split:,:..|unknown_op}").is_err());
}

#[test]
fn test_malformed_second_operation() {
    assert!(process("a,b,c", "{split:,:..|upper:invalid}").is_err());
}

#[test]
fn test_operation_on_empty_split() {
    assert_eq!(process("", "{split:,:..|map:{upper}}").unwrap(), "");
}

#[test]
fn test_invalid_range_in_pipeline() {
    assert!(process("a,b,c", "{split:,:abc|upper}").is_err());
}

#[test]
fn test_invalid_range_in_three_step() {
    assert!(process("a,b,c", "{split:,:abc|map:{upper}|join:-}").is_err());
}

// Join on a string that wasn't split should work (treat as single item)
#[test]
fn test_join_without_list() {
    assert_eq!(process("hello", "{join:-}").unwrap(), "hello");
}

#[test]
fn test_join_chaining_no_effect() {
    // Multiple joins should work - second join treats string as single item
    assert_eq!(
        process("a,b,c", "{split:,:..|join:-|join:_}").unwrap(),
        "a-b-c"
    );
}

#[test]
fn test_upper_join() {
    assert_eq!(
        process("hello world test", "{upper|split: :..|join:-}").unwrap(),
        "HELLO-WORLD-TEST"
    );
}

#[test]
fn test_replace_upper() {
    assert_eq!(
        process("hello world", "{replace:s/world/universe/|upper}").unwrap(),
        "HELLO UNIVERSE"
    );
}

#[test]
fn test_trim_split() {
    assert_eq!(process("  a,b,c  ", "{trim|split:,:..}").unwrap(), "a,b,c");
}

#[test]
fn test_replace_trim() {
    assert_eq!(
        process("  hello world  ", "{replace:s/world/universe/|trim}").unwrap(),
        "hello universe"
    );
}

#[test]
fn test_nested_transformations() {
    assert_eq!(
        process(
            "  HELLO,WORLD  ",
            "{trim|split:,:..|map:{lower|prepend:item_}}"
        )
        .unwrap(),
        "item_hello,item_world"
    );
}

#[test]
fn test_replace_split() {
    assert_eq!(
        process("hello-world-test", "{replace:s/-/,/g|split:,:..}").unwrap(),
        "hello,world,test"
    );
}

#[test]
fn test_substring_replace() {
    assert_eq!(
        process("hello world", "{substring:6..|replace:s/world/universe/}").unwrap(),
        "universe"
    );
}

#[test]
fn test_replace_split_join() {
    assert_eq!(
        process("hello-world-test", "{replace:s/-/,/g|split:,:..|join: }").unwrap(),
        "hello world test"
    );
}

#[test]
fn test_filter_with_replace() {
    let input = "test1,test2,prod1,prod2,dev1";

    // Filter then replace
    assert_eq!(
        process(
            input,
            "{split:,:..|filter:test|map:{replace:s/test/demo/g}|join:,}"
        )
        .unwrap(),
        "demo1,demo2"
    );

    // Replace then filter
    assert_eq!(
        process(
            input,
            "{split:,:..|map:{replace:s/\\d+//g}|filter:^test$|join:,}"
        )
        .unwrap(),
        "test,test"
    );
}

// Negative tests
#[test]
fn test_invalid_regex_in_pipeline() {
    assert!(process("test", "{split:,:..|map:{replace:s/[/invalid/|upper}}").is_err());
}

#[test]
fn test_substring_append_substring() {
    assert_eq!(
        process("hello", "{substring:1..4|append:_test|substring:0..5}").unwrap(),
        "ell_t"
    );
}

#[test]
fn test_prepend_append_prepend() {
    assert_eq!(
        process("test", "{prepend:[|append:]|prepend:>>}").unwrap(),
        ">>[test]"
    );
}

#[test]
fn test_filter_chaining() {
    let input = "Apple,banana,Cherry,grape,KIWI";

    // Filter then transform
    assert_eq!(
        process(input, r"{split:,:..|filter:^[A-Z]|map:{lower}|join:,}").unwrap(),
        "apple,cherry,kiwi"
    );

    // Transform then filter
    assert_eq!(
        process(input, r"{split:,:..|map:{lower}|filter:^[ag]|join:,}").unwrap(),
        "apple,grape"
    );

    // Multiple filters
    assert_eq!(
        process(input, r"{split:,:..|filter:^[A-Za-z]|filter:a|join:,}").unwrap(),
        "banana,grape"
    );
}

#[test]
fn test_filter_with_slicing() {
    let input = "apple,banana,cherry,date,elderberry";

    // Filter then slice
    assert_eq!(
        process(input, "{split:,:..|filter:e|slice:0..2|join:,}").unwrap(),
        "apple,cherry"
    );

    // slice then filter
    assert_eq!(
        process(input, "{split:,:..|slice:1..4|filter:a|join:,}").unwrap(),
        "banana,date"
    );
}

#[test]
fn test_filter_complex_chains() {
    let input = "  Apple  , banana ,  CHERRY  , grape,  KIWI  ";

    // Complex processing chain
    assert_eq!(
        process(
            input,
            r"{split:,:..|map:{trim}|filter:^[A-Z]|map:{lower|append:-fruit}|join: \| }"
        )
        .unwrap(),
        "apple-fruit | cherry-fruit | kiwi-fruit"
    );

    // Filter, sort-like operation with join
    let input2 = "zebra,apple,banana,cherry";
    assert_eq!(
        process(input2, "{split:,:..|filter:^[abc]|map:{upper}|join:-}").unwrap(),
        "APPLE-BANANA-CHERRY"
    );
}

#[test]
fn test_filter_log_processing() {
    let input = "INFO: Starting application,ERROR: Database connection failed,DEBUG: Query executed,WARNING: Deprecated function used,ERROR: Timeout occurred";

    // Error messages only
    assert_eq!(
        process(input, "{split:,:..|filter:^ERROR|join:\\n}").unwrap(),
        "ERROR: Database connection failed\nERROR: Timeout occurred"
    );

    // Non-debug messages
    assert_eq!(
        process(input, "{split:,:..|filter_not:^DEBUG|join:\\n}").unwrap(),
        "INFO: Starting application\nERROR: Database connection failed\nWARNING: Deprecated function used\nERROR: Timeout occurred"
    );
}

#[test]
fn test_filter_large_lists() {
    // Test with a larger dataset
    let large_input: Vec<String> = (0..1000).map(|i| format!("item{}", i)).collect();
    let input_str = large_input.join(",");

    // Filter even numbers
    let result = process(
        &input_str,
        r"{split:,:..|filter:[02468]$|slice:0..5|join:,}",
    )
    .unwrap();
    assert_eq!(result, "item0,item2,item4,item6,item8");
}

#[test]
fn test_complex_pipeline_with_new_ops() {
    assert_eq!(
        process("  c,a,b,a,c  ", "{trim|split:,:..|map:{trim}|unique|sort}").unwrap(),
        "a,b,c"
    );
}

#[test]
fn test_pipeline_with_map_and_pad() {
    assert_eq!(
        process("a,bb,c", "{split:,:..|map:{pad:3:*:both}}").unwrap(),
        "*a*,bb*,*c*"
    );
}

#[test]
fn test_regex_extract_with_map() {
    let input = "user123,admin456,guest789";
    assert_eq!(
        process(input, r"{split:,:..|map:{regex_extract:\d+}|join:-}").unwrap(),
        "123-456-789"
    );
}

#[test]
fn test_sort_reverse_combination() {
    assert_eq!(
        process("b,a,d,c", "{split:,:..|sort|reverse}").unwrap(),
        "d,c,b,a"
    );
}

#[test]
fn test_pad_trim_combination() {
    assert_eq!(
        process("  hello  ", "{trim|pad:20:*:both}").unwrap(),
        "*******hello********"
    );
}

#[test]
fn test_special_chars_pipeline() {
    assert_eq!(
        process("a\tb\tc", "{split:\t:..|map:{prepend:[|append:]}|join: }").unwrap(),
        "[a] [b] [c]"
    );
}

#[test]
fn test_escaped_pipes_pipeline() {
    let result = process("test", r"{replace:s/test/a|b/|split:|:..|join:-}");
    assert_eq!(result.unwrap(), "a-b");
}

#[test]
fn test_csv_processing() {
    assert_eq!(
        process("Name,Age,City", "{split:,:..|map:{lower|prepend:col_}}").unwrap(),
        "col_name,col_age,col_city"
    );
}

#[test]
fn test_path_processing() {
    assert_eq!(
        process(
            "/home/user/documents/file.txt",
            "{split:/:-1|split:.:..|map:{append:_backup}}"
        )
        .unwrap(),
        "file_backup.txt_backup"
    );
}

#[test]
fn test_log_processing() {
    assert_eq!(
        process("2023-01-01 ERROR Failed", "{split: :1..|join:_|lower}").unwrap(),
        "error_failed"
    );
}

#[test]
fn test_empty_string_three_steps() {
    assert_eq!(process("", "{trim|upper|append:test}").unwrap(), "test");
}

#[test]
fn test_single_char_pipeline() {
    assert_eq!(process("a", "{upper|append:!|prepend:->}").unwrap(), "->A!");
}

#[test]
fn test_empty_results_propagation() {
    assert_eq!(process("", "{split:,:..|map:{upper}|join:-}").unwrap(), "");
}

#[test]
fn test_invalid_middle_operation() {
    assert!(process("test", "{split:,:..|invalid_op|join:-}").is_err());
}

#[test]
fn test_invalid_final_operation() {
    assert!(process("test", "{split:,:..|map:{upper}|invalid_op}").is_err());
}

#[test]
fn test_malformed_three_step() {
    assert!(process("test", "{split:,|map:{upper}|}").is_err());
}

#[test]
fn test_missing_arguments_in_pipeline() {
    assert!(process("test", "{split|upper|join}").is_err());
}

#[test]
fn test_invalid_pipeline_syntax() {
    assert!(process("test", "{split:,||}").is_err());
}

#[test]
fn test_missing_pipe_separator() {
    // This should be treated as a single operation with malformed args
    assert!(process("test", "{split:, upper}").is_err());
}

#[test]
fn test_too_many_pipe_separators() {
    let result = process("test", "{split:,|||||||||upper}");
    assert!(result.is_err());
}

#[test]
fn test_map_split_basic() {
    // Test splitting each item in a list
    assert_eq!(
        process(
            "hello world,foo bar,test case",
            "{split:,:..|map:{split: :..}}"
        )
        .unwrap(),
        "hello world,foo bar,test case"
    );
}

#[test]
fn test_map_split_with_index() {
    // Extract first word from each line - simulating user extraction
    assert_eq!(
        process(
            "alice 123 firefox,bob 456 bash,charlie 789 vim",
            "{split:,:..|map:{split: :0}}"
        )
        .unwrap(),
        "alice,bob,charlie"
    );
}

#[test]
fn test_map_split_with_range() {
    // Extract multiple columns
    assert_eq!(
        process(
            "alice 123 firefox,bob 456 bash,charlie 789 vim",
            "{split:,:..|map:{split: :0..2}}"
        )
        .unwrap(),
        "alice 123,bob 456,charlie 789"
    );
}

#[test]
fn test_map_unique_after_split() {
    // This simulates extracting users and removing duplicates per line
    // Not the most practical example but tests the functionality
    assert_eq!(
        process("a a b,c c d,e e f", "{split:,:..|map:{split: :..|unique}}").unwrap(),
        "a b,c d,e f"
    );
}

#[test]
fn test_map_sort_after_split() {
    // Sort words in each item
    assert_eq!(
        process(
            "zebra apple,banana cherry",
            "{split:,:..|map:{split: :..|sort}}"
        )
        .unwrap(),
        "apple zebra,banana cherry"
    );
}

#[test]
fn test_map_filter_after_split() {
    // Filter words containing 'a' in each line
    assert_eq!(
        process(
            "apple banana cherry,dog cat fish,grape orange",
            "{split:,:..|map:{split: :..|filter:a}}"
        )
        .unwrap(),
        "apple banana,cat,grape orange"
    );
}

#[test]
fn test_map_slice_after_split() {
    // Take first 2 words from each line
    assert_eq!(
        process(
            "one two three four,five six seven eight",
            "{split:,:..|map:{split: :..|slice:0..2}}"
        )
        .unwrap(),
        "one two,five six"
    );
}

#[test]
fn test_map_join_with_different_separator() {
    // Split by space, then join with dash
    assert_eq!(
        process(
            "hello world,foo bar",
            "{split:,:..|map:{split: :..|join:-}}"
        )
        .unwrap(),
        "hello-world,foo-bar"
    );
}

#[test]
fn test_ps_aux_user_extraction() {
    // Simulate the ps aux use case
    let ps_output = "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\nroot           1  0.0  0.1  168404 11808 ?        Ss   Dec01   0:02 /sbin/init\nalice        123  0.1  0.5  256789 45123 ?        S    10:00   0:15 /usr/bin/firefox\nbob          456  0.0  0.2  123456 12345 ?        S    10:05   0:01 /bin/bash\nalice        789  0.2  1.0  512000 89012 ?        S    10:10   0:25 /usr/bin/chrome\ncharlie     1011  0.0  0.1   98765  6789 ?        S    10:15   0:02 /usr/bin/vim";

    // Extract users from each line (skip header) - normalize whitespace then split
    let result = process(
        ps_output,
        r"{split:\n:1..|map:{replace:s/ +/ /g|split: :0}|join:,}",
    )
    .unwrap();

    // Should extract the first word (user) from each line
    assert!(result.contains("root"));
    assert!(result.contains("alice"));
    assert!(result.contains("bob"));
    assert!(result.contains("charlie"));
}

#[test]
fn test_map_extract_with_list_level_unique_sort() {
    // This requires a different approach since we can't put unique outside map
    // Instead, we extract users and then apply unique and sort at the list level
    let ps_output = "USER         PID\nroot           1\nalice        123\nbob          456\nalice        789\ncharlie     1011\nbob         1213";

    // Extract users, then apply unique and sort at the list level
    // Use multiple spaces as separator and trim to handle whitespace
    let result = process(
        ps_output,
        r"{split:\n:1..|map:{replace:s/ +/ /g|split: :0}|unique|sort|join:,}",
    )
    .unwrap();

    // Should be sorted unique users
    assert_eq!(result, "alice,bob,charlie,root");
}

#[test]
fn test_map_extract_first_word_with_uppercase() {
    // Debug step by step
    let result1 = process(
        "hello world,foo bar,test case",
        "{split:,:..|map:{split: :0}}",
    );
    println!("Step 1: {:?}", result1);

    let result2 = process("hello", "{upper}");
    println!("Simple upper: {:?}", result2);

    // Complex pipeline: split by comma, then for each item split by space, take first word, uppercase
    let result = process(
        "hello world,foo bar,test case",
        "{split:,:..|map:{split: :0|upper}}",
    );
    println!("Full pipeline: {:?}", result);

    assert_eq!(result.unwrap(), "HELLO,FOO,TEST");
}

#[test]
fn test_map_filter_not_after_split() {
    // Filter out words containing 'a'
    assert_eq!(
        process(
            "apple banana cherry,dog cat fish",
            "{split:,:..|map:{split: :..|filter_not:a}}"
        )
        .unwrap(),
        "cherry,dog fish"
    );
}

#[test]
fn test_map_trim_then_extract_first_word() {
    // Handle whitespace issues in data
    assert_eq!(
        process(
            "  hello world  ,  foo bar  ",
            "{split:,:..|map:{trim|split: :0}}"
        )
        .unwrap(),
        "hello,foo"
    );
}

#[test]
fn test_map_complex_nested_pipeline() {
    // Complex nested operations: lowercase, split, slice first word, join with dash
    assert_eq!(
        process(
            "HELLO WORLD,FOO BAR",
            "{split:,:..|map:{lower|split: :..|slice:0..1|join:-}}"
        )
        .unwrap(),
        "hello,foo"
    );
}

#[test]
fn test_map_reverse_after_split() {
    // Reverse word order in each line
    assert_eq!(
        process(
            "one two three,four five six",
            "{split:,:..|map:{split: :..|reverse}}"
        )
        .unwrap(),
        "three two one,six five four"
    );
}

#[test]
fn test_map_list_operations_error_handling() {
    // Test error handling for invalid operations

    // Invalid regex should error
    assert!(process("test,data", "{split:,:..|map:{split: :..|filter:[}}").is_err());

    // Invalid range should error
    assert!(process("test,data", "{split:,:..|map:{split: :..|slice:abc}}").is_err());
}

#[test]
fn test_realistic_log_processing_with_map() {
    // Process log lines to extract specific information
    let logs = "2023-01-01 10:00:00 ERROR user alice failed login\n2023-01-01 10:01:00 INFO user bob successful login\n2023-01-01 10:02:00 ERROR user alice failed login\n2023-01-01 10:03:00 WARN user charlie timeout";

    // Extract users from error lines only
    let result = process(logs, r"{split:\n:..|filter:ERROR|map:{split: :4}|join:,}").unwrap();
    assert_eq!(result, "alice,alice");
}

#[test]
fn test_csv_column_extraction_with_map() {
    // Extract specific column from CSV-like data
    let csv = "name,age,city\nAlice,25,NYC\nBob,30,LA\nCharlie,35,SF";

    // Extract names (first column) - need to explicitly join with comma
    let result = process(csv, "{split:\n:1..|map:{split:,:0}|join:,}").unwrap();
    assert_eq!(result, "Alice,Bob,Charlie");

    // Extract cities (third column) - need to explicitly join with comma
    let result = process(csv, "{split:\n:1..|map:{split:,:2}|join:,}").unwrap();
    assert_eq!(result, "NYC,LA,SF");
}

#[test]
fn test_map_normalize_whitespace_extract_first_field() {
    // Test handling of various whitespace patterns
    let input = "  alice   123  ,  bob   456  ,  charlie   789  ";

    // Extract first field, handling extra whitespace - split by multiple spaces, take first word
    let result = process(
        input,
        r"{split:,:..|map:{trim|replace:s/ +/ /g|split: :0}|join:,}",
    )
    .unwrap();
    assert_eq!(result, "alice,bob,charlie");
}
