use super::process;

pub mod split_operations {
    use super::process;

    // Split operation tests
    #[test]
    fn test_split_basic() {
        assert_eq!(process("a,b,c,d", "{split:,:..}").unwrap(), "a,b,c,d");
    }

    #[test]
    fn test_split_with_index() {
        assert_eq!(process("a,b,c,d", "{split:,:1}").unwrap(), "b");
    }

    #[test]
    fn test_split_negative_index() {
        assert_eq!(process("a,b,c,d", "{split:,:-1}").unwrap(), "d");
    }

    #[test]
    fn test_split_range_exclusive() {
        assert_eq!(process("a,b,c,d", "{split:,:1..3}").unwrap(), "b,c");
    }

    #[test]
    fn test_split_range_inclusive() {
        assert_eq!(process("a,b,c,d", "{split:,:1..=3}").unwrap(), "b,c,d");
    }

    #[test]
    fn test_split_range_from() {
        assert_eq!(process("a,b,c,d", "{split:,:2..}").unwrap(), "c,d");
    }

    #[test]
    fn test_split_range_to() {
        assert_eq!(process("a,b,c,d", "{split:,:..2}").unwrap(), "a,b");
    }

    #[test]
    fn test_split_range_to_inclusive() {
        assert_eq!(process("a,b,c,d", "{split:,:..=2}").unwrap(), "a,b,c");
    }

    #[test]
    fn test_split_special_separator() {
        assert_eq!(
            process("a||b||c||d", r"{split:\|\|:..}").unwrap(),
            "a||b||c||d"
        );
    }

    #[test]
    fn test_split_newline_separator() {
        assert_eq!(
            process("a\nb\nc\nd", "{split:\\n:..}").unwrap(),
            "a\nb\nc\nd"
        );
    }

    #[test]
    fn test_split_tab_separator() {
        assert_eq!(
            process("a\tb\tc\td", r"{split:\t:..}").unwrap(),
            "a\tb\tc\td"
        );
    }

    #[test]
    fn test_split_empty_parts() {
        assert_eq!(process("a,,b,c", "{split:,:2}").unwrap(), "b");
    }

    #[test]
    fn test_split_single_item() {
        assert_eq!(process("single", "{split:,:..}").unwrap(), "single");
    }

    #[test]
    fn test_split_empty_string() {
        assert_eq!(process("", "{split:,:..}").unwrap(), "");
    }

    #[test]
    fn test_split_invalid_range() {
        assert!(process("a,b,c,d", "{split:,:abc}").is_err());
    }

    #[test]
    fn test_split_malformed_range() {
        assert!(process("a,b,c,d", "{split:,:1..abc}").is_err());
    }
}

pub mod join_operations {
    use super::process;

    // Join operation tests
    #[test]
    fn test_join_basic() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:-}").unwrap(),
            "a-b-c-d"
        );
    }

    #[test]
    fn test_join_empty_separator() {
        assert_eq!(process("a,b,c,d", "{split:,:..|join:}").unwrap(), "abcd");
    }

    #[test]
    fn test_join_newline() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:\\n}").unwrap(),
            "a\nb\nc\nd"
        );
    }

    #[test]
    fn test_join_special_chars() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:@@}").unwrap(),
            "a@@b@@c@@d"
        );
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:\\|\\|}").unwrap(),
            "a||b||c||d"
        );
    }

    #[test]
    fn test_join_unicode() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:🔥}").unwrap(),
            "a🔥b🔥c🔥d"
        );
    }

    #[test]
    fn test_join_single_item() {
        assert_eq!(process("single", "{split:,:..|join:-}").unwrap(), "single");
    }

    #[test]
    fn test_join_empty_list() {
        assert_eq!(process("", "{split:,:..|join:-}").unwrap(), "");
    }

    #[test]
    fn test_join_without_list() {
        assert_eq!(process("hello", "{join:-}").unwrap(), "hello");
    }

    #[test]
    fn test_join_chaining_no_effect_on_string() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|join:-|join:_}").unwrap(),
            "a-b-c-d"
        );
    }
}

pub mod replace_operations {
    use super::process;

    // Replace operation tests
    #[test]
    fn test_replace_basic() {
        assert_eq!(
            process("hello world", "{replace:s/world/universe/}").unwrap(),
            "hello universe"
        );
    }

    #[test]
    fn test_replace_global() {
        assert_eq!(
            process("foo foo foo", "{replace:s/foo/bar/g}").unwrap(),
            "bar bar bar"
        );
    }

