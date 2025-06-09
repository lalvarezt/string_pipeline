use super::process;

pub mod individual_operations {

    pub mod basic_operations {
        use super::super::process;

        #[test]
        fn test_map_upper() {
            assert_eq!(
                process("apple,banana,cherry", "{split:,:..|map:{upper}}").unwrap(),
                "APPLE,BANANA,CHERRY"
            );
        }

        #[test]
        fn test_map_lower() {
            assert_eq!(
                process("APPLE,BANANA,CHERRY", "{split:,:..|map:{lower}}").unwrap(),
                "apple,banana,cherry"
            );
        }

        #[test]
        fn test_map_trim_default() {
            assert_eq!(
                process("  apple  ,  banana  ,  cherry  ", "{split:,:..|map:{trim}}").unwrap(),
                "apple,banana,cherry"
            );
        }

        #[test]
        fn test_map_trim_both() {
            assert_eq!(
                process(
                    "  apple  ,  banana  ,  cherry  ",
                    "{split:,:..|map:{trim:both}}"
                )
                .unwrap(),
                "apple,banana,cherry"
            );
        }

        #[test]
        fn test_map_trim_left() {
            assert_eq!(
                process("  apple  ,  banana  ", "{split:,:..|map:{trim:left}}").unwrap(),
                "apple  ,banana  "
            );
        }

        #[test]
        fn test_map_trim_right() {
            assert_eq!(
                process("  apple  ,  banana  ", "{split:,:..|map:{trim:right}}").unwrap(),
                "  apple,  banana"
            );
        }

        #[test]
        fn test_map_strip_ansi() {
            let input = "\x1b[31mred\x1b[0m,\x1b[32mgreen\x1b[0m,\x1b[34mblue\x1b[0m";
            assert_eq!(
                process(input, "{split:,:..|map:{strip_ansi}}").unwrap(),
                "red,green,blue"
            );
        }
    }

    pub mod string_operations {
        use super::super::process;

        #[test]
        fn test_map_append_basic() {
            assert_eq!(
                process("apple,banana,cherry", "{split:,:..|map:{append:!}}").unwrap(),
                "apple!,banana!,cherry!"
            );
        }

        #[test]
        fn test_map_prepend_basic() {
            assert_eq!(
                process("apple,banana,cherry", "{split:,:..|map:{prepend:*}}").unwrap(),
                "*apple,*banana,*cherry"
            );
        }

        #[test]
        fn test_map_trim_custom_chars_basic() {
            assert_eq!(
                process("xappleX,xbananaX,xcherryX", "{split:,:..|map:{trim:xX}}").unwrap(),
                "apple,banana,cherry"
            );
        }

        #[test]
        fn test_map_pad_default() {
            assert_eq!(
                process("a,bb,ccc", "{split:,:..|map:{pad:5}}").unwrap(),
                "a    ,bb   ,ccc  "
            );
        }

        #[test]
        fn test_map_pad_left() {
            assert_eq!(
                process("a,bb,ccc", "{split:,:..|map:{pad:5: :left}}").unwrap(),
                "    a,   bb,  ccc"
            );
        }

        #[test]
        fn test_map_pad_both() {
            assert_eq!(
                process("a,bb", "{split:,:..|map:{pad:6: :both}}").unwrap(),
                "  a   ,  bb  "
            );
        }

        #[test]
        fn test_map_pad_custom_char() {
            assert_eq!(
                process("a,bb", "{split:,:..|map:{pad:4:0:left}}").unwrap(),
                "000a,00bb"
            );
        }
    }

    pub mod substring_operations {
        use super::super::process;

