use crate::filters::Filters;
use crate::flags::{Flag, Flags};
use crate::utils::{colorize_text, handle_flags_on_result};
use colored::Color;
use std::env;

/// Processes pattern stripping operations on the given `value` string, based on the provided `op` character and the pattern in `operation_data`.
///
/// # Arguments
///
/// * `op` - The operation character: '#' for prefix stripping and '%' for suffix stripping.
/// * `value` - The input string on which the operation is performed.
/// * `operation_data` - The pattern wrapped in an `Option`. If `None`, the original value is returned.
/// * `inner_expr` - The inner expression string to use in error messages.
/// * `colored` - A boolean flag that determines whether the output string should be colorized based on the operation success.
///
/// # Returns
///
/// * `Ok(String)` - The processed string after the pattern stripping operation is applied. If `colored` is true, the text will be colorized based on the operation success.
/// * `Err(String)` - An error string in the case of an invalid operation character: "{`inner_expr`} - Invalid operation: '{`op`}'"
///
/// # Errors
///
/// * "Invalid operation: '{op}'" - If the `op` character is not '#' or '%'.
fn process_pattern_stripping(
    op: char,
    value: &str,
    operation_data: Option<&str>,
    inner_expr: &str,
    colored: bool,
) -> Result<String, String> {
    // If value is empty, return the original value
    if value.is_empty() {
        return Ok(value.to_string());
    }

    match operation_data {
        Some(pattern) => {
            let (new_value, color) = match op {
                // If the operation is '#', remove the shortest matching prefix (if any) from the value
                '#' if value.starts_with(pattern) => {
                    (value.strip_prefix(pattern).unwrap_or(value), Color::Blue)
                }
                // If the operation is '%', remove the shortest matching suffix (if any) from the value
                '%' if value.ends_with(pattern) => {
                    (value.strip_suffix(pattern).unwrap_or(value), Color::Blue)
                }
                // This is the case where the value does not start nor end with the pattern
                '%' | '#' => (value, Color::Red),
                // If the operation is invalid, return an error
                _ => return Err(format!("\"{inner_expr}\" - Invalid operation: '{op}'")),
            };

            return Ok(colorize_text(colored, new_value.to_string(), color));
        }
        None => return Ok(value.to_string()),
    }
}

/// Processes pattern replacement operations on the given `value` string, based on the provided pattern and replacement in `operation_data`.
///
/// # Arguments
///
/// * `value` - The input string on which the operation is performed.
/// * `operation_data` - The pattern and replacement string wrapped in an `Option`. If `None`, the original value is returned.
/// * `inner_expr` - The inner expression string to use in error messages.
/// * `colored` - A boolean flag that determines whether the output string should be colorized based on the operation success.
///
/// # Returns
///
/// * `Ok(String)` - The processed string after the pattern replacement operation is applied. If `colored` is true, the text will be colorized based on the operation success.
/// * `Err(String)` - An error string in the case of not being able to split the pattern and replacement: "{`inner_expr`} - Cannot split at '/'"
fn process_pattern_replacement(
    value: &str,
    operation_data: Option<&str>,
    inner_expr: &str,
    colored: bool,
) -> Result<String, String> {
    // If value is empty, return the original value
    if value.is_empty() {
        return Ok(value.to_string());
    }

    match operation_data {
        Some(replace_data) => {
            // Split the pattern and replacement using the '/' character as a separator
            let mut parts = replace_data.splitn(2, '/');
            // Throw an error if cannot split
            let pattern = parts
                .next()
                .ok_or(format!("\"{inner_expr}\" - Cannot split at '/'"))?;
            let replacement = parts
                .next()
                .ok_or(format!("\"{inner_expr}\" - Cannot split at '/'"))?;

            let color = if value.contains(pattern) {
                Color::Blue
            } else {
                Color::Red
            };

            // Replace all occurrences of the pattern with the replacement string
            return Ok(colorize_text(
                colored,
                value.replace(pattern, replacement),
                color,
            ));
        }
        None => return Ok(value.to_string()),
    }
}

