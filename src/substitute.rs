use crate::args::{Filters, Flags};
use std::env;
use std::io::{BufRead, BufReader};

// get environment variable value
fn get_env_var_value(
    var_name: &str,
    original_variable: String,
    default_value: &str,
    flags: &Flags,
) -> Result<String, String> {
    let var_value = env::var(var_name).unwrap_or(default_value.to_string());

    if flags.fail_on_empty && var_value.is_empty() {
        return Err(format!("environment variable '{}' is empty", var_name));
    }
    if flags.fail_on_unset && var_value.is_empty() && default_value.is_empty() {
        return Err(format!("environment variable '{}' is not set", var_name));
    }
    if flags.no_replace_empty && var_value.is_empty() && default_value.is_empty() {
        return Ok(original_variable);
    }
    if flags.no_replace_unset && var_value.is_empty() && default_value.is_empty() {
        return Ok(original_variable);
    }

    return Ok(var_value);
}

// checks if a filter is set and if the variable name matches the filter
// returns None if no filters are set
fn matches_filters(filters: &Filters, var_name: &str) -> Option<bool> {
    // has_filter is true if at least one filter is set
    let has_filter: bool =
        filters.prefix.is_some() || filters.suffix.is_some() || filters.variables.is_some();
    if !has_filter {
        return None;
    }

    // check if the variable name matches the filters
    let match_prefix: bool = filters
        .prefix
        .as_ref()
        .map_or(false, |p| var_name.starts_with(p));
    let match_suffix: bool = filters
        .suffix
        .as_ref()
        .map_or(false, |s| var_name.ends_with(s));
    let match_variable: bool = filters
        .variables
        .as_ref()
        .map_or(false, |v| v.contains(&var_name.to_string()));

    // has_match is true if the var_name matches at least one filter
    let has_match: bool = match_prefix || match_suffix || match_variable;

    return Some(has_match);
}

// substitute variables in the line
fn process_line(line: &str, flags: &Flags, filters: &Filters) -> Result<String, String> {
    let mut start_index: usize = 0;
    let mut new_line: String = String::new();
    let line_len: usize = line.len();

    while start_index < line_len {
        let c: char = line.chars().nth(start_index).unwrap(); // current character

        if c != '$' {
            // if the character is not a '$', skip substitution
            new_line.push(c);
            start_index += 1;
            continue;
        }

        let mut var_start: usize = start_index + 1;
        let mut var_end: usize = start_index + 1;
        let mut brace_ended: bool = false;
        let var_name: &str; // extracted variable name
        let original_var: &str; // original variable name, including the braces
        let mut default_value: String = "".to_string(); // part after the ':-' in ${VARNAME:-default_value}

        match line.chars().nth(var_end) {
            // check if the character after '$' is a '{' (${VARNAME} or ${VARNAME:-default_value})
            Some(c) if c == '{' => {
                var_start += 1; // skip the '{' character
                var_end = var_start;

                // check if the character after '{' is a number
                // if so, skip because it is not a valid variable => ${1VAR}
                // var_start < line_len means that the { is not at the end of the line
                if var_start < line_len && line.chars().nth(var_start).unwrap().is_numeric() {
                    new_line.push('$');
                    start_index += 1;
                    continue;
                }

                let mut default_value_found: bool = false;
                let mut default_value_start: usize = var_start;

                // iterate over the characters until the closing brace is found
                // in the same loop, check if a default value is provided
                while var_end < line_len {
                    // check if the character is a ':' this can be a default value
                    if line.chars().nth(var_end).unwrap() == ':' {
                        // check if var_end + 1 is out of bounds
                        // this is only necessary if it is a broken variable like ${VARNAME:
                        if (var_end + 1) >= line_len {
                            break;
                        }
                        // check if the next character is a '-'
                        if line.chars().nth(var_end + 1).unwrap() != '-' {
                            // if the next character is not a '-', it is not a default value
                            break;
                        }
                        default_value_found = true;
                        default_value_start = var_end;
                        var_end += 1; // skip also '-' characters
                    }

                    if line.chars().nth(var_end).unwrap() == '}' {
                        // brace is closed, finish searching
                        brace_ended = true;
                        break;
                    }
                    var_end += 1; // end not found, continue searching
                }

                // if the brace is not closed, do not any substitution
                if !brace_ended {
                    new_line.push('$');
                    start_index += 1;
                    continue;
                }

                if default_value_found {
                    // +2 to skip the ':-'
                    default_value = line[default_value_start + 2..var_end].to_string();
                } else {
                    // no default value found, set to the end of the variable name
                    default_value_start = var_end;
                }

                original_var = &line[var_start - 2..var_end + 1]; // with dollar sign and braces
                var_name = &line[var_start..default_value_start]; // extract the variable name
            }
            // processing of $VARNAME
            Some(c) if c.is_alphabetic() || c == '_' => {
                // search for the end of the variable name
                while var_end < line_len
                    && (line.chars().nth(var_end).unwrap().is_alphanumeric()
                        || line.chars().nth(var_end).unwrap() == '_')
                {
                    var_end += 1;
                }

                original_var = &line[var_start - 1..var_end]; // with dollar sign
                var_name = &line[var_start..var_end]; // extract the variable name
            }
            // if the character after '$' is not a '{' and not an alphabetic character and not '_',
            // then we don't do any substitution
            _ => {
                new_line.push(c);
                start_index += 1;
                continue;
            }
        }

        // check if value before was a dollar sign or a slash
        if !flags.no_escape
            && start_index > 1 // variable is not at the beginning of the line
            && line.chars().nth(start_index - 1).unwrap() == '$'
        {
            start_index += 1;
            continue;
        }

        // check if filters are set and if so, if filters match
        if matches_filters(filters, var_name) == Some(false) {
            new_line.push_str(original_var);
            start_index = if brace_ended { var_end + 1 } else { var_end };
            continue;
        }

        match get_env_var_value(var_name, original_var.to_string(), &default_value, flags) {
            Ok(val) => new_line.push_str(&val),
            Err(err) => return Err(err),
        }

        // if the variable name ends with a brace, then we don't include the brace in the substitution
        start_index = if brace_ended { var_end + 1 } else { var_end };
    }

    return Ok(new_line);
}

