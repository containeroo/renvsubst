# Testcases

# ./src/substitute.rs
| Test Name | Description | Test | Env | Result |
| :--- | :--- | :--- | :--- | :--- |
| test_process_line_regular_var_found | regular variable found | $REGULAR_VAR_FOUND | REGULAR_VAR_FOUND=value | value |
| test_process_line_regular_var_starting_dash | regular variable with starting dash | $_REGULAR_VAR_FOUND_WITH_DASH | _REGULAR_VAR_FOUND_WITH_DASH=value | value |
| test_process_line_regular_var_not_found_fail_on_unset | regular variable not found | $REGULAR_VAR_NOT_FOUND | - | - |
| test_process_line_regular_var_not_found | regular variable not found | $REGULAR_VAR_NOT_FOUND | - | - |
| test_process_line_braces_var_found | braces variable found | ${BRACES_VAR_FOUND} | BRACES_VAR_FOUND=value | value |
| test_process_line_braces_var_found_starting_dash | braces variable found with starting dash | ${_BRACES_VAR_WITH_DASH} | _BRACES_VAR_WITH_DASH=value | value |
| test_process_line_regular_var_found_long_value | regular variable found | $REGULAR_VAR_FOUND | REGULAR_VAR_FOUND=value | value |
| test_process_line_braces_var_not_found | braces variable not found | ${BRACES_VAR_NOT_FOUND} | unset | - |
| test_process_line_braces_var_not_found_fail_on_unset | braces variable not found | ${BRACES_VAR_NOT_FOUND} | unset | - |
| test_process_line_braces_var_default_use_default | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT:-default} | unset | default |
| test_process_line_braces_var_default_use_colon_in_default | braces variable with colon inside default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa:ult} | unset | defa:ult |
| test_process_line_braces_var_default_use_dollar_in_default | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa$ult} | unset | defa:ult |
| test_process_line_braces_var_default_use_braces_in_default | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa$ult} | unset | defa:ult |
| test_process_line_braces_var_default_use_var | braces variable with default value, use variable | ${BRACES_VAR_DEFAULT_USE_VAR:-default} | BRACES_VAR_DEFAULT_USE_VAR=value | value |
| test_process_line_braces_var_default_use_var_dash | braces variable with default value, use variable | ${_BRACES_VAR_DEFAULT_USE_VAR:-default} | _BRACES_VAR_DEFAULT_USE_VAR=value | value |
| test_process_line_braces_var_default_use_default_dash | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT_DASH:-_default} | BRACES_VAR_DEFAULT_USE_DEFAULT_DASH=value | value |
| test_process_line_braces_var_default_use_default_empty | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-} | unset | - |
| test_process_line_escape_text_double_dollar_invalid_var | escape text, double dollar, invalid var | i like cas$$ not so much! | - | i like cas$$ not so much! |
| test_process_line_escape_text_double_dollar_invald_var_no_escape_true | escape text, double dollar, no escape true | i like cas$$ not so much! | - | i like cas$ not so much! |
| test_process_line_escape_var_double_dollar_valid_var | escape variable, double dollar, valid var | I have a pa$$word | - | I have a pa$word |
| test_process_line_escape_var_double_dollar_no_replace_unset | escape variable, double dollar, no replace unset | I have a pa$$word | - | I have a pa$word |
| test_process_line_escape_text_single_dollar_no_escape_true | escape text, single dollar, no escape | this $ is a dollar sign | - | this $ is a dollar sign |
| test_process_line_escape_var_double_dollar_no_escape | escape variable, double dollar, no escape | I have a pa$$word | - | I have a pa$$word |
| test_process_line_escape_text_single_dollar_no_escape_false | escape text, single dollar, no escape | this $ is a dollar sign | - | this $ is a dollar sign |
| test_process_line_broken_var_braces_end | broken variable, braces end | this variable $BROKEN_VAR_BRACES_END} is broken | BROKEN_VAR_BRACES_END=value | this variable value} is broken |
| test_process_line_broken_var_braces_begin | broken variable, braces begin | this variable ${BROKEN_VAR_BRACES_BEGIN is broken | BROKEN_VAR_BRACES_BEGIN=value | this variable ${BROKEN_VAR_BRACES_BEGIN is broken |
| test_process_line_invalid_regular_var_digit_begin | invalid regular variable, digit begin | this $1INVALID_VAR_DIGIT_BEGIN is not valid | - | this $1INVALID_VAR_DIGIT_BEGIN is not valid |
| test_process_line_invalid_braces_var_digit_begin | invalid braces variable, digit begin | this ${1INVALID_VAR_DIGIT_BEGIN} is not valid | - | this ${1INVALID_VAR_DIGIT_BEGIN} is not valid |
| test_process_line_valid_regular_var_digit_middle | valid regular variable, digit middle | this $VALID_REGULAR_VAR_1_DIGIT_MIDDLE is valid | VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value | this value is valid |
| test_process_line_valid_regular_var_digit_end | valid regular variable, digit end | this $VALID_REGULAR_VAR_DIGIT_END_1 is valid | VALID_REGULAR_VAR_DIGIT_END_1=value | this value is valid |
| test_process_line_valid_braces_var_digit_middle | valid braces variable, digit middle | this ${VALID_REGULAR_VAR_1_DIGIT_MIDDLE} is valid | VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value | this value is valid |
| test_process_line_valid_braces_var_digit_end | valid braces variable, digit end | this ${VALID_REGULAR_VAR_DIGIT_END_1} is valid | VALID_REGULAR_VAR_DIGIT_END_1=value | this value is valid |
| test_process_line_valid_braces_var_end | valid braces variable, end of line | braces var at the end ${VALID_BRACES_VAR_END} | VALID_BRACES_VAR_END=value | braces var at the end value |
| test_process_line_valid_braces_var_begin | valid braces variable, begin of line | ${VALID_BRACES_VAR_BEGIN} braces var at the begin | VALID_BRACES_VAR_BEGIN=value | value braces var at the begin |
| test_process_line_valid_regular_var_end | valid regular variable, at end of line | regular var at the end $VALID_REGULAR_VAR_END | VALID_REGULAR_VAR_END=value | regular var at the end value |
| test_process_line_valid_regular_var_begin | valid regular variable, at begin of line | $VALID_REGULAR_VAR_BEGIN regular var at the begin | VALID_REGULAR_VAR_BEGIN=value | value regular var at the begin |
| test_process_line_valid_regular_var_fail_on_unset | valid regular variable, fail on empty | $VALID_REGULAR_VAR_FAIL_ON_UNSET | // env: | // result: |
| test_process_line_valid_braces_var_fail_on_unset | valid braces variable, fail on unset | ${VALID_BRACES_VAR_FAIL_ON_UNSET} | // env: | // result: |
| test_process_line_valid_regular_var_fail_on_empty | valid regular variable, fail on empty | $VALID_REGULAR_VAR_BEGIN | VALID_REGULAR_VAR_BEGIN="" | - |
| test_process_line_valid_braces_var_fail_on_empty | valid braces variable, fail on empty | $VALID_REGULAR_VAR_BEGIN regular var at the begin | VALID_REGULAR_VAR_BEGIN="" | - |
| test_process_line_valid_regular_var_no_replace_unset | valid regular variable, no replace on unset | $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET | // env: | $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET |
| test_process_line_valid_braces_var_no_replace_unset | valid braces variable, no replace on unset | ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET} | // env: | ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET} |
| test_process_line_valid_regular_var_no_replace_empty | valid regular variable, no replace on empty | $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY | VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY="" | $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY |
| test_process_line_valid_braces_var_no_replace_empty | valid braces variable, no replace on empty | ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY} | VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY="" | ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY} |
| test_process_line_invalid_braces_var_default_end | invalid braces variable, default at the end | ${IVALID_BRACES_VAR_DEFAULT_END:- | - | ${IVALID_BRACES_VAR_DEFAULT_END:- |
| test_process_line_invalid_braces_var_broken_default_end | invalid braces variable, default at the end | ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY: | - | ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY: |
| test_process_line_dollar_end | only one dollar sign at the end of line | this is a test line with only one dollar sign at the end of line $ | - | this is a test line with only one dollar sign at the end of line $ |
| test_process_line_double_dollar_end | two dollar sign at the end of line | this is a test line with two dollar sign at the end of line $$ | - | this is a test line with two dollar sign at the end of line $$ |
| test_process_line_double_dollar_end_escape_true | double dollar sign at the end of line, no escape true | this is a test line with two dollar sign at the end of line $$ | - | this is a test line with two dollar sign at the end of line $$ |
| test_process_line_regular_var_prefix | regular variable with prefix | this $ENV1 has a prefix. This $TEST_VAR1 has a prefix. | ENV1=env1, TEST_VAR1=test_var1 | // result:this $ENV1 has a prefix. This test_var1 has a prefix. |
| test_process_line_braces_var_prefix | braces variable with prefix | this $ENV1 has a prefix. This $TEST_VAR1 has a prefix. | ENV1=env1, TEST_VAR1=test_var1 | // result:this $ENV1 has a prefix. This test_var1 has a prefix. |
| test_process_line_regular_var_suffix | regular variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_regular_var_multiple_suffix | regular variable with multiple suffix | this $ENV1 has no suffix. This "$VAR_FIRST" has a suffix. And this "${VAR_SECOND}" has another suffix. | ENV1=env1, VAR_FIRST=first suffix, VAR_SECOND=second suffix | this this $ENV1 has a prefix. This "first suffix" has a suffix. And this "second suffix" has another suffix. |
| test_process_line_regular_var_multiple_prefix | regular variable with multiple prefix | this $ENV1 has no prefix. This "$FIRST_VAR" has a prefix. And this "${SECOND_VAR}" has another suffix. | ENV1=env1, FIRST_VAR=first prefix, SECOND_VAR=second prefix | this this $ENV1 has no prefix. This "first suffix" has a suffix. And this "second suffix" has another suffix. |
| test_process_line_braces_var_suffix | braces variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_braces_var_no_prefix_valid_suffix | braces variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_braces_var_valid_prefix_no_suffix | braces variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_braces_var_valid_no_prefix_valid_suffix | braces variable with suffix | this var $ENV1 should not be touched. this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. This var1_test has a suffix. |
| test_process_line_regular_var_list_variables | regular variable with a list of variables | Only ENV1 and ENV2 should be replaced. ENV3 should not be replaced. | ENV1=env1, ENV2=env2 | Only env1 and env2 should be replaced. ENV2 should not be replaced. |
| test_process_line_regular_var_list_variables_prefix_suffix_not_found | all filter set, non matches | $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced. | - | $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced. |
| test_process_line_regular_var_all_filter_match | all filter set, all match | ${PREFIX_VAR_SUFFIX} | - | prefix var suffix |
| test_matches_filters_no_filters | no filters | - | - | true |
| test_matches_filters_all_filters | all filters | - | - | true |
| test_matches_filters_prefix | prefix filter | - | - | true |
| test_matches_filters_suffix | suffix filter | - | - | true |
| test_matches_filters_variables | variables filter | - | - | true |
| test_matches_filters_prefix_not_found | prefix filter not found | - | - | false |
| test_matches_filters_suffix_not_found | suffix filter not found | - | - | false |
| test_matches_filters_variables_not_found | variables filter not found | - | - | false |
| test_matches_filters_prefix_suffix_not_found | prefix and suffix filter not found | - | - | false |
| test_matches_filters_variables_prefix_not_found | variables and prefix filter not found | - | - | false |
| test_matches_filters_variables_suffix_not_found | variables and suffix filter not found | - | - | false |
| test_matches_filters_variables_prefix_suffix_not_found | variables, prefix and suffix filter not found | - | - | false |
| test_evaluate_variable_regular_var | regular variable | ${VAR} | VAR=var | var |
| test_evaluate_variable_regular_var_with_default | regular variable with default value | ${VAR:-default} | VAR=var | var |
| test_evaluate_variable_regular_var_no_replace_empty_true | regular variable with no replace empty true | ${VAR} | VAR= | "" |
| test_evaluate_variable_regular_var_no_replace_unset | regular variable with no replace unset | ${VAR} | - | "" |
| test_evaluate_variable_regular_var_no_replace_unset_empty_true | regular variable with no replace unset and empty true | ${VAR} | - | "" |
| test_evaluate_variable_regular_fail_on_empty | regular variable with fail on empty | ${VAR} | VAR= | error |
| test_evaluate_variable_regular_fail_on_unset | regular variable with fail on unset | ${VAR} | - | error |
| test_evaluate_variable_regular_fail_on_unset_empty_true | regular variable with fail on unset and empty true | ${VAR} | - | error |
| test_example_process_line | let line = "Hello, ${NAME:-User}! How are you, ${NAME}?"; | let result = process_line(line, &Flags::default(), &Filters::default()); |  | assert!(result.is_ok()); |
| test_example_match_filters | let filters = Filters { | Some(HashSet::from_iter(vec!["prefixed_".to_string()])), | Some(HashSet::from_iter(vec!["_suffixed".to_string()])), | Some(HashSet::from_iter(vec![ |
| test_example_get_env_var_value | let var_value = get_env_var_value( | "MY_VAR", | "default_value", | "fallback_value", |
| test_example_perform_substitution | use std::io::Cursor; |  | let input = Cursor::new("Hello $WORLD!"); | let mut output = Cursor::new(Vec::new()); |
