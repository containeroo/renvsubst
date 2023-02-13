# Testcases

# ./src/substitute.rs
| Test Name | Description | Test | Env | Result |
| :--- | :--- | :--- | :--- | :--- |
| test_process_line_regular_var_found | regular variable found | $REGULAR_VAR_NOT_FOUND | REGULAR_VAR_FOUND=value | value |
| test_process_line_regular_var_not_found | regular variable not found | $REGULAR_VAR_NOT_FOUND | - | - |
| test_process_line_braces_var_found | braces variable found | ${BRACES_VAR_FOUND} | BRACES_VAR_FOUND=value | value |
| test_process_line_braces_var_not_found | braces variable not found | ${BRACES_VAR_NOT_FOUND} | unset | - |
| test_process_line_braces_var_default_use_default | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT:-default} | unset | default |
| test_process_line_braces_var_default_use_var | braces variable with default value, use variable | ${BRACES_VAR_DEFAULT_USE_VAR:-default} | BRACES_VAR_DEFAULT_USE_VAR=value | value |
| test_process_line_braces_var_default_use_default_empty | braces variable with default value, use default | ${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-} | unset | - |
| test_process_line_escape_text_double_dollar_no_escape_true | escape text, double dollar, no escape | i like cas$$ not so much! | - | i like cas$$ not so much! |
| test_process_line_escape_text_double_dollar_no_escape_false | escape text, double dollar, no escape | i like cas$$ not so much! | - | i like cas$ not so much! |
| test_process_line_escape_var_double_dollar | escape variable, double dollar | I have a pa$$word | - | I have a pa$word |
| test_process_line_escape_var_double_dollar_no_replace_unset | escape variable, double dollar, no replace unset | I have a pa$$word | - | I have a pa$word |
| test_process_line_escape_var_double_dollar_no_replace_unset_no_escape | escape variable, double dollar, no replace unset, no escape | I have a pa$$word | - | I have a pa$$word |
| test_process_line_escape_text_single_dollar_no_escape_true | escape text, single dollar, no escape | this $ is a dollar sign | - | this $ is a dollar sign |
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
| test_process_line_double_dollar_end_escape_true | double dollar sign at the end of line, escape true | this is a test line with two dollar sign at the end of line $$ | - | this is a test line with two dollar sign at the end of line $$ |
| test_process_line_regular_var_prefix | regular variable with prefix | this $ENV1 has a prefix. This $TEST_VAR1 has a prefix. | ENV1=env1, TEST_VAR1=test_var1 | // result:this $ENV1 has a prefix. This test_var1 has a prefix. |
| test_process_line_braces_var_prefix | braces variable with prefix | this $ENV1 has a prefix. This $TEST_VAR1 has a prefix. | ENV1=env1, TEST_VAR1=test_var1 | // result:this $ENV1 has a prefix. This test_var1 has a prefix. |
| test_process_line_regular_var_suffix | regular variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_braces_var_suffix | braces variable with suffix | this $ENV1 has a prefix. This $VAR1_TEST has a suffix. | ENV1=env1, VAR1_TEST=var1_var | // result:this $ENV1 has a prefix. This test_var1 has a suffix. |
| test_process_line_regular_var_list_variables | regular variable with a list of variables | Only ENV1 and ENV2 should be replaced. ENV3 should not be replaced. | ENV1=env1, ENV2=env2 | Only env1 and env2 should be replaced. ENV2 should not be replaced. |


# ./src/file_io.rs
| Test Name | Description | Test | Env | Result |
| :--- | :--- | :--- | :--- | :--- |
| test_open_input_file_with_empty | let input = None; | let result = open_input_file(input); | assert!(result.is_ok()); | } |
| test_open_input_file_with_existent_file | let input_file = NamedTempFile::new().unwrap(); | let input_file_path = input_file.path().to_str().unwrap().to_string(); | let result = open_input_file(input_file_path.into()); | input_file.close().unwrap(); |
| test_open_input_file_with_nonexistent_file | let input = String::from("tests/nonexistent_file.txt"); | let result = open_input_file(input.into()); | assert!(result.is_err()); | } |
| test_open_output_file_with_stdout | let output = None; | let result = open_output_file(output); | assert!(result.is_ok()); | } |
| test_open_output_file_with_nonexistent_directory | let output = Some(String::from("tests/nonexistent_folder/output_file.txt")); | let result = open_output_file(output); | assert!(result.is_err()); | } |
| test_open_output_file_with_existent_file | let output_file = NamedTempFile::new().unwrap(); | let output_file_path = output_file.path().to_str().unwrap().to_string(); | let result = open_output_file(Some(output_file_path)); | output_file.close().unwrap(); |


# ./src/main.rs
| Test Name | Description | Test | Env | Result |
| :--- | :--- | :--- | :--- | :--- |


# ./src/args.rs
| Test Name | Description | Test | Env | Result |
| :--- | :--- | :--- | :--- | :--- |