        #[test]
        fn test_map_substring_index() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:0}}").unwrap(),
                "h,w,t"
            );
        }

        #[test]
        fn test_map_substring_negative_index() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:-1}}").unwrap(),
                "o,d,g"
            );
        }

        #[test]
        fn test_map_substring_range_exclusive() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:0..3}}").unwrap(),
                "hel,wor,tes"
            );
        }

        #[test]
        fn test_map_substring_range_inclusive() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:0..=2}}").unwrap(),
                "hel,wor,tes"
            );
        }

        #[test]
        fn test_map_substring_range_from() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:2..}}").unwrap(),
                "llo,rld,sting"
            );
        }

        #[test]
        fn test_map_substring_range_to() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:..3}}").unwrap(),
                "hel,wor,tes"
            );
        }

        #[test]
        fn test_map_substring_range_to_inclusive() {
            assert_eq!(
                process("hello,world,testing", "{split:,:..|map:{substring:..=2}}").unwrap(),
                "hel,wor,tes"
            );
        }
    }

    pub mod replace_operations {
        use super::super::process;

        #[test]
        fn test_map_replace_basic() {
            assert_eq!(
                process("hello,world,hell", "{split:,:..|map:{replace:s/l/L/}}").unwrap(),
                "heLlo,worLd,heLl"
            );
        }

        #[test]
        fn test_map_replace_global() {
            assert_eq!(
                process("hello,world,hell", "{split:,:..|map:{replace:s/l/L/g}}").unwrap(),
                "heLLo,worLd,heLL"
            );
        }

        #[test]
        fn test_map_replace_case_insensitive() {
            assert_eq!(
                process("Hello,WORLD,heLLo", "{split:,:..|map:{replace:s/l/X/gi}}").unwrap(),
                "HeXXo,WORXD,heXXo"
            );
        }

        #[test]
        fn test_map_replace_digits() {
            assert_eq!(
                process(
                    "test123,abc456,xyz789",
                    r"{split:,:..|map:{replace:s/\d+/NUM/g}}"
                )
                .unwrap(),
                "testNUM,abcNUM,xyzNUM"
            );
        }
    }

    pub mod regex_extract_operations {
        use super::super::process;

        #[test]
        fn test_map_regex_extract_basic() {
            assert_eq!(
                process(
                    "test123,abc456,xyz789",
                    r"{split:,:..|map:{regex_extract:\d+}}"
                )
                .unwrap(),
                "123,456,789"
            );
        }

        #[test]
        fn test_map_regex_extract_group() {
            assert_eq!(
                process(
                    "user:alice,user:bob,user:charlie",
                    r"{split:,:..|map:{regex_extract:user\:(\w+):1}}"
                )
                .unwrap(),
                "alice,bob,charlie"
            );
        }

        #[test]
        fn test_map_regex_extract_no_match() {
            assert_eq!(
                process("abc,def,ghi", r"{split:,:..|map:{regex_extract:\d+}}").unwrap(),
                ",,"
            );
        }

        #[test]
        fn test_map_regex_extract_letters() {
            assert_eq!(
                process(
                    "123abc456,789def012,345ghi678",
                    r"{split:,:..|map:{regex_extract:[a-z]+}}"
                )
                .unwrap(),
                "abc,def,ghi"
            );
        }

        #[test]
        fn test_map_regex_extract_date_pattern_workaround() {
            // Note: Due to parser limitations, curly brace quantifiers in regex patterns
            // within map operations need to be written as repeated patterns instead
            // Use \d\d\d\d-\d\d-\d\d instead of \d{4}-\d{2}-\d{2}
            assert_eq!(
                process(
                    "2023-01-01 ERROR Failed,2023-12-25 INFO Success",
                    r"{split:,:..|map:{regex_extract:\d\d\d\d-\d\d-\d\d}}"
                )
                .unwrap(),
                "2023-01-01,2023-12-25"
            );
        }

        #[test]
        fn test_map_regex_extract_character_class_alternative() {
            // Alternative approach using character classes
            assert_eq!(
                process(
                    "2023-01-01 ERROR Failed,2023-12-25 INFO Success",
                    r"{split:,:..|map:{regex_extract:[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]}}"
                )
                .unwrap(),
                "2023-01-01,2023-12-25"
            );
        }
    }
}

pub mod escaped_characters {
    use super::process;

    #[test]
    fn test_map_append_escaped_colon() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{append:\:}}").unwrap(),
            "apple:,banana:"
        );
    }

    #[test]
    fn test_map_prepend_escaped_colon() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{prepend:\:}}").unwrap(),
            ":apple,:banana"
        );
    }

    #[test]
    fn test_map_append_escaped_pipe() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{append:\|}}").unwrap(),
            "apple|,banana|"
        );
    }

    #[test]
    fn test_map_prepend_escaped_pipe() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{prepend:\|}}").unwrap(),
            "|apple,|banana"
        );
    }

    #[test]
    fn test_map_append_escaped_backslash() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{append:\\}}").unwrap(),
            r"apple\,banana\"
        );
    }

    #[test]
    fn test_map_append_escaped_newline() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{append:\n}}").unwrap(),
            "apple\n,banana\n"
        );
    }

    #[test]
    fn test_map_append_escaped_tab() {
        assert_eq!(
            process("apple,banana", r"{split:,:..|map:{append:\t}}").unwrap(),
            "apple\t,banana\t"
        );
    }

    #[test]
    fn test_map_trim_escaped_chars() {
        assert_eq!(
            process(":apple:,|banana|", r"{split:,:..|map:{trim:\:\|}}").unwrap(),
            "apple,banana"
        );
    }

    #[test]
    fn test_map_replace_escaped_pattern() {
        assert_eq!(
            process("a:b,c:d,e:f", r"{split:,:..|map:{replace:s/\:/=/g}}").unwrap(),
            "a=b,c=d,e=f"
        );
    }

    #[test]
    fn test_map_pad_escaped_char() {
        assert_eq!(
            process("a,b", r"{split:,:..|map:{pad:3:\::right}}").unwrap(),
            "a::,b::"
        );
    }

    #[test]
    fn test_map_regex_extract_escaped_pattern() {
        assert_eq!(
            process("a:1,b:2,c:3", r"{split:,:..|map:{regex_extract:\w\:\d}}").unwrap(),
            "a:1,b:2,c:3"
        );
    }
}