    #[test]
    fn test_replace_case_insensitive() {
        assert_eq!(
            process("Hello HELLO hello", "{replace:s/hello/hi/gi}").unwrap(),
            "hi hi hi"
        );
    }

    #[test]
    fn test_replace_multiline() {
        assert_eq!(
            process("hello\nworld", "{replace:s/hello.world/hi universe/ms}").unwrap(),
            "hi universe"
        );
    }

    #[test]
    fn test_replace_digits() {
        assert_eq!(
            process("test123", "{replace:s/\\d+/456/}").unwrap(),
            "test456"
        );
    }

    #[test]
    fn test_replace_word_boundaries() {
        assert_eq!(
            process("cat caterpillar", "{replace:s/\\bcat\\b/dog/g}").unwrap(),
            "dog caterpillar"
        );
    }

    #[test]
    fn test_replace_capture_groups() {
        assert_eq!(
            process("hello world", "{replace:s/(\\w+) (\\w+)/$2 $1/}").unwrap(),
            "world hello"
        );
    }

    #[test]
    fn test_replace_empty_replacement() {
        assert_eq!(
            process("hello world", "{replace:s/world//}").unwrap(),
            "hello "
        );
    }

    #[test]
    fn test_replace_special_chars() {
        assert_eq!(
            process("a.b*c+d", "{replace:s/[.*+]/X/g}").unwrap(),
            "aXbXcXd"
        );
    }

    #[test]
    fn test_replace_no_match() {
        assert_eq!(
            process("hello world", "{replace:s/xyz/abc/}").unwrap(),
            "hello world"
        );
    }

    #[test]
    fn test_replace_invalid_sed_format() {
        assert!(process("test", "{replace:invalid}").is_err());
    }

    #[test]
    fn test_replace_empty_pattern() {
        assert!(process("test", "{replace:s//replacement/}").is_err());
    }

    #[test]
    fn test_replace_invalid_regex() {
        assert!(process("test", "{replace:s/[/replacement/}").is_err());
    }

    #[test]
    fn test_replace_missing_delimiter() {
        assert!(process("test", "{replace:s/pattern}").is_err());
    }

    #[test]
    fn test_replace_unicode_pattern() {
        assert_eq!(
            process("café naïve", "{replace:s/café/coffee/}").unwrap(),
            "coffee naïve"
        );
    }

    #[test]
    fn test_replace_flags_combination() {
        assert_eq!(
            process("Hello HELLO hello", "{replace:s/hello/hi/gi}").unwrap(),
            "hi hi hi"
        );
    }

    #[test]
    fn test_replace_backslash_escaping() {
        assert_eq!(
            process("test/path", r"{replace:s/\//_/g}").unwrap(),
            "test_path"
        );
    }

    #[test]
    fn test_replace_dollar_in_replacement() {
        assert_eq!(
            process("test", r"{replace:s/test/$&_suffix/}").unwrap(),
            "$&_suffix"
        );
    }

    #[test]
    fn test_replace_empty_input() {
        assert_eq!(process("", "{replace:s/anything/something/}").unwrap(), "");
    }

    #[test]
    fn test_replace_multiline_flag() {
        assert_eq!(
            process("line1\nline2", "{replace:s/^line/LINE/gm}").unwrap(),
            "LINE1\nLINE2"
        );
    }

    #[test]
    fn test_replace_dotall_flag() {
        assert_eq!(process("a\nb", "{replace:s/a.b/X/s}").unwrap(), "X");
    }
}

pub mod case_operations {
    use super::process;

    // Case operation tests
    #[test]
    fn test_upper_basic() {
        assert_eq!(process("hello world", "{upper}").unwrap(), "HELLO WORLD");
    }

    #[test]
    fn test_upper_unicode() {
        assert_eq!(process("café naïve", "{upper}").unwrap(), "CAFÉ NAÏVE");
    }

    #[test]
    fn test_lower_basic() {
        assert_eq!(process("HELLO WORLD", "{lower}").unwrap(), "hello world");
    }

    #[test]
    fn test_lower_unicode() {
        assert_eq!(process("CAFÉ NAÏVE", "{lower}").unwrap(), "café naïve");
    }
}

pub mod trim_operations {
    use super::process;

    // Trim operation tests
    #[test]
    fn test_trim_basic() {
        assert_eq!(process("  hello world  ", "{trim}").unwrap(), "hello world");
    }

