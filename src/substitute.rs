use crate::args::{Filters, Flags};
use std::env;
use std::io::{BufRead, BufReader};

/// Retrieves the value of an environment variable and performs additional checks according to the specified `Flags`.
///
/// # Arguments
///
/// * `var_name` - A string reference to the name of the environment variable to retrieve.
/// * `original_variable` - A string reference to the original value of the variable (used for the `no_replace_*` flags).
/// * `default_value` - A string reference to the default value to use if the environment variable is not set.
/// * `flags` - A reference to a `Flags` object containing additional behavior settings.
///
/// # Returns
///
/// Returns a `Result` that is either an `Ok` containing the retrieved variable value, or an `Err` containing an error message if the `fail_on_empty` or `fail_on_unset` flags are set and the variable value does not meet the specified criteria.
///
/// # Errors
///
/// Returns an error if the `fail_on_empty` flag is set and the variable value is empty, or if the `fail_on_unset` flag is set, the variable value is empty, and the default value is also empty.
///
/// # Examples
///
/// ```
/// let var_name = "MY_ENV_VAR";
/// let default_value = "default_value";
/// let flags = Flags {
///     fail_on_empty: false,
///     fail_on_unset: false,
///     no_replace_empty: false,
///     no_replace_unset: false,
///     no_escape: false,
/// };
///
/// let result = get_env_var_value(var_name, "", default_value, &flags);
///
/// assert_eq!(result, Ok(default_value.to_string()));
/// ```
fn get_env_var_value(
    var_name: &str,
    original_variable: &str,
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
        return Ok(original_variable.to_string());
    }
    if flags.no_replace_unset && var_value.is_empty() && default_value.is_empty() {
        return Ok(original_variable.to_string());
    }

    return Ok(var_value);
}