pub mod pipeline_operations {
    use super::process;

    #[test]
    fn test_map_pipeline_two_steps() {
        assert_eq!(
            process("  hello  ,  world  ", "{split:,:..|map:{trim|upper}}").unwrap(),
            "HELLO,WORLD"
        );
    }

    #[test]
    fn test_map_pipeline_three_steps() {
        assert_eq!(
            process(
                "  hello  ,  world  ",
                "{split:,:..|map:{trim|upper|append:!}}"
            )
            .unwrap(),
            "HELLO!,WORLD!"
        );
    }

    #[test]
    fn test_map_pipeline_complex() {
        assert_eq!(
            process(
                "  abc123  ,  def456  ",
                r"{split:,:..|map:{trim|regex_extract:\d+|append:_num}}"
            )
            .unwrap(),
            "123_num,456_num"
        );
    }

    #[test]
    fn test_map_pipeline_substring_upper_append() {
        assert_eq!(
            process(
                "hello,world,testing",
                "{split:,:..|map:{substring:1..4|upper|append:!}}"
            )
            .unwrap(),
            "ELL!,ORL!,EST!"
        );
    }

    #[test]
    fn test_map_pipeline_prepend_replace_trim() {
        assert_eq!(
            process(
                "abc,def,ghi",
                r"{split:,:..|map:{prepend: X |replace:s/X/Y/g|trim}}"
            )
            .unwrap(),
            "Y abc,Y def,Y ghi"
        );
    }

    #[test]
    fn test_map_pipeline_pad_trim() {
        assert_eq!(
            process("a,bb,ccc", "{split:,:..|map:{pad:5:*:both|trim:*}}").unwrap(),
            "a,bb,ccc"
        );
    }

    #[test]
    fn test_map_pipeline_with_escapes() {
        assert_eq!(
            process(
                "hello,world",
                r"{split:,:..|map:{append:\:|upper|prepend:[|append:]}}"
            )
            .unwrap(),
            r"[HELLO:],[WORLD:]"
        );
    }
}

pub mod invalid_operations {
    use super::process;

    #[test]
    fn test_map_invalid_split() {
        assert!(process("a,b,c", "{split:,:..|map:{split::}}").is_err());
    }

    #[test]
    fn test_map_invalid_sort() {
        assert!(process("a,b,c", "{split:,:..|map:{sort}}").is_err());
    }

    #[test]
    fn test_map_invalid_unique() {
        assert!(process("a,b,c", "{split:,:..|map:{unique}}").is_err());
    }

    #[test]
    fn test_map_invalid_slice() {
        assert!(process("a,b,c", "{split:,:..|map:{slice:1..3}}").is_err());
    }

    #[test]
    fn test_map_invalid_nested_map() {
        // Nested map operations should fail
        assert!(process("a,b,c", "{split:,:..|map:{map:{upper}}}").is_err());
    }

    #[test]
    fn test_map_unknown_operation() {
        assert!(process("a,b,c", "{split:,:..|map:{unknown_op}}").is_err());
    }

    #[test]
    fn test_map_invalid_operation_in_pipeline() {
        assert!(process("a,b,c", "{split:,:..|map:{upper|split::}}").is_err());
    }

    #[test]
    fn test_map_malformed_braces() {
        assert!(process("a,b,c", "{split:,:..|map:upper}").is_err());
    }

    #[test]
    fn test_map_empty_operation() {
        assert!(process("a,b,c", "{split:,:..|map:{}}").is_err());
    }

    #[test]
    fn test_map_missing_colon() {
        assert!(process("a,b,c", "{split:,:..|map{upper}}").is_err());
    }

    #[test]
    fn test_map_invalid_regex_in_pipeline() {
        assert!(process("a,b,c", r"{split:,:..|map:{regex_extract:[|upper}}").is_err());
    }
}

pub mod edge_cases {
    use super::process;

    #[test]
    fn test_map_empty_string() {
        assert_eq!(process("", "{split:,:..|map:{upper}}").unwrap(), "");
    }

    #[test]
    fn test_map_single_item() {
        assert_eq!(
            process("hello", "{split:,:..|map:{upper}}").unwrap(),
            "HELLO"
        );
    }

    #[test]
    fn test_map_empty_items() {
        assert_eq!(
            process("a,,c", "{split:,:..|map:{append:!}}").unwrap(),
            "a!,!,c!"
        );
    }