    #[test]
    fn test_trim_tabs() {
        assert_eq!(process("\t\thello\t\t", "{trim}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_newlines() {
        assert_eq!(process("\n\nhello\n\n", "{trim}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_mixed_whitespace() {
        assert_eq!(process(" \t\n hello \t\n ", "{trim}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_no_whitespace() {
        assert_eq!(process("hello", "{trim}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_only_whitespace() {
        assert_eq!(process("   ", "{trim}").unwrap(), "");
    }

    #[test]
    fn test_trim_empty_string() {
        assert_eq!(process("", "{trim}").unwrap(), "");
    }

    #[test]
    fn test_trim_custom_chars_single_char() {
        assert_eq!(process("aaahelloaaa", "{trim:a}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_custom_chars_multiple_chars() {
        assert_eq!(process("xyzhellopqr", "{trim:xyzpqr}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_custom_chars_no_match() {
        assert_eq!(process("hello", "{trim:xyz}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_custom_chars_all_chars() {
        assert_eq!(process("aaaa", "{trim:a}").unwrap(), "");
    }

    #[test]
    fn test_trim_custom_chars_empty() {
        assert_eq!(process("hello", "{trim:}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_custom_chars_unicode() {
        assert_eq!(process("🔥hello🔥", "{trim:🔥}").unwrap(), "hello");
    }

    #[test]
    fn test_trim_custom_chars_left() {
        assert_eq!(process("xyhelloxy", "{trim:xy:left}").unwrap(), "helloxy");
    }

    #[test]
    fn test_trim_custom_chars_right() {
        assert_eq!(process("xyhelloxy", "{trim:xy:right}").unwrap(), "xyhello");
    }

    #[test]
    fn test_trim_custom_chars_both_explicit() {
        assert_eq!(process("xyhelloxy", "{trim:xy:both}").unwrap(), "hello");
    }
}

pub mod substring_operations {
    use super::process;

    // substring operation tests
    #[test]
    fn test_substring_index() {
        assert_eq!(process("hello", "{substring:1}").unwrap(), "e");
    }

    #[test]
    fn test_substring_negative_index() {
        assert_eq!(process("hello", "{substring:-1}").unwrap(), "o");
    }

    #[test]
    fn test_substring_range_exclusive() {
        assert_eq!(process("hello", "{substring:1..3}").unwrap(), "el");
    }

    #[test]
    fn test_substring_range_inclusive() {
        assert_eq!(process("hello", "{substring:1..=3}").unwrap(), "ell");
    }

    #[test]
    fn test_substring_range_from() {
        assert_eq!(process("hello", "{substring:2..}").unwrap(), "llo");
    }

    #[test]
    fn test_substring_range_to() {
        assert_eq!(process("hello", "{substring:..3}").unwrap(), "hel");
    }

    #[test]
    fn test_substring_range_to_inclusive() {
        assert_eq!(process("hello", "{substring:..=2}").unwrap(), "hel");
    }

    #[test]
    fn test_substring_full_range() {
        assert_eq!(process("hello", "{substring:..}").unwrap(), "hello");
    }

    #[test]
    fn test_substring_empty_string() {
        assert_eq!(process("", "{substring:0}").unwrap(), "");
    }

    #[test]
    fn test_substring_out_of_bounds() {
        assert_eq!(process("hi", "{substring:5}").unwrap(), "i");
    }

    #[test]
    fn test_substring_unicode() {
        assert_eq!(process("café", "{substring:1..3}").unwrap(), "af");
    }

    #[test]
    fn test_substring_invalid_range() {
        assert!(process("hello", "{substring:abc}").is_err());
    }

    #[test]
    fn test_substring_malformed_range() {
        assert!(process("hello", "{substring:1..abc}").is_err());
    }
}

pub mod append_operations {
    use super::process;

    // Append operation tests
    #[test]
    fn test_append_basic() {
        assert_eq!(process("hello", "{append:!}").unwrap(), "hello!");
    }

    #[test]
    fn test_append_multiple_chars() {
        assert_eq!(process("hello", "{append:_world}").unwrap(), "hello_world");
    }

    #[test]
    fn test_append_empty_string() {
        assert_eq!(process("", "{append:test}").unwrap(), "test");
    }

    #[test]
    fn test_append_unicode() {
        assert_eq!(process("hello", "{append:🔥}").unwrap(), "hello🔥");
    }

    #[test]
    fn test_append_special_chars() {
        assert_eq!(process("test", "{append:\\n}").unwrap(), "test\n");
    }

    #[test]
    fn test_append_escaped_colon() {
        assert_eq!(process("test", "{append:\\:value}").unwrap(), "test:value");
    }

    #[test]
    fn test_append_missing_argument() {
        let result = process("hello", "{append}");
        assert!(result.is_err());
    }
}

pub mod prepend_operations {
    use super::process;

    // Prepend operation tests
    #[test]
    fn test_prepend_basic() {
        assert_eq!(process("world", "{prepend:hello_}").unwrap(), "hello_world");
    }

    #[test]
    fn test_prepend_empty_string() {
        assert_eq!(process("", "{prepend:test}").unwrap(), "test");
    }

    #[test]
    fn test_prepend_unicode() {
        assert_eq!(process("world", "{prepend:🔥}").unwrap(), "🔥world");
    }

    #[test]
    fn test_prepend_special_chars() {
        assert_eq!(process("test", "{prepend:\\n}").unwrap(), "\ntest");
    }

    #[test]
    fn test_prepend_escaped_colon() {
        assert_eq!(process("test", "{prepend:value\\:}").unwrap(), "value:test");
    }

    #[test]
    fn test_prepend_missing_argument() {
        let result = process("hello", "{prepend}");
        assert!(result.is_err());
    }
}

pub mod surround_operations {
    use super::process;

    // Surround operation tests
    #[test]
    fn test_surround_basic() {
        assert_eq!(process("hello", "{surround:\"}").unwrap(), "\"hello\"");
    }

    #[test]
    fn test_surround_single_quotes() {
        assert_eq!(process("world", "{surround:'}").unwrap(), "'world'");
    }

    #[test]
    fn test_surround_multiple_chars() {
        assert_eq!(process("text", "{surround:**}").unwrap(), "**text**");
    }

    #[test]
    fn test_surround_empty_string() {
        assert_eq!(process("", "{surround:()}").unwrap(), "()()");
    }

    #[test]
    fn test_surround_unicode() {
        assert_eq!(process("hello", "{surround:🔥}").unwrap(), "🔥hello🔥");
    }

    #[test]
    fn test_surround_special_chars() {
        assert_eq!(process("test", "{surround:<<>>}").unwrap(), "<<>>test<<>>");
    }

    #[test]
    fn test_surround_escaped_chars() {
        assert_eq!(process("test", "{surround:\\n}").unwrap(), "\ntest\n");
    }

    #[test]
    fn test_surround_escaped_colon() {
        assert_eq!(process("test", "{surround:\\:}").unwrap(), ":test:");
    }

    #[test]
    fn test_surround_escaped_pipe() {
        assert_eq!(process("test", "{surround:\\|}").unwrap(), "|test|");
    }

    #[test]
    fn test_surround_empty_chars() {
        assert_eq!(process("hello", "{surround:}").unwrap(), "hello");
    }

    #[test]
    fn test_surround_missing_argument() {
        let result = process("hello", "{surround}");
        assert!(result.is_err());
    }

    #[test]
    fn test_surround_with_newlines() {
        assert_eq!(process("content", "{surround:--}").unwrap(), "--content--");
    }

    #[test]
    fn test_surround_complex_chars() {
        assert_eq!(
            process("data", "{surround:[[ ]]}").unwrap(),
            "[[ ]]data[[ ]]"
        );
    }
}

pub mod quote_operations {
    use super::process;

    // Quote operation tests (alias for surround)
    #[test]
    fn test_quote_basic() {
        assert_eq!(process("hello", "{quote:\"}").unwrap(), "\"hello\"");
    }

    #[test]
    fn test_quote_single_quotes() {
        assert_eq!(process("world", "{quote:'}").unwrap(), "'world'");
    }

    #[test]
    fn test_quote_multiple_chars() {
        assert_eq!(process("text", "{quote:**}").unwrap(), "**text**");
    }

    #[test]
    fn test_quote_empty_string() {
        assert_eq!(process("", "{quote:()}").unwrap(), "()()");
    }

    #[test]
    fn test_quote_unicode() {
        assert_eq!(process("hello", "{quote:🔥}").unwrap(), "🔥hello🔥");
    }

    #[test]
    fn test_quote_escaped_chars() {
        assert_eq!(process("test", "{quote:\\n}").unwrap(), "\ntest\n");
    }

    #[test]
    fn test_quote_escaped_colon() {
        assert_eq!(process("test", "{quote:\\:}").unwrap(), ":test:");
    }

    #[test]
    fn test_quote_empty_chars() {
        assert_eq!(process("hello", "{quote:}").unwrap(), "hello");
    }

    #[test]
    fn test_quote_missing_argument() {
        let result = process("hello", "{quote}");
        assert!(result.is_err());
    }

    #[test]
    fn test_quote_brackets() {
        assert_eq!(process("content", "{quote:[]}").unwrap(), "[]content[]");
    }
}

pub mod shorthand_operations {
    use super::process;

    // Shorthand syntax tests
    #[test]
    fn test_shorthand_index() {
        assert_eq!(process("a b c d", "{1}").unwrap(), "b");
    }

    #[test]
    fn test_shorthand_negative_index() {
        assert_eq!(process("a b c d", "{-1}").unwrap(), "d");
    }

    #[test]
    fn test_shorthand_range_exclusive() {
        assert_eq!(process("a b c d", "{1..3}").unwrap(), "b c");
    }

    #[test]
    fn test_shorthand_range_inclusive() {
        assert_eq!(process("a b c d", "{1..=3}").unwrap(), "b c d");
    }

    #[test]
    fn test_shorthand_range_from() {
        assert_eq!(process("a b c d", "{2..}").unwrap(), "c d");
    }

    #[test]
    fn test_shorthand_range_to() {
        assert_eq!(process("a b c d", "{..3}").unwrap(), "a b c");
    }

    #[test]
    fn test_shorthand_full_range() {
        assert_eq!(process("a b c d", "{..}").unwrap(), "a b c d");
    }

    #[test]
    fn test_shorthand_invalid_index() {
        assert!(process("a b c", "{abc}").is_err());
    }

    #[test]
    fn test_shorthand_invalid_range() {
        assert!(process("a b c", "{1..abc}").is_err());
    }
}

pub mod strip_ansi_operations {
    use super::process;

    // Strip Ansi operation tests
    #[test]
    fn test_strip_ansi_basic() {
        // Basic ANSI color codes
        let input = "\x1b[31mRed text\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Red text");

        // Multiple ANSI sequences
        let input = "\x1b[1m\x1b[31mBold Red\x1b[0m\x1b[32m Green\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Bold Red Green");

        // No ANSI sequences (should be unchanged)
        let input = "Plain text";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Plain text");
    }

    #[test]
    fn test_strip_ansi_complex_sequences() {
        // Complex ANSI sequences
        let input = "\x1b[38;5;196mHello\x1b[0m \x1b[48;5;21mWorld\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Hello World");

        // Cursor movement sequences
        let input = "\x1b[2J\x1b[H\x1b[31mCleared screen\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Cleared screen");

        // Mixed content
        let input = "Normal \x1b[1mBold\x1b[0m and \x1b[4mUnderlined\x1b[0m text";
        assert_eq!(
            process(input, "{strip_ansi}").unwrap(),
            "Normal Bold and Underlined text"
        );
    }

    #[test]
    fn test_strip_ansi_edge_cases() {
        // Empty string
        assert_eq!(process("", "{strip_ansi}").unwrap(), "");

        // Only ANSI sequences
        let input = "\x1b[31m\x1b[1m\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "");

        // Malformed ANSI sequences (should still work)
        let input = "\x1b[99mText\x1b[";
        let result = process(input, "{strip_ansi}").unwrap();
        assert!(result.contains("Text"));
    }

    #[test]
    fn test_strip_ansi_real_world_examples() {
        // Git colored output
        let input = "\x1b[32m+\x1b[0m\x1b[32madded line\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "+added line");

        // ls colored output
        let input = "\x1b[0m\x1b[01;34mfolder\x1b[0m  \x1b[01;32mexecutable\x1b[0m";
        assert_eq!(
            process(input, "{strip_ansi}").unwrap(),
            "folder  executable"
        );

        // Grep colored output
        let input = "file.txt:\x1b[01;31m\x1b[Kmatch\x1b[m\x1b[Ked text";
        assert_eq!(
            process(input, "{strip_ansi}").unwrap(),
            "file.txt:matched text"
        );
    }

    #[test]
    fn test_strip_ansi_unicode_preservation() {
        // Ensure Unicode characters are preserved
        let input = "\x1b[31m🚀 Rocket\x1b[0m \x1b[32m🌟 Star\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "🚀 Rocket 🌟 Star");

        // Unicode with combining characters
        let input = "\x1b[31mCafé naïve résumé\x1b[0m";
        assert_eq!(process(input, "{strip_ansi}").unwrap(), "Café naïve résumé");
    }
}

pub mod filter_operations {
    use super::process;

    // Filter operation tests
    #[test]
    fn test_filter_on_string_value() {
        // Filter on string - match keeps string
        assert_eq!(process("hello", "{filter:hello}").unwrap(), "hello");
        assert_eq!(process("hello", "{filter:^hello$}").unwrap(), "hello");
        assert_eq!(
            process("hello world", "{filter:world}").unwrap(),
            "hello world"
        );

        // Filter on string - no match returns empty
        assert_eq!(process("hello", "{filter:goodbye}").unwrap(), "");
        assert_eq!(process("hello", "{filter:^world$}").unwrap(), "");
    }

    #[test]
    fn test_filter_not_on_string_value() {
        // Filter not on string - match returns empty
        assert_eq!(process("hello", "{filter_not:hello}").unwrap(), "");
        assert_eq!(process("hello world", "{filter_not:world}").unwrap(), "");

        // Filter not on string - no match keeps string
        assert_eq!(process("hello", "{filter_not:goodbye}").unwrap(), "hello");
        assert_eq!(process("hello", "{filter_not:^world$}").unwrap(), "hello");
    }

    #[test]
    fn test_filter_empty_inputs() {
        // Empty string
        assert_eq!(process("", "{filter:anything}").unwrap(), "");
        assert_eq!(process("", "{filter_not:anything}").unwrap(), "");
    }

    #[test]
    fn test_filter_invalid_regex() {
        // Invalid regex patterns should return errors
        assert!(process("test", "{filter:[}").is_err());
        assert!(process("test", "{filter:(}").is_err());
        assert!(process("test", r"{filter:*}").is_err());
        assert!(process("test", r"{filter:+}").is_err());
        assert!(process("test", r"{filter:?}").is_err());

        // Same for filter_not
        assert!(process("test", "{filter_not:[}").is_err());
        assert!(process("test", "{filter_not:(}").is_err());
        assert!(process("test", r"{filter_not:*}").is_err());
        assert!(process("test", r"{filter_not:+}").is_err());
        assert!(process("test", r"{filter_not:?}").is_err());
    }

    #[test]
    fn test_filter_complex_regex() {
        assert_eq!(process("test123", r"{filter:\w+\d+}").unwrap(), "test123");
    }

    #[test]
    fn test_filter_unicode_pattern() {
        assert_eq!(process("café", r"{filter:caf[éè]}").unwrap(), "café");
    }

    #[test]
    fn test_filter_case_sensitive() {
        assert_eq!(process("Hello", "{filter:hello}").unwrap(), "");
        assert_eq!(process("Hello", "{filter:Hello}").unwrap(), "Hello");
    }

    #[test]
    fn test_filter_word_boundary() {
        assert_eq!(process("hello", r"{filter:\bhello\b}").unwrap(), "hello");
        assert_eq!(process("hellox", r"{filter:\bhello\b}").unwrap(), "");
    }

    #[test]
    fn test_filter_not_complex_pattern() {
        assert_eq!(process("file.txt", r"{filter_not:\.txt$}").unwrap(), "");
        assert_eq!(
            process("file.doc", r"{filter_not:\.txt$}").unwrap(),
            "file.doc"
        );
    }
}

pub mod sort_operations {
    use super::process;

    // Sort operation tests
    #[test]
    fn test_sort_asc() {
        assert_eq!(
            process("zebra,apple,banana", "{split:,:..|sort}").unwrap(),
            "apple,banana,zebra"
        );
    }

    #[test]
    fn test_sort_desc() {
        assert_eq!(
            process("zebra,apple,banana", "{split:,:..|sort:desc}").unwrap(),
            "zebra,banana,apple"
        );
    }

    #[test]
    fn test_sort_asc_explicit() {
        assert_eq!(process("c,a,b", "{split:,:..|sort:asc}").unwrap(), "a,b,c");
    }

    #[test]
    fn test_sort_on_string_error() {
        assert!(process("hello", "{sort}").is_err());
    }
}

pub mod reverse_operations {
    use super::process;

    // Reverse operation tests
    #[test]
    fn test_reverse_string() {
        assert_eq!(process("hello", "{reverse}").unwrap(), "olleh");
    }

    #[test]
    fn test_reverse_list() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|reverse}").unwrap(),
            "d,c,b,a"
        );
    }

    #[test]
    fn test_reverse_unicode_string() {
        assert_eq!(process("café", "{reverse}").unwrap(), "éfac");
    }
}

pub mod unique_operations {
    use super::process;

    // Unique operation tests
    #[test]
    fn test_unique_basic() {
        assert_eq!(
            process("a,b,a,c,b,d", "{split:,:..|unique}").unwrap(),
            "a,b,c,d"
        );
    }

    #[test]
    fn test_unique_empty_list() {
        assert_eq!(process("", "{split:,:..|unique}").unwrap(), "");
    }

    #[test]
    fn test_unique_no_duplicates() {
        assert_eq!(process("a,b,c", "{split:,:..|unique}").unwrap(), "a,b,c");
    }

    #[test]
    fn test_unique_on_string_error() {
        assert!(process("hello", "{unique}").is_err());
    }
}

pub mod pad_operations {
    use super::process;

    // Pad operation tests
    #[test]
    fn test_pad_right_default() {
        assert_eq!(process("hi", "{pad:5}").unwrap(), "hi   ");
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(process("hi", "{pad:5: :left}").unwrap(), "   hi");
    }

    #[test]
    fn test_pad_both() {
        assert_eq!(process("hi", "{pad:6: :both}").unwrap(), "  hi  ");
    }

    #[test]
    fn test_pad_custom_char() {
        assert_eq!(process("hi", "{pad:5:*:right}").unwrap(), "hi***");
    }

    #[test]
    fn test_pad_already_long_enough() {
        assert_eq!(process("hello", "{pad:3}").unwrap(), "hello");
    }

    #[test]
    fn test_pad_list_elements_via_map() {
        assert_eq!(
            process("a,bb,ccc", "{split:,:..|map:{pad:4:0:left}}").unwrap(),
            "000a,00bb,0ccc"
        );
    }

    #[test]
    fn test_pad_unicode() {
        assert_eq!(process("café", "{pad:6:*:both}").unwrap(), "*café*");
    }

    #[test]
    fn test_pad_zero_width() {
        assert_eq!(process("hello", "{pad:0}").unwrap(), "hello");
    }

    #[test]
    fn test_pad_exact_width() {
        assert_eq!(process("hello", "{pad:5}").unwrap(), "hello");
    }

    #[test]
    fn test_pad_multi_char_error() {
        assert_eq!(process("hi", "{pad:5:ab}").unwrap(), "hiaaa");
    }

    #[test]
    fn test_pad_empty_string() {
        assert_eq!(process("", "{pad:3}").unwrap(), "   ");
    }

    #[test]
    fn test_pad_unicode_char() {
        assert_eq!(process("hi", "{pad:5:🔥:left}").unwrap(), "🔥🔥🔥hi");
    }

    #[test]
    fn test_pad_both_odd_padding() {
        assert_eq!(process("hi", "{pad:7:*:both}").unwrap(), "**hi***");
    }

    #[test]
    fn test_pad_missing_width() {
        assert!(process("hello", "{pad}").is_err());
    }

    #[test]
    fn test_pad_invalid_width() {
        assert!(process("hello", "{pad:abc}").is_err());
    }

    #[test]
    fn test_pad_on_list_error() {
        assert!(process("a,b,c", "{split:,:..|pad:5}").is_err());
    }
}

pub mod slice_operations {
    use super::process;

    // Slice operation tests
    #[test]
    fn test_slice_basic() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:1..3}").unwrap(),
            "b,c"
        );
    }

    #[test]
    fn test_slice_with_single_index() {
        assert_eq!(process("a,b,c,d", "{split:,:..|slice:1}").unwrap(), "b");
    }

    #[test]
    fn test_slice_negative_index() {
        assert_eq!(process("a,b,c,d", "{split:,:..|slice:-1}").unwrap(), "d");
    }

    #[test]
    fn test_slice_range_inclusive() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:1..=3}").unwrap(),
            "b,c,d"
        );
    }

    #[test]
    fn test_slice_range_from() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:2..}").unwrap(),
            "c,d,e"
        );
    }