/// Determines whether a given variable name matches any of the filters in the specified `Filters` object.
///
/// # Arguments
///
/// * `filters` - A reference to a `Filters` object containing optional filter criteria.
/// * `var_name` - A string reference to the variable name to be tested.
///
/// # Returns
///
/// Returns `Some(true)` if the variable name matches any of the filters, `Some(false)` if it doesn't match any of the filters, or `None` if no filters are set.
///
/// # Examples
///
/// ```
/// let filters = Filters {
///     prefix: Some("prefixed_".to_string()),
///     suffix: Some("_suffixed".to_string()),
///     variables: Some(vec!["my_variable".to_string(), "another_variable".to_string()]),
/// };
///
/// assert_eq!(matches_filters(&filters, "prefixed_variable"), Some(true));
/// assert_eq!(matches_filters(&filters, "variable_suffixed"), Some(true));
/// assert_eq!(matches_filters(&filters, "my_variable"), Some(true));
/// assert_eq!(matches_filters(&filters, "another_variable"), Some(true));
/// assert_eq!(matches_filters(&filters, "your_variable"), Some(false));
/// ```
fn matches_filters(filters: &Filters, var_name: &str) -> Option<bool> {
    // return None if no filters are set
    if !(filters.prefix.is_some() || filters.suffix.is_some() || filters.variables.is_some()) {
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

    return Some(match_prefix || match_suffix || match_variable);
}

/// Processes a single line of text and performs variable substitution according to the specified `Flags` and `Filters`.
///
/// # Arguments
///
/// * `line` - A string reference to the line of text to process.
/// * `flags` - A reference to a `Flags` object containing additional behavior settings.
/// * `filters` - A reference to a `Filters` object containing optional filter criteria for variable names.
///
/// # Returns
///
/// Returns a `Result` that is either an `Ok` containing the processed line of text with variable substitutions, or an `Err` containing an error message if an error occurs during variable substitution.
///
/// # Errors
///
/// Returns an error if an error occurs during variable substitution.
///
/// # Examples
///
/// ```
/// let line = "Hello, ${NAME:-User}! How are you, ${NAME}?";
/// let flags = Flags {
///     fail_on_empty: false,
///     fail_on_unset: false,
///     no_replace_empty: false,
///     no_replace_unset: false,
///     no_escape: false,
/// };
/// let filters = Filters {
///     prefix: None,
///     suffix: None,
///     variables: None,
/// };
///
/// let result = process_line(line, &flags, &filters);
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), "Hello, User! How are you, ?");
/// ```
fn process_line(line: &str, flags: &Flags, filters: &Filters) -> Result<String, String> {
    let mut new_line = String::with_capacity(line.len());
    let mut iter = line.chars().peekable();

    while let Some(c) = iter.next() {
        if c != '$' {
            new_line.push(c);
            continue;
        }

        let next_char = iter.peek();

        if !flags.no_escape && next_char == Some(&'$') {
            // if inside here, then we have a double $
            iter.next(); // skip the second $
            new_line.push(c);

            if iter.peek().is_none() {
                // double $ at the end of the line
                new_line.push(c);
                continue;
            }

            // if the next character is not a valid variable character, then push the second $
            if !iter
                .peek()
                .map(|c| c.is_ascii_alphabetic() || c == &'_')
                .unwrap_or(false)
            {
                new_line.push(c);
            }

            continue;
        }

        //let mut var_start = i + 1;
        //et mut var_end = var_start;
        let mut brace_ended = false;
        let mut original_variable: String = String::new();
        let mut var_name: String = String::new();
        let mut default_value: String = String::new();

        // match next character after the $
        match next_char {
            // Handles ${VAR} and ${VAR:-DEFAULT}
            Some('{') => {
                iter.next(); // skip the '{'

                // if next character is a number,
                // it is not a valid variable, eg. ${1VAR} or ${1VAR:-DEFAULT}
                if let Some(next) = iter.peek().filter(|next| next.is_ascii_digit()) {
                    // append ${ and the number ($ and { are skipped)
                    new_line.push_str(&format!("${{{}", next));
                    iter.next(); // skip the number
                    continue;
                }

                let mut default_value_found = false;
                let mut default_error = false;

                while let Some(c) = iter.next() {
                    // check if possible default value => :
                    if c == ':' {
                        // if next character is not '-', then the ':' is not part of the default value
                        if iter.peek() != Some(&'-') {
                            // if reached here, then the ':' is not part of the default value
                            default_error = true;
                            break;
                        }
                        default_value_found = true;
                        iter.next(); // skip the '-'
                        continue;
                    }

                    if c == '}' {
                        brace_ended = true;
                        break;
                    }

                    if default_value_found {
                        default_value.push(c);
                        continue;
                    }

                    var_name.push(c); // append the "regular" character to the variable name
                }

                if default_error {
                    // this only occurs if the ':' is not part of the default value

                    // append everything that was iterated over
                    new_line.push_str(&format!("${{{}", var_name));

                    // append found default value
                    if default_value_found {
                        new_line.push_str(&format!(":-{}", default_value));
                    }

                    // append the "broken" :
                    new_line.push(':');
                    continue; // continue to the next character
                }

                if !brace_ended {
                    // append everything that was iterated over
                    new_line.push_str(&format!("${{{}", var_name));
                    if default_value_found {
                        new_line.push_str(&format!(":-{}", default_value));
                    }
                    continue;
                }

                original_variable.push_str(&format!("${{{}", var_name));

                if default_value_found {
                    original_variable.push_str(&format!(":-{}", default_value));
                }
                original_variable.push('}');
            }
            // Handles $VAR and $VAR
            Some(next) if next.is_ascii_alphabetic() || next == &'_' => {
                // look ahead to see if the next character is valid
                // peek does not consume the character
                // if the character ahead is valid, it will be consumed with iter.next()
                while let Some(c) = iter.peek() {
                    if !c.is_ascii_alphanumeric() && c != &'_' {
                        break;
                    }
                    var_name.push(*c);
                    iter.next(); // consume character
                }
                original_variable = format!("${}", var_name);
            }
            // Everything else
            _ => {
                new_line.push(c);
                continue;
            }
        }

        if matches_filters(filters, &var_name) == Some(false) {
            new_line.push_str(&original_variable);
            continue;
        }

        match get_env_var_value(&var_name, &original_variable, &default_value, flags) {
            Ok(val) => new_line.push_str(&val),
            Err(err) => return Err(err),
        }
    }

    Ok(new_line)
}