/// Processes case conversion operations on the given `value` string, based on the provided operation character and `operation_data`.
///
/// # Arguments
///
/// * `op` - The operation character, either ',' or '^'.
/// * `value` - The input string on which the operation is performed.
/// * `operation_data` - Additional operation data wrapped in an `Option`. If `None`, the original value is returned.
/// * `inner_expr` - The inner expression string to use in error messages.
/// * `colored` - A boolean flag that determines whether the output string should be colorized based on the operation success.
///
/// # Returns
///
/// * `Ok(String)` - The processed string after the case conversion operation is applied. If `colored` is true, the text will be colorized based on the operation success.
/// * `Err(String)` - An error string in the case of an invalid operation: "{`inner_expr`} - Invalid operation: '{`op`}'"
fn process_case_conversion(
    op: char,
    value: &str,
    operation_data: Option<&str>,
    inner_expr: &str,
    colored: bool,
) -> Result<String, String> {
    // If value is empty, return the original value
    if value.is_empty() {
        return Ok(value.to_string());
    }

    let (new_value, color) = match (op, operation_data) {
        // If operation_data is None, return the original value
        (_, None) => (value.to_string(), Color::Magenta),
        // If operation_data is an empty string, convert the first character
        (',', Some("")) => {
            let lower = value[..1].to_ascii_lowercase();
            let color = if lower == value[..1] {
                Color::Red
            } else {
                Color::Blue
            };
            (lower + &value[1..], color)
        }
        ('^', Some("")) => {
            let upper = value[..1].to_ascii_uppercase();
            let color = if upper == value[..1] {
                Color::Red
            } else {
                Color::Blue
            };
            (upper + &value[1..], color)
        }
        // If conversion is ',', convert the value to lowercase
        (',', Some(",")) => {
            let lower = value.to_lowercase();
            let color = if lower == value {
                Color::Red
            } else {
                Color::Blue
            };
            (lower, color)
        }
        // If conversion is '^', convert the value to uppercase
        ('^', Some("^")) => {
            let upper = value.to_uppercase();
            let color = if upper == value {
                Color::Red
            } else {
                Color::Blue
            };
            (upper, color)
        }
        // If the operation is invalid, return an error
        _ => return Err(format!("\"{inner_expr}\" - Invalid operation: '{op}'")),
    };

    return Ok(colorize_text(colored, new_value, color));
}

/// Processes substring extraction operations on the given `value` string, based on the provided `operation_data`.
///
/// # Arguments
///
/// * `value` - The input string on which the operation is performed.
/// * `operation_data` - Additional operation data wrapped in an `Option`. If `None`, the original value is returned.
/// * `inner_expr` - The inner expression string to use in error messages.
/// * `colored` - A boolean flag that determines whether the output string should be colorized based on the operation success.
///
/// # Returns
///
/// * `Ok(String)` - The extracted substring from the input value, colorized if `colored` is true.
/// * `Err(String)` - An error string in the case of an invalid start offset or length.
fn process_substring_extraction(
    value: &str,
    operation_data: Option<&String>,
    inner_expr: &str,
    colored: bool,
) -> Result<String, String> {
    // If value is empty, return the original value
    if value.is_empty() {
        return Ok(value.to_string());
    }

    // If operation_data is None, return the original value
    if operation_data.is_none() {
        return Ok(value.to_string());
    }

    let operation_data = operation_data.unwrap();

    // Split operation_data using ':' to get start and len parts
    let mut parts = operation_data.splitn(2, ':');

    // Parse the first part (start) as a usize
    let start = parts
        .next()
        .ok_or(format!("\"${{{inner_expr}}}\" - invalid start offset"))?
        .parse::<usize>()
        .map_err(|_| format!("\"${{{inner_expr}}}\" - invalid start offset"))?;

    // Parse the second part (len) as an optional usize
    let len = parts
        .next()
        .map(str::parse)
        .transpose()
        .map_err(|_| format!("\"${{{inner_expr}}}\" - Invalid length"))?;

    // Extract the substring from value, skipping 'start' characters and taking 'len' characters
    return Ok(colorize_text(
        colored,
        value
            .chars()
            .skip(start)
            .take(len.unwrap_or(value.len() - start))
            .collect(),
        Color::Blue,
    ));
}