    #[test]
    fn test_slice_range_to() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:..3}").unwrap(),
            "a,b,c"
        );
    }

    #[test]
    fn test_slice_range_to_inclusive() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:..=2}").unwrap(),
            "a,b,c"
        );
    }

    #[test]
    fn test_slice_full_range() {
        assert_eq!(
            process("a,b,c,d", "{split:,:..|slice:..}").unwrap(),
            "a,b,c,d"
        );
    }

    #[test]
    fn test_slice_empty_list() {
        assert_eq!(process("", "{split:,:..|slice:0..2}").unwrap(), "");
    }

    #[test]
    fn test_slice_out_of_bounds() {
        assert_eq!(process("a,b", "{split:,:..|slice:5..10}").unwrap(), "");
    }

    #[test]
    fn test_slice_single_item_list() {
        assert_eq!(process("single", "{split:,:..|slice:0}").unwrap(), "single");
    }

    #[test]
    fn test_slice_negative_range() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:-3..-1}").unwrap(),
            "c,d"
        );
    }

    #[test]
    fn test_slice_mixed_indices() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:..|slice:-3..3}").unwrap(),
            "c"
        );
    }

    #[test]
    fn test_slice_on_string_error() {
        assert!(process("hello", "{slice:1..3}").is_err());
    }

    #[test]
    fn test_slice_invalid_range() {
        assert!(process("a,b,c", "{split:,:..|slice:abc}").is_err());
    }

    #[test]
    fn test_slice_malformed_range() {
        assert!(process("a,b,c", "{split:,:..|slice:1..abc}").is_err());
    }
}