    #[test]
    fn test_map_unicode() {
        assert_eq!(
            process("café,naïve,résumé", "{split:,:..|map:{upper}}").unwrap(),
            "CAFÉ,NAÏVE,RÉSUMÉ"
        );
    }

    #[test]
    fn test_map_special_characters() {
        assert_eq!(
            process("@hello,#world,$test", "{split:,:..|map:{prepend:>}}").unwrap(),
            ">@hello,>#world,>$test"
        );
    }

    #[test]
    fn test_map_very_long_pipeline() {
        assert_eq!(
            process(
                "abc,def",
                "{split:,:..|map:{upper|append:1|prepend:2|substring:1..5|lower}}"
            )
            .unwrap(),
            "abc1,def1"
        );
    }

    #[test]
    fn test_map_with_whitespace() {
        assert_eq!(
            process(
                "hello world,foo bar",
                "{split:,:..|map:{replace:s/ /_/g|upper}}"
            )
            .unwrap(),
            "HELLO_WORLD,FOO_BAR"
        );
    }

    #[test]
    fn test_map_multiple_maps() {
        assert_eq!(
            process("hello,world", "{split:,:..|map:{upper}|map:{append:!}}").unwrap(),
            "HELLO!,WORLD!"
        );
    }
}

pub mod template_variations {
    use super::process;

    #[test]
    fn test_template_with_different_separators() {
        assert_eq!(
            process("a|b|c", r"{split:\|:..|map:{upper}|join:,}").unwrap(),
            "A,B,C"
        );
    }

    #[test]
    fn test_template_with_range_and_map() {
        assert_eq!(
            process("a,b,c,d,e", "{split:,:1..3|map:{upper}}").unwrap(),
            "B,C"
        );
    }

    #[test]
    fn test_template_with_newline_separator() {
        assert_eq!(
            process("hello\nworld\ntest", r"{split:\n:..|map:{upper}|join:,}").unwrap(),
            "HELLO,WORLD,TEST"
        );
    }

    #[test]
    fn test_template_with_tab_separator() {
        assert_eq!(
            process("hello\tworld\ttest", r"{split:\t:..|map:{upper}|join:,}").unwrap(),
            "HELLO,WORLD,TEST"
        );
    }

    #[test]
    fn test_template_complex_separator() {
        assert_eq!(
            process("hello::world::test", r"{split:\:\::..|map:{upper}|join:,}").unwrap(),
            "HELLO,WORLD,TEST"
        );
    }
}

pub mod comprehensive_scenarios {
    use super::process;

    #[test]
    fn test_csv_processing_with_map() {
        let csv_line = "John Doe,25,Engineer,New York";
        assert_eq!(
            process(csv_line, "{split:,:..|map:{trim|upper}}").unwrap(),
            "JOHN DOE,25,ENGINEER,NEW YORK"
        );
    }

    #[test]
    fn test_log_processing_with_map() {
        let log_line = "2023-01-01 ERROR Failed to connect,2023-01-02 INFO Connected successfully,2023-01-03 WARN Connection timeout";
        assert_eq!(
            process(
                log_line,
                r"{split:,:..|map:{regex_extract:\d\d\d\d-\d\d-\d\d|append: (DATE)}}"
            )
            .unwrap(),
            "2023-01-01 (DATE),2023-01-02 (DATE),2023-01-03 (DATE)"
        );
    }

    #[test]
    fn test_file_extension_processing() {
        assert_eq!(
            process(
                "file1.txt,file2.pdf,file3.doc",
                r"{split:,:..|map:{regex_extract:\.\w+$|upper}}"
            )
            .unwrap(),
            ".TXT,.PDF,.DOC"
        );
    }

    #[test]
    fn test_url_processing() {
        let urls = "https://example.com/page1,http://test.org/page2,https://demo.net/page3";
        assert_eq!(
            process(
                urls,
                r"{split:,:..|map:{regex_extract://([^/]+):1|prepend:HOST\: }}"
            )
            .unwrap(),
            "HOST: example.com,HOST: test.org,HOST: demo.net"
        );
    }

    #[test]
    fn test_email_processing() {
        let emails = "john@example.com,jane@test.org,bob@demo.net";
        assert_eq!(
            process(
                emails,
                r"{split:,:..|map:{regex_extract:@(.+):1|upper|prepend:DOMAIN\: }}"
            )
            .unwrap(),
            "DOMAIN: EXAMPLE.COM,DOMAIN: TEST.ORG,DOMAIN: DEMO.NET"
        );
    }

    #[test]
    fn test_data_cleaning_pipeline() {
        let messy_data = "  John123  ,  Jane456  ,  Bob789  ";
        assert_eq!(
            process(
                messy_data,
                r"{split:,:..|map:{trim|regex_extract:[A-Za-z]+|lower|prepend:clean_}}"
            )
            .unwrap(),
            "clean_john,clean_jane,clean_bob"
        );
    }
}