/// Processes an input file and performs variable substitution according to the specified `Flags` and `Filters`.
///
/// # Arguments
///
/// * `input_file` - A boxed input file stream (e.g. `std::fs::File`) containing the text to process.
/// * `output_file` - A boxed output file stream (e.g. `std::fs::File` or `std::io::stdout()`) to write the processed text to.
/// * `flags` - A reference to a `Flags` object containing additional behavior settings.
/// * `filters` - A reference to a `Filters` object containing optional filter criteria for variable names.
///
/// # Returns
///
/// Returns `Ok(())` if the processing is successful, or an `Err` containing an error message if an error occurs during variable substitution or writing to the output file.
///
/// # Errors
///
/// Returns an error if an error occurs during variable substitution or writing to the output file.
///
/// # Examples
///
/// ```
/// use std::io::{Read, Write};
///
/// let content = "Hello, ${NAME:-User}! How are you, ${NAME}?";
/// let flags = Flags {
///     fail_on_empty: false,
///     fail_on_unset: false,
///     no_replace_empty: false,
///     no_replace_unset: false,
///     no_escape: false,
/// };
/// let filters = Filters {
///     prefix: None,
///     suffix: None,
///     variables: None,
/// };
///
/// //convert line to match input_file from perform_substitution
/// let line = Box::new(content.as_bytes());
///
/// let result = perform_substitution(line, Box::new(std::io::stdout()), &flags, &filters);
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), ());

/// ```
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
    fn test_process_line_regular_var_found_long_value() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        env::set_var(
            "REGULAR_VAR_LONG_FOUND",
            "valuevaluevaluevaluevaluevaluevaluevaluevaluevaluevalue",
        );
        let line = "$REGULAR_VAR_LONG_FOUND".to_string();
        let result = process_line(&line, &EMPTY_FLAGS, &EMPTY_FILTERS);
        assert_eq!(
            result,
            Ok("valuevaluevaluevaluevaluevaluevaluevaluevaluevaluevalue".to_string())
        );
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
        // description: escape variable, double dollar, no escape
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
        // description: double dollar sign at the end of line, no escape true
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
            original_var,
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
            original_var,
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
            original_var,
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
            original_var,
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
            original_var,
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
            original_var,
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
            original_var,
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
            original_var,
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

    #[test]
    fn test_example_process_line() {
        let line = "Hello, ${NAME:-User}! How are you, ${NAME}?";
        let flags = Flags {
            fail_on_empty: false,
            fail_on_unset: false,
            no_replace_empty: false,
            no_replace_unset: false,
            no_escape: false,
        };
        let filters = Filters {
            prefix: None,
            suffix: None,
            variables: None,
        };

        let result = process_line(line, &flags, &filters);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, User! How are you, ?");
    }

    #[test]
    fn test_example_match_filters() {
        let filters = Filters {
            prefix: Some("prefixed_".to_string()),
            suffix: Some("_suffixed".to_string()),
            variables: Some(vec![
                "my_variable".to_string(),
                "another_variable".to_string(),
            ]),
        };

        assert_eq!(matches_filters(&filters, "prefixed_variable"), Some(true));
        assert_eq!(matches_filters(&filters, "variable_suffixed"), Some(true));
        assert_eq!(matches_filters(&filters, "my_variable"), Some(true));
        assert_eq!(matches_filters(&filters, "another_variable"), Some(true));
        assert_eq!(matches_filters(&filters, "your_variable"), Some(false));
    }

    #[test]
    fn test_example_get_env_var_value() {
        let var_name = "MY_ENV_VAR";
        let default_value = "default_value";
        let flags = Flags {
            fail_on_empty: false,
            fail_on_unset: false,
            no_replace_empty: false,
            no_replace_unset: false,
            no_escape: false,
        };

        let result = get_env_var_value(var_name, "", default_value, &flags);

        assert_eq!(result, Ok(default_value.to_string()));
    }

    #[test]
    fn test_example_perform_substitution() {
        let content = "Hello, ${NAME:-User}! How are you, ${NAME}?";
        let flags = Flags {
            fail_on_empty: false,
            fail_on_unset: false,
            no_replace_empty: false,
            no_replace_unset: false,
            no_escape: false,
        };
        let filters = Filters {
            prefix: None,
            suffix: None,
            variables: None,
        };

        // convert line to match input_file from perform_substitution
        let line = Box::new(content.as_bytes());

        let result = perform_substitution(line, Box::new(std::io::stdout()), &flags, &filters);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }
}