// function to perform the substitution on the input file and write the result to output file
pub fn perform_substitution(
    input_file: Box<dyn std::io::Read>,
    mut output_file: Box<dyn std::io::Write>,
    flags: &Flags,
    filters: &Filters,
) -> Result<(), String> {
    let reader = BufReader::new(input_file);
    let mut buffer = vec![]; // Vector to store the processed lines

    // replace variables in each line and write the replaced line in a buffer
    // if no error occurs, write the butter to the wanted output_file (or stdout)
    for line in reader.lines() {
        let line = line.unwrap();
        // Replace variables with their values
        let replaced: Result<String, String> = process_line(&line, flags, filters);
        match replaced {
            Ok(output) => buffer.push(output),
            Err(e) => return Err(format!("Failed to replace variables: {}", e)),
        }
    }
    // Write the processed lines to the output buffer
    for line in buffer {
        match writeln!(output_file, "{}", line) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Failed to write to output file: {}", e));
            }
        }
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    const EMPTY_FLAGS: Flags = Flags {
        no_escape: false,
        no_replace_unset: false,
        no_replace_empty: false,
        fail_on_unset: false,
        fail_on_empty: false,
    };
    const EMPTY_FILTERS: Filters = Filters {
        prefix: None,
        suffix: None,
        variables: None,
    };

    #[test]
    fn test_process_line_regular_var_found() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        env::set_var("REGULAR_VAR_FOUND", "value");
        let line = "$REGULAR_VAR_FOUND".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_regular_var_starting_dash() {
        // description: regular variable with starting dash
        // test: $_REGULAR_VAR_FOUND_WITH_DASH
        // env: _REGULAR_VAR_FOUND_WITH_DASH=value
        // result: value
        env::set_var("_REGULAR_VAR_FOUND_WITH_DASH", "value");
        let line = "$_REGULAR_VAR_FOUND_WITH_DASH".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_regular_var_not_found() {
        // description: regular variable not found
        // test: $REGULAR_VAR_NOT_FOUND
        // env: -
        // result: -
        let line = "$REGULAR_VAR_NOT_FOUND".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_found() {
        // description: braces variable found
        // test: ${BRACES_VAR_FOUND}
        // env: BRACES_VAR_FOUND=value
        // result: value
        env::set_var("BRACES_VAR_FOUND", "value");
        let line = "${BRACES_VAR_FOUND}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_found_starting_dash() {
        // description: braces variable found with starting dash
        // test: ${_BRACES_VAR_WITH_DASH}
        // env: _BRACES_VAR_WITH_DASH=value
        // result: value
        env::set_var("_BRACES_VAR_WITH_DASH", "value");
        let line = "${_BRACES_VAR_WITH_DASH}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_not_found() {
        // description: braces variable not found
        // test: ${BRACES_VAR_NOT_FOUND}
        // env: unset
        // result: -
        let line = "${BRACES_VAR_NOT_FOUND}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_default_use_default() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT:-default}
        // env: unset
        // result: default
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT:-default}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("default".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_default_use_var() {
        // description: braces variable with default value, use variable
        // test: ${BRACES_VAR_DEFAULT_USE_VAR:-default}
        // env: BRACES_VAR_DEFAULT_USE_VAR=value
        // result: value
        env::set_var("BRACES_VAR_DEFAULT_USE_VAR", "value");
        let line = "${BRACES_VAR_DEFAULT_USE_VAR:-default}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_default_use_var_dash() {
        // description: braces variable with default value, use variable
        // test: ${_BRACES_VAR_DEFAULT_USE_VAR:-default}
        // env: _BRACES_VAR_DEFAULT_USE_VAR=value
        // result: value
        env::set_var("_BRACES_VAR_DEFAULT_USE_VAR", "value");
        let line = "${_BRACES_VAR_DEFAULT_USE_VAR:-default}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_default_use_default_dash() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT_DASH:-_default}
        // env: BRACES_VAR_DEFAULT_USE_DEFAULT_DASH=value
        // result: value
        env::set_var("BRACES_VAR_DEFAULT_USE_DEFAULT_DASH", "value");
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT_DASH:-default}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_process_line_braces_var_default_use_default_empty() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-}
        // env: unset
        // result: -
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("".to_string()));
    }

    #[test]
    fn test_process_line_escape_text_double_dollar_invalid_var() {
        // description: escape text, double dollar, invalid var
        // test: i like cas$$ not so much!
        // env: -
        // result: i like cas$$ not so much!
        let line = "i like cas$$ not so much!".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_process_line_escape_text_double_dollar_invald_var_no_escape_true() {
        // description: escape text, double dollar, no escape true
        // test: i like cas$$ not so much!
        // env: -
        // result: i like cas$ not so much!
        let line = "i like cas$$ not so much!".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_escape: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_process_line_escape_var_double_dollar_valid_var() {
        // description: escape variable, double dollar, valid var
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$word
        let line = "I have a pa$$word".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("I have a pa$word".to_string()));
    }

    #[test]
    fn test_process_line_escape_var_double_dollar_no_replace_unset() {
        // description: escape variable, double dollar, no replace unset
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$word
        let line = "I have a pa$$word".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_unset: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok("I have a pa$word".to_string()));
    }

    #[test]
    fn test_process_line_escape_text_single_dollar_no_escape_true() {
        // description: escape text, single dollar, no escape
        // test: this $ is a dollar sign
        // env: -
        // result: this $ is a dollar sign
        let line = "this $ is a dollar sign".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_escape: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_process_line_escape_var_double_dollar_no_escape() {
        // description: escape variable, double dollar, no replace unset, no escape
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$$word
        let line = "I have a pa$$word".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_escape: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok("I have a pa$".to_string()));
    }

    #[test]
    fn test_process_line_escape_text_single_dollar_no_escape_false() {
        // description: escape text, single dollar, no escape
        // test: this $ is a dollar sign
        // env: -
        // result: this $ is a dollar sign
        let line = "this $ is a dollar sign".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_escape: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_process_line_broken_var_braces_end() {
        // description: broken variable, braces end
        // test: this variable $BROKEN_VAR_BRACES_END} is broken
        // env: BROKEN_VAR_BRACES_END=value
        // result: this variable value} is broken
        env::set_var("BROKEN_VAR_BRACES_END", "value");
        let line = "this variable $BROKEN_VAR_BRACES_END} is broken".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("this variable value} is broken".to_string()));
    }

    #[test]
    fn test_process_line_broken_var_braces_begin() {
        // description: broken variable, braces begin
        // test: this variable ${BROKEN_VAR_BRACES_BEGIN is broken
        // env: BROKEN_VAR_BRACES_BEGIN=value
        // result: this variable ${BROKEN_VAR_BRACES_BEGIN is broken
        env::set_var("BROKEN_VAR_BRACES_BEGIN", "value");
        let line = "this variable ${BROKEN_VAR_BRACES_END is broken".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("this variable ${BROKEN_VAR_BRACES_END is broken".to_string())
        );
    }

    #[test]
    fn test_process_line_invalid_regular_var_digit_begin() {
        // description: invalid regular variable, digit begin
        // test: this $1INVALID_VAR_DIGIT_BEGIN is not valid
        // env: -
        // result: this $1INVALID_VAR_DIGIT_BEGIN is not valid
        let line = "this $1INVALID_VAR_DIGIT_BEGIN is not valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("this $1INVALID_VAR_DIGIT_BEGIN is not valid".to_string())
        );
    }

    #[test]
    fn test_process_line_invalid_braces_var_digit_begin() {
        // description: invalid braces variable, digit begin
        // test: this ${1INVALID_VAR_DIGIT_BEGIN} is not valid
        // env: -
        // result: this ${1INVALID_VAR_DIGIT_BEGIN} is not valid
        let line = "this ${1INVALID_VAR_DIGIT_BEGIN} is not valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("this ${1INVALID_VAR_DIGIT_BEGIN} is not valid".to_string())
        );
    }

    #[test]
    fn test_process_line_valid_regular_var_digit_middle() {
        // description: valid regular variable, digit middle
        // test: this $VALID_REGULAR_VAR_1_DIGIT_MIDDLE is valid
        // env: VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_1_DIGIT_MIDDLE", "value");
        let line = "this $VALID_REGULAR_VAR_1_DIGIT_MIDDLE is valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_process_line_valid_regular_var_digit_end() {
        // description: valid regular variable, digit end
        // test: this $VALID_REGULAR_VAR_DIGIT_END_1 is valid
        // env: VALID_REGULAR_VAR_DIGIT_END_1=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_DIGIT_END_1", "value");
        let line = "this $VALID_REGULAR_VAR_DIGIT_END_1 is valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_process_line_valid_braces_var_digit_middle() {
        // description: valid braces variable, digit middle
        // test: this ${VALID_REGULAR_VAR_1_DIGIT_MIDDLE} is valid
        // env: VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_1_DIGIT_MIDDLE", "value");
        let line = "this ${VALID_REGULAR_VAR_1_DIGIT_MIDDLE} is valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_process_line_valid_braces_var_digit_end() {
        // description: valid braces variable, digit end
        // test: this ${VALID_REGULAR_VAR_DIGIT_END_1} is valid
        // env: VALID_REGULAR_VAR_DIGIT_END_1=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_DIGIT_END_1", "value");
        let line = "this ${VALID_REGULAR_VAR_DIGIT_END_1} is valid".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_process_line_valid_braces_var_end() {
        // description: valid braces variable, end of line
        // test: braces var at the end ${VALID_BRACES_VAR_END}
        // env: VALID_BRACES_VAR_END=value
        // result: braces var at the end value
        env::set_var("VALID_BRACES_VAR_END", "value");
        let line = "braces var at the end ${VALID_BRACES_VAR_END}".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("braces var at the end value".to_string()));
    }

    #[test]
    fn test_process_line_valid_braces_var_begin() {
        // description: valid braces variable, begin of line
        // test: ${VALID_BRACES_VAR_BEGIN} braces var at the begin
        // env: VALID_BRACES_VAR_BEGIN=value
        // result: value braces var at the begin
        env::set_var("VALID_BRACES_VAR_BEGIN", "value");
        let line = "${VALID_BRACES_VAR_BEGIN} braces var at the begin".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value braces var at the begin".to_string()));
    }

    #[test]
    fn test_process_line_valid_regular_var_end() {
        // description: valid regular variable, at end of line
        // test: regular var at the end $VALID_REGULAR_VAR_END
        // env: VALID_REGULAR_VAR_END=value
        // result: regular var at the end value
        env::set_var("VALID_REGULAR_VAR_END", "value");
        let line = "regular var at the end $VALID_REGULAR_VAR_END".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("regular var at the end value".to_string()));
    }

    #[test]
    fn test_process_line_valid_regular_var_begin() {
        // description: valid regular variable, at begin of line
        // test: $VALID_REGULAR_VAR_BEGIN regular var at the begin
        // env: VALID_REGULAR_VAR_BEGIN=value
        // result: value regular var at the begin
        env::set_var("VALID_REGULAR_VAR_BEGIN", "value");
        let line = "$VALID_REGULAR_VAR_BEGIN regular var at the begin".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(result, Ok("value regular var at the begin".to_string()));
    }

    #[test]
    fn test_process_line_valid_regular_var_fail_on_unset() {
        // description: valid regular variable, fail on empty
        // test: $VALID_REGULAR_VAR_FAIL_ON_UNSET
        // env:
        // result:
        let line = "$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string();
        let result = process_line(
            &line,
            &Flags {
                fail_on_unset: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_line_valid_braces_var_fail_on_unset() {
        // description: valid braces variable, fail on unset
        // test: ${VALID_BRACES_VAR_FAIL_ON_UNSET}
        // env:
        // result:
        let line = "${VALID_BRACES_VAR_FAIL_ON_UNSET}".to_string();
        let result = process_line(
            &line,
            &Flags {
                fail_on_unset: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_line_valid_regular_var_fail_on_empty() {
        // description: valid regular variable, fail on empty
        // test: $VALID_REGULAR_VAR_BEGIN
        // env: VALID_REGULAR_VAR_BEGIN=""
        // result: -
        env::set_var("VALID_REGULAR_VAR_FAIL_ON_EMPTY", "");
        let line = "$VALID_REGULAR_VAR_FAIL_ON_EMPTY".to_string();
        let result = process_line(
            &line,
            &Flags {
                fail_on_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_line_valid_braces_var_fail_on_empty() {
        // description: valid braces variable, fail on empty
        // test: $VALID_REGULAR_VAR_BEGIN regular var at the begin
        // env: VALID_REGULAR_VAR_BEGIN=""
        // result: -
        env::set_var("VALID_REGULAR_VAR_FAIL_ON_EMPTY", "");
        let line = "${VALID_REGULAR_VAR_FAIL_ON_EMPTY}".to_string();
        let result = process_line(
            &line,
            &Flags {
                fail_on_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_line_valid_regular_var_no_replace_unset() {
        // description: valid regular variable, no replace on unset
        // test: $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET
        // env:
        // result: $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET
        let line = "$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_unset: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok("$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string()));
    }

    #[test]
    fn test_process_line_valid_braces_var_no_replace_unset() {
        // description: valid braces variable, no replace on unset
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET}
        // env:
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET}
        let line = "${VALID_REGULAR_VAR_FAIL_ON_UNSET}".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_unset: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok("${VALID_REGULAR_VAR_FAIL_ON_UNSET}".to_string()));
    }

    #[test]
    fn test_process_line_valid_regular_var_no_replace_empty() {
        // description: valid regular variable, no replace on empty
        // test: $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY
        // env: VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY=""
        // result: $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY
        env::set_var("VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY", "");
        let line = "$VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(
            result,
            Ok("$VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY".to_string())
        );
    }

    #[test]
    fn test_process_line_valid_braces_var_no_replace_empty() {
        // description: valid braces variable, no replace on empty
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}
        // env: VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY=""
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}
        env::set_var("VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY", "");
        let line = "${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(
            result,
            Ok("${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}".to_string())
        );
    }

    #[test]
    fn test_process_line_invalid_braces_var_default_end() {
        // description: invalid braces variable, default at the end
        // test: ${IVALID_BRACES_VAR_DEFAULT_END:-
        // env: -
        // result: ${IVALID_BRACES_VAR_DEFAULT_END:-
        let line = "${IVALID_BRACES_VAR_DEFAULT_END:-".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(result, Ok("${IVALID_BRACES_VAR_DEFAULT_END:-".to_string()));
    }

    #[test]
    fn test_process_line_invalid_braces_var_broken_default_end() {
        // description: invalid braces variable, default at the end
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:
        // env: -
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:
        let line = "${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(
            result,
            Ok("${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:".to_string())
        );
    }

    #[test]
    fn test_process_line_dollar_end() {
        // description: only one dollar sign at the end of line
        // test: this is a test line with only one dollar sign at the end of line $
        // env: -
        // result: this is a test line with only one dollar sign at the end of line $
        let line = "this is a test line with only one dollar sign at the end of line $".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("this is a test line with only one dollar sign at the end of line $".to_string())
        );
    }

    #[test]
    fn test_process_line_double_dollar_end() {
        // description: two dollar sign at the end of line
        // test: this is a test line with two dollar sign at the end of line $$
        // env: -
        // result: this is a test line with two dollar sign at the end of line $$
        let line = "this is a test line with two dollar sign at the end of line $$".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("this is a test line with two dollar sign at the end of line $$".to_string())
        );
    }

    #[test]
    fn test_process_line_double_dollar_end_escape_true() {
        // description: double dollar sign at the end of line, escape true
        // test: this is a test line with two dollar sign at the end of line $$
        // env: -
        // result: this is a test line with two dollar sign at the end of line $$
        let line = "this is a test line with two dollar sign at the end of line $$".to_string();
        let result = process_line(
            &line,
            &Flags {
                no_escape: true,
                ..EMPTY_FLAGS
            },
            &EMPTY_FILTERS,
        );
        assert_eq!(
            result,
            Ok("this is a test line with two dollar sign at the end of line $$".to_string())
        );
    }

    #[test]
    fn test_process_line_regular_var_prefix() {
        // description: regular variable with prefix
        // test: this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.
        // env: ENV1=env1, TEST_VAR1=test_var1
        // result:this $ENV1 has a prefix. This test_var1 has a prefix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        let line = "this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                prefix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This test_var1 has a prefix.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_prefix() {
        // description: braces variable with prefix
        // test: this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.
        // env: ENV1=env1, TEST_VAR1=test_var1
        // result:this $ENV1 has a prefix. This test_var1 has a prefix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        let line = "this $ENV1 has a no prefix. This ${TEST_VAR1} has a valid prefix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                prefix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a no prefix. This test_var1 has a valid prefix.".to_string())
        );
    }

    #[test]
    fn test_process_line_regular_var_suffix() {
        // description: regular variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $ENV1 has a prefix. This $VAR1_TEST has a suffix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                suffix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $ENV1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                suffix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_no_prefix_valid_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                suffix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this $TEST_VAR1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_valid_prefix_no_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                prefix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this test_var1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_valid_prefix_valid_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this var $ENV1 should not be touched. this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                suffix: Some("TEST".to_string()),
                prefix: Some("TEST".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("this var $ENV1 should not be touched. this test_var1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }
    #[test]
    fn test_process_line_regular_var_list_variables() {
        // description: regular variable with a list of variables
        // test: Only ENV1 and ENV2 should be replaced. ENV3 should not be replaced.
        // env: ENV1=env1, ENV2=env2
        // result: Only env1 and env2 should be replaced. ENV2 should not be replaced.
        env::set_var("ENV1", "env1");
        env::set_var("ENV2", "env2");
        env::set_var("ENV3", "env4");
        let line =
            "Only $ENV1 and $ENV2 should be replaced. $ENV3 should not be replaced.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                variables: Some(vec!["ENV1".to_string(), "ENV2".to_string()]),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(
            result,
            Ok("Only env1 and env2 should be replaced. $ENV3 should not be replaced.".to_string())
        );
    }

    #[test]
    fn test_process_line_regular_var_list_variables_prefix_suffix_not_found() {
        // description: all filter set, non matches
        // test: $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.
        // env: -
        // result: $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.
        let line = "$PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                variables: Some(vec!["ENV1".to_string(), "ENV2".to_string()]),
                prefix: Some("BAD_PREFIX".to_string()),
                suffix: Some("BAD_SUFFIX".to_string()),
            },
        );
        assert_eq!(
            result,
            Ok("$PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.".to_string())
        );
    }

    #[test]
    fn test_process_line_braces_var_invalid_default() {
        // description: braces variable with invalid default
        // test: This ${BRACES_VAR_INVALID_DEFAULT:-def:ault} a broken default.
        // env: BRACES_VAR_INVALID_DEFAULT=var1_var
        // result: This ${BRACES_VAR_INVALID_DEFAULT:-def:ault} a broken default.
        env::set_var("BRACES_VAR_INVALID_DEFAULT", "var1_test");
        let line = "This ${BRACES_VAR_INVALID_DEFAULT:-def:ault} a broken default.".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("This ${BRACES_VAR_INVALID_DEFAULT:-def:ault} a broken default.".to_string())
        );
    }

    #[test]
    fn test_process_line_regular_var_all_filter_match() {
        // description: all filter set, all match
        // test: ${PREFIX_VAR_SUFFIX}
        // env: -
        // result: prefix var suffix
        env::set_var("PREFIX_VAR_SUFFIX", "prefix var suffix");
        let line = "${PREFIX_VAR_SUFFIX}".to_string();
        let result = process_line(
            &line,
            &EMPTY_FLAGS,
            &Filters {
                variables: Some(vec!["PREFIX_VAR_SUFFIX".to_string()]),
                prefix: Some("PREFIX".to_string()),
                suffix: Some("SUFFIX".to_string()),
                ..EMPTY_FILTERS
            },
        );
        assert_eq!(result, Ok("prefix var suffix".to_string()));
    }
    #[test]
    fn test_matches_filters_no_filters() {
        // description: no filters
        // test: -
        // env: -
        // result: true
        assert_eq!(matches_filters(&EMPTY_FILTERS, "VAR"), None);
    }

    #[test]
    fn test_matches_filters_all_filters() {
        // description: all filters
        // test: -
        // env: -
        // result: true
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    prefix: Some("PREFIX".to_string()),
                    suffix: Some("SUFFIX".to_string()),
                },
                "PREFIX_VAR_SUFFIX"
            ),
            Some(true)
        );
    }
    #[test]
    fn test_matches_filters_prefix() {
        // description: prefix filter
        // test: -
        // env: -
        // result: true
        assert_eq!(
            matches_filters(
                &Filters {
                    prefix: Some("PREFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "PREFIX_VAR"
            ),
            Some(true)
        );
    }
    #[test]
    fn test_matches_filters_suffix() {
        // description: suffix filter
        // test: -
        // env: -
        // result: true
        assert_eq!(
            matches_filters(
                &Filters {
                    suffix: Some("SUFFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR_SUFFIX"
            ),
            Some(true)
        );
    }
    #[test]
    fn test_matches_filters_variables() {
        // description: variables filter
        // test: -
        // env: -
        // result: true
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    ..EMPTY_FILTERS
                },
                "VAR"
            ),
            Some(true)
        );
    }
    #[test]
    fn test_matches_filters_prefix_not_found() {
        // description: prefix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    prefix: Some("PREFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_suffix_not_found() {
        // description: suffix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    suffix: Some("SUFFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_variables_not_found() {
        // description: variables filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    ..EMPTY_FILTERS
                },
                "VAR2"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_prefix_suffix_not_found() {
        // description: prefix and suffix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    prefix: Some("PREFIX".to_string()),
                    suffix: Some("SUFFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_variables_prefix_not_found() {
        // description: variables and prefix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    prefix: Some("PREFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR2"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_variables_suffix_not_found() {
        // description: variables and suffix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    suffix: Some("SUFFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR2"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_matches_filters_variables_prefix_suffix_not_found() {
        // description: variables, prefix and suffix filter not found
        // test: -
        // env: -
        // result: false
        assert_eq!(
            matches_filters(
                &Filters {
                    variables: Some(vec!["VAR".to_string()]),
                    prefix: Some("PREFIX".to_string()),
                    suffix: Some("SUFFIX".to_string()),
                    ..EMPTY_FILTERS
                },
                "VAR2"
            ),
            Some(false)
        );
    }
    #[test]
    fn test_evaluate_variable_regular_var() {
        // description: regular variable
        // test: ${VAR}
        // env: VAR=var
        // result: var
        env::set_var("REGULAR_VAR", "var");
        let var_name = "REGULAR_VAR";
        let original_var = "${REGULAR_VAR}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags { ..EMPTY_FLAGS },
        );
        assert_eq!(result, Ok("var".to_string()));
    }

    #[test]
    fn test_evaluate_variable_regular_var_with_default() {
        // description: regular variable with default value
        // test: ${VAR:-default}
        // env: VAR=var
        // result: var
        env::set_var("REGULAR_VAR_WITH_DEFAULT", "var");
        let var_name = "REGULAR_VAR_WITH_DEFAULT";
        let original_var = "${REGULAR_VAR_WITH_DEFAULT:-default}";
        let default_value = "default";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags { ..EMPTY_FLAGS },
        );
        assert_eq!(result, Ok("var".to_string()));
    }

    #[test]
    fn test_evaluate_variable_regular_var_no_replace_empty_true() {
        // description: regular variable with no replace empty true
        // test: ${VAR}
        // env: VAR=
        // result: ""
        env::set_var("REGULAR_VAR_NO_REPLACE_EMTPY_TRUE", "");
        let var_name = "REGULAR_VAR_NO_REPLACE_EMTPY_TRUE";
        let original_var = "${REGULAR_VAR_NO_REPLACE_EMTPY_TRUE}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
        );
        assert_eq!(
            result,
            Ok("${REGULAR_VAR_NO_REPLACE_EMTPY_TRUE}".to_string())
        );
    }

    #[test]
    fn test_evaluate_variable_regular_var_no_replace_unset() {
        // description: regular variable with no replace unset
        // test: ${VAR}
        // env: -
        // result: ""
        let var_name = "REGULAR_VAR_NO_REPLACE_UNSET";
        let original_var = "${REGULAR_VAR_NO_REPLACE_UNSET}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                no_replace_unset: true,
                ..EMPTY_FLAGS
            },
        );
        assert_eq!(result, Ok("${REGULAR_VAR_NO_REPLACE_UNSET}".to_string()));
    }

    #[test]
    fn test_evaluate_variable_regular_var_no_replace_unset_empty_true() {
        // description: regular variable with no replace unset and empty true
        // test: ${VAR}
        // env: -
        // result: ""
        let var_name = "REGULAR_VAR_NO_REPLACE_UNSET_EMPTY_TRUE";
        let original_var = "${REGULAR_VAR_NO_REPLACE_UNSET_EMPTY_TRUE}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                no_replace_unset: true,
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
        );
        assert_eq!(
            result,
            Ok("${REGULAR_VAR_NO_REPLACE_UNSET_EMPTY_TRUE}".to_string())
        );
    }

    #[test]
    fn test_evaluate_variable_regular_fail_on_empty() {
        // description: regular variable with fail on empty
        // test: ${VAR}
        // env: VAR=
        // result: error
        env::set_var("REGULAR_FAIL_ON_EMPTY", "");
        let var_name = "REGULAR_FAIL_ON_EMPTY";
        let original_var = "${REGULAR_FAIL_ON_EMPTY}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                fail_on_empty: true,
                ..EMPTY_FLAGS
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_variable_regular_fail_on_unset() {
        // description: regular variable with fail on unset
        // test: ${VAR}
        // env: -
        // result: error
        let var_name = "REGULAR_FAIL_ON_UNSET";
        let original_var = "${REGULAR_FAIL_ON_UNSET}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                fail_on_unset: true,
                ..EMPTY_FLAGS
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_variable_regular_fail_on_unset_empty_true() {
        // description: regular variable with fail on unset and empty true
        // test: ${VAR}
        // env: -
        // result: error
        let var_name = "REGULAR_FAIL_ON_UNSET_EMTPY_TRUE";
        let original_var = "${REGULAR_FAIL_ON_UNSET_EMTPY_TRUE}";
        let default_value = "";
        let result = get_env_var_value(
            &var_name,
            original_var.to_string(),
            default_value,
            &Flags {
                fail_on_unset: true,
                no_replace_empty: true,
                ..EMPTY_FLAGS
            },
        );
        // check if the result is an error
        assert!(result.is_err());
    }
}