pub mod regex_extract_operations {
    use super::process;

    // Regex extract operation tests
    #[test]
    fn test_regex_extract_basic() {
        assert_eq!(
            process("hello123world", r"{regex_extract:\d+}").unwrap(),
            "123"
        );
    }

    #[test]
    fn test_regex_extract_no_match() {
        assert_eq!(process("hello world", r"{regex_extract:\d+}").unwrap(), "");
    }

    #[test]
    fn test_regex_extract_group() {
        assert_eq!(
            process("email@domain.com", r"{regex_extract:(\w+)@(\w+):1}").unwrap(),
            "email"
        );
    }

    #[test]
    fn test_regex_extract_group_2() {
        assert_eq!(
            process("email@domain.com", r"{regex_extract:(\w+)@(\w+):2}").unwrap(),
            "domain"
        );
    }

    #[test]
    fn test_regex_extract_list_elements_via_map() {
        assert_eq!(
            process(
                "test123,abc456,xyz",
                r"{split:,:..|map:{regex_extract:\d+}}"
            )
            .unwrap(),
            "123,456,"
        );
    }

    #[test]
    fn test_regex_extract_invalid_regex() {
        assert!(process("test", r"{regex_extract:[}").is_err());
    }

    #[test]
    fn test_regex_extract_group_0_explicit() {
        assert_eq!(
            process("hello123world", r"{regex_extract:\d+:0}").unwrap(),
            "123"
        );
    }