/// Processes the inner expression of a variable, applying the specified operations and flags.
///
/// # Arguments
///
/// * `inner_expr` - The inner expression string to process.
/// * `flags` - Flags that affect how the inner expression is processed.
/// * `filters` - Filters that determine which variables should be processed.
///
/// # Returns
///
/// * `Ok(String)` - The processed inner expression, taking into account the specified operations and flags.
/// * `Err(String)` - An error string in the case of an invalid character, operation, or flag handling error.
pub fn process_inner_expression(
    inner_expr: &str,
    flags: &Flags,
    filters: &Filters,
) -> Result<String, String> {
    let mut iter = inner_expr.chars().peekable();

    let mut var_name: String = String::new();
    let mut operation: Option<char> = None;
    let mut operation_data: Option<String> = None;
    let colored = flags.is_flag_set(Flag::Color);

    // Iterate through the characters of the inner expression
    while let Some(c) = iter.next() {
        if c.is_ascii_alphanumeric() || c == '_' {
            // If the character is alphanumeric or '_', add it to the var_name
            var_name.push(c);
            continue;
        }

        if c == '#' || c == '%' || c == '/' || c == ',' || c == ':' || c == '=' || c == '^' {
            // If an operation hasn't been found yet, and the current character is a valid operation, set the operation
            operation = Some(c);
            let mut data = String::new();
            for c in iter.by_ref() {
                data.push(c);
            }
            operation_data = Some(data);
            break;
        }
        return Err(format!("Invalid character in expression: {c}"));
    }

    // Get the environment variable value for the given var_name
    let value = env::var(&var_name).unwrap_or_default();

    // Perform the specified operation, if any, on the value
    let result = if let Some(op) = operation {
        match op {
            // Process '#' and '%' operations for pattern stripping
            '#' | '%' => process_pattern_stripping(
                op,
                &value,
                operation_data.as_deref(),
                inner_expr,
                colored,
            ),
            // Process '/' operation for pattern replacement
            '/' => {
                process_pattern_replacement(&value, operation_data.as_deref(), inner_expr, colored)
            }
            // Process ',' and '^' operations for case conversion
            ',' | '^' => {
                process_case_conversion(op, &value, operation_data.as_deref(), inner_expr, colored)
            }
            // Process ':' operation for default value or substring extraction
            ':' => {
                // check if next character is -
                if operation_data.as_ref().unwrap().starts_with('-') {
                    // extract default value
                    let default_value = operation_data.as_ref().unwrap().get(1..).unwrap();
                    // return default value if value is empty
                    if value.is_empty() {
                        Ok(colorize_text(
                            colored,
                            default_value.to_string(),
                            Color::Yellow,
                        ))
                    } else {
                        Ok(colorize_text(colored, value, Color::Green))
                    }
                } else {
                    // otherwise, process substring extraction
                    process_substring_extraction(
                        &value,
                        operation_data.as_ref(),
                        inner_expr,
                        colored,
                    )
                }
            }
            _ => return Err(format!("Invalid operation: {op}")),
        }
    } else {
        Ok(value)
    };

    let original_variable = format!("${{{var_name}}}");

    // Check if the variable name matches any filters
    if filters.matches(&var_name) == Some(false) {
        return Ok(colorize_text(colored, original_variable, Color::Magenta));
    }

    // Handle Fail, FailOnEmpty, FailOnUnset, NoReplace, NoReplaceUnset, and NoReplaceEmpty flags
    let result = handle_flags_on_result(result?, &var_name, &original_variable, flags)?;

    return Ok(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::filters::Filter;
    use colored::Colorize;

    #[test]
    fn test_process_inner_expression_invalid_operation() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");
        let result = process_inner_expression("TEST_VAR=", &flags, &filters);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid operation: =");
    }

    #[test]
    fn test_process_inner_expression_basic_variable_replacement() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");
        let result = process_inner_expression("TEST_VAR", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, world!");
    }

    #[test]
    fn test_process_inner_expression_invalid_character_expression() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");
        let result = process_inner_expression("TEST_VAR@", &flags, &filters);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid character in expression: @");
    }

    #[test]
    fn test_process_inner_expression_pattern_stripping_prefix() {
        let flags = Flags::default();
        let filters = Filters::default();
        let result = process_inner_expression("TEST_VAR#H", &flags, &filters);
        assert_eq!(result.unwrap(), "ello, world!");
    }

    #[test]
    fn test_process_inner_expression_pattern_stripping_suffix() {
        let flags = Flags::default();
        let filters = Filters::default();
        let result = process_inner_expression("TEST_VAR%d!", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, worl");
    }

    #[test]
    fn test_process_inner_expression_pattern_replacement() {
        let flags = Flags::default();
        let filters = Filters::default();
        let result = process_inner_expression("TEST_VAR/world/moon", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, moon!");
    }

    #[test]
    fn test_process_inner_expression_case_conversion_first_character_lowercase() {
        let flags = Flags::default();
        let filters = Filters::default();
        let result = process_inner_expression("TEST_VAR,", &flags, &filters);
        assert_eq!(result.unwrap(), "hello, world!");
    }

    #[test]
    fn test_process_inner_expression_case_conversion_first_character_uppercase() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "hello, world!");
        let result = process_inner_expression("TEST_VAR^", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, world!");
    }

    #[test]
    fn test_process_inner_expression_case_conversion_all_lowercase() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");
        let result = process_inner_expression("TEST_VAR,,", &flags, &filters);
        assert_eq!(result.unwrap(), "hello, world!");
    }

    #[test]
    fn test_process_inner_expression_case_conversion_all_uppercase() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");
        let result = process_inner_expression("TEST_VAR^^", &flags, &filters);
        assert_eq!(result.unwrap(), "HELLO, WORLD!");
    }

    #[test]
    fn test_process_inner_expression_default_value() {
        let flags = Flags::default();
        let filters = Filters::default();

        let result = process_inner_expression("UNSET_VAR:-default", &flags, &filters);
        assert_eq!(result.unwrap(), "default");
    }

    #[test]
    fn test_process_inner_expression_default_value_empty_string() {
        let flags = Flags::default();
        let filters = Filters::default();

        let result = process_inner_expression("UNSET_VAR:-", &flags, &filters);
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_process_inner_expression_substring_extraction() {
        let flags = Flags::default();
        let filters = Filters::default();

        let result = process_inner_expression("TEST_VAR:7:5", &flags, &filters);
        assert_eq!(result.unwrap(), "world");
    }

    #[test]
    fn test_process_inner_expression_substring_extraction_empty_result_fail_flag() {
        let mut flags = Flags::default();
        let filters = Filters::default();

        flags
            .set(Flag::Fail, "--fail", true)
            .expect("Failed to set Fail flag");
        let result = process_inner_expression("UNSET_VAR", &flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'UNSET_VAR' is not set"
        );
    }

    #[test]
    fn test_process_inner_expression_substring_extraction_empty_result_no_replace_flag() {
        let filters = Filters::default();

        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplace, "--no-replace", true)
            .expect("Failed to set NoReplace flag");
        let result = process_inner_expression("NOT_FOUND_VARIALBE", &flags, &filters);
        assert_eq!(result.unwrap(), "${NOT_FOUND_VARIALBE}");
    }

    #[test]
    fn test_process_inner_expression_substring_extraction_empty_result() {
        // Test return Ok(format!("${{{inner_expr}}}"));
        let mut filters = Filters::default();
        let flags = Flags::default();

        let filter_result = filters.add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter());
        assert!(filter_result.is_ok());
        let result = process_inner_expression("TEST_VAR", &flags, &filters);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "${TEST_VAR}");
    }

    #[test]
    fn test_process_pattern_stripping_empty_value() {
        // Test '#' operation with operation_data
        let op = '#';
        let value = "";
        let operation_data = Some("ex".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "");
    }
    #[test]
    fn test_process_pattern_stripping_prefix() {
        // Test '#' operation with operation_data
        let op = '#';
        let value = "example_value";
        let operation_data = Some("blub".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "example_value".red()));

        // Test '#' operation without operation_data
        let op = '#';
        let value = "example_value";
        let operation_data = None;
        let inner_expr = op;
        let result =
            process_pattern_stripping(op, value, operation_data, &inner_expr.to_string(), false);
        assert_eq!(result.unwrap(), "example_value");

        // Test '#' operation with operation_data not found
        let op = '#';
        let value = "example_value";
        let operation_data = Some("not_found".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "example_value".red()));
    }

    #[test]
    fn test_process_pattern_stripping_suffix() {
        // Test '%' operation with operation_data
        let op = '%';
        let value = "example_value";
        let operation_data = Some("_value".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "example".blue()));

        // Test '%' operation without operation_data
        let op = '%';
        let value = "example_value";
        let operation_data = None;
        let inner_expr = op;
        let result =
            process_pattern_stripping(op, value, operation_data, &inner_expr.to_string(), false);
        assert_eq!(result.unwrap(), "example_value");

        // Test '%' operation with operation_data not found
        let op = '%';
        let value = "example_value";
        let operation_data = Some("not_found".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "example_value".red()));
    }

    #[test]
    fn test_process_pattern_stripping_invalid_operation() {
        // Test unreachable operation
        let op = '!';
        let value = "example_value";
        let operation_data = Some("ex".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_stripping(op, value, operation_data.as_deref(), &inner_expr, false);
        assert!(result.is_err(), "Expected error from invalid operation");
    }

    #[test]
    fn test_process_pattern_replacement() {
        // Test pattern replacement with an empty replacement string
        let op = "/";
        let value = "hello_world";
        let operation_data = Some("o/".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_replacement(value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "hell_wrld");

        // Test pattern replacement pattern not found
        let op = "/";
        let value = "hello_world";
        let operation_data = Some("k/l".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_replacement(value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "hello_world".red()));

        // Test pattern replacement with an empty replacement string
        let op = "/";
        let value = "hello_world";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_replacement(value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(
            result.unwrap_err(),
            format!("\"{inner_expr}\" - Cannot split at '/'")
        );

        // Test pattern replacement with operation_data
        let op = "/";
        let value = "hello_world";
        let operation_data = Some("o/_".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_replacement(value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "hell__w_rld");

        // Test pattern replacement without operation_data
        let op = "/";
        let value = "hello_world";
        let operation_data = None;
        let inner_expr = op;
        let result =
            process_pattern_replacement(value, operation_data, inner_expr, false);
        assert_eq!(result.unwrap(), "hello_world");

        // Test empty value
        let op = "/";
        let value = "";
        let operation_data = Some("o/_".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_pattern_replacement(value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_process_case_conversion_first_upper() {
        // Test '^' operation with empty conversion
        let op = '^';
        let value = "helloWorld";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "HelloWorld");

        // Test '^' operation identical to first upper
        let op = '^';
        let value = "HelloWorld";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "HelloWorld".red()));
    }

    #[test]
    fn test_process_case_conversion_first_lower() {
        // Test ',' operation with empty conversion
        let op = ',';
        let value = "HelloWorld";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "helloWorld");

        // Test ',' operation identical to first lower
        let op = ',';
        let value = "helloWorld";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "helloWorld".red()));
    }

    #[test]
    fn test_process_case_conversion_all_upper() {
        // Test '^' operation with "^" conversion
        let op = '^';
        let value = "HelloWorld";
        let operation_data = Some("^".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "HELLOWORLD");

        // Test '^' operation with "^" conversion identical to all upper
        let op = '^';
        let value = "HELLOWORLD";
        let operation_data = Some("^".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "HELLOWORLD".red()));
    }

    #[test]
    fn test_process_case_conversion_all_lower() {
        // Test ',' operation with "," conversion
        let op = ',';
        let value = "HelloWorld";
        let operation_data = Some(",".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "helloworld");

        // Test ',' operation with "," conversion identical to all lower
        let op = ',';
        let value = "helloworld";
        let operation_data = Some(",".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, true);
        assert_eq!(result.unwrap(), format!("{}", "helloworld".red()));
    }

    #[test]
    fn test_process_case_conversion_invalid_conversion() {
        // Test ',' operation with invalid conversion
        let op = ',';
        let value = "HelloWorld";
        let operation_data = Some("invalid".to_string());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("\"{inner_expr}\" - Invalid operation: ','")
        );
    }

    #[test]
    fn test_process_case_conversion_no_operation() {
        // Test ',' operation without operation_data
        let op = ',';
        let value = "HELLOWORLD";
        let operation_data = None;
        let inner_expr = op;
        let result =
            process_case_conversion(op, value, operation_data, &inner_expr.to_string(), false);
        assert_eq!(result.unwrap(), "HELLOWORLD");

        // Test '^' operation without operation_data
        let value = "helloworld";
        let op = '^';
        let operation_data = None;
        let inner_expr = op;
        let result =
            process_case_conversion(op, value, operation_data, &inner_expr.to_string(), false);
        assert_eq!(result.unwrap(), "helloworld");
    }

    #[test]
    fn test_process_case_conversion_invalid_operation() {
        // Test '$' operation with invalid operation
        let op = '$';
        let value = "Hello World";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.clone().unwrap());
        let result =
            process_case_conversion('$', value, operation_data.as_deref(), &inner_expr, false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("\"{inner_expr}\" - Invalid operation: '$'")
        );
    }

    #[test]
    fn process_case_conversion_empty_value() {
        // Test ',' operation with empty value
        let op = ',';
        let value = "";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(',', value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "");

        // Test '^' operation with empty value
        let op = '^';
        let value = "";
        let operation_data = Some(String::new());
        let inner_expr = format!("{}{}", op, operation_data.as_deref().unwrap());
        let result =
            process_case_conversion(op, value, operation_data.as_deref(), &inner_expr, false);
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_process_substring_extraction_without_data() {
        // Test without operation_data
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = None;
        let result = process_substring_extraction(value, operation_data, inner_expr, false);
        assert_eq!(result.unwrap(), "HelloWorld");
    }

    #[test]
    fn test_process_substring_extraction_with_start_and_len() {
        // Test substring extraction with start and len
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("1:3".to_string());
        let result =
            process_substring_extraction(value, operation_data.as_ref(), inner_expr, false);
        assert_eq!(result.unwrap(), "ell");
    }

    #[test]
    fn test_process_substring_extraction_with_start_only() {
        // Test substring extraction with start only
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("2".to_string());
        let result =
            process_substring_extraction(value, operation_data.as_ref(), inner_expr, false);
        assert_eq!(result.unwrap(), "lloWorld");
    }

    #[test]
    fn test_process_substring_extraction_with_start_only_invalid() {
        // Test substring extraction with invalid start
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("Worl".to_string());
        let result =
            process_substring_extraction(value, operation_data.as_ref(), inner_expr, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_substring_extraction_with_len_only_invalid() {
        // Test substring extraction with invalid len
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("2:invalid".to_string());
        let result =
            process_substring_extraction(value, operation_data.as_ref(), inner_expr, false);
        assert!(result.is_err());
    }

    #[test]
    fn process_substring_extraction_empty_value() {
        // Test substring extraction with empty value
        let value = "";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("1:3".to_string());
        let result =
            process_substring_extraction(value, operation_data.as_ref(), inner_expr, false);
        assert_eq!(result.unwrap(), "");
    }
}