    #[test]
    fn test_regex_extract_invalid_group() {
        assert_eq!(
            process("email@domain.com", r"{regex_extract:(\w+)@(\w+):10}").unwrap(),
            ""
        );
    }

    #[test]
    fn test_regex_extract_empty_capture_group() {
        assert_eq!(process("test", r"{regex_extract:(x?):1}").unwrap(), "");
    }

    #[test]
    fn test_regex_extract_unicode() {
        assert_eq!(
            process("café123naïve", r"{regex_extract:\d+}").unwrap(),
            "123"
        );
    }

    #[test]
    fn test_regex_extract_complex_pattern() {
        assert_eq!(
            process(
                "Version: 1.2.3-beta",
                r"{regex_extract:Version: (\d+\.\d+\.\d+):1}"
            )
            .unwrap(),
            "1.2.3"
        );
    }

    #[test]
    fn test_regex_extract_beginning_anchor() {
        assert_eq!(process("123hello", r"{regex_extract:^\d+}").unwrap(), "123");
    }

    #[test]
    fn test_regex_extract_end_anchor() {
        assert_eq!(process("hello123", r"{regex_extract:\d+$}").unwrap(), "123");
    }

    #[test]
    fn test_regex_extract_on_list_error() {
        assert!(process("a,b,c", r"{split:,:..|regex_extract:\d+}").is_err());
    }
}

pub mod general_negative_tests {
    use super::process;

    // Unknown operation tests
    #[test]
    fn test_unknown_operation() {
        assert!(process("test", "{unknown_op}").is_err());
    }

    #[test]
    fn test_malformed_template_braces() {
        assert!(process("test", "{split:,").is_err());
    }

    #[test]
    fn test_empty_template() {
        assert_eq!(process("test", "{}").unwrap(), "test");
    }
}
