use crate::filters::Filters;
use crate::flags::{Flag, Flags};
use crate::utils::colorize_text;
use colored::Color;
use std::env;
use std::io::{BufRead, BufReader};

/// Reads lines from a buffered reader and returns an iterator over the lines.
///
/// This function takes a mutable reference to an object implementing the `BufRead` trait,
/// and creates an iterator that yields lines as `std::io::Result<String>`.
///
/// # Arguments
///
/// * `input` - A mutable reference to an object implementing the `BufRead` trait.
///
/// # Returns
///
/// * `impl Iterator<Item = std::io::Result<String>>` - An iterator over the lines read from the input.
///   Each item in the iterator is a `std::io::Result<String>`, where the `Ok` variant contains a line,
///   and the `Err` variant contains an error that occurred while reading.
///
fn read_lines(mut input: impl BufRead) -> impl Iterator<Item = std::io::Result<String>> {
    std::iter::from_fn(move || {
        let mut vec = String::new();
        match input.read_line(&mut vec) {
            Ok(0) => return None, // Reached the end of the input
            Ok(_) => {
                // Use std::mem::take to replace the vec with an empty String
                // and return the original vec as the result
                return Some(Ok(std::mem::take(&mut vec)));
            }
            Err(e) => return Some(Err(e)), // Return any errors that occur while reading
        }
    })
}

/// Processes pattern stripping operations for the given value and pattern.
///
/// This function takes an operation (`op`), a value, and an optional pattern to apply the pattern
/// stripping operation. The supported operations are:
/// * '#' - Remove the shortest matching prefix (if any) from the value.
/// * '%' - Remove the shortest matching suffix (if any) from the value.
///
/// # Arguments
///
/// * `op` - A character representing the pattern stripping operation ('#' or '%').
/// * `value` - A string slice containing the value on which the operation should be performed.
/// * `operation_data` - An `Option<&String>` containing the pattern for the stripping operation.
///   If the pattern is `None`, the value will be returned unchanged.
///
/// # Returns
///
/// * `Result<String, String>` - A `Result` containing either the processed value as a `String` on success
///   or an error message as a `String` on failure.
///
/// # Errors
///
/// This function will return an error if an invalid operation is provided.
///
fn process_pattern_stripping(
    op: char,
    value: &str,
    operation_data: Option<&String>,
) -> Result<String, String> {
    return operation_data.map_or_else(
        // If operation_data is None, return the value unchanged
        || Ok(value.to_string()),
        |pattern| match op {
            // If the operation is '#', remove the shortest matching prefix (if any) from the value
            '#' => Ok(value.strip_prefix(pattern).unwrap_or(value).to_string()),
            // If the operation is '%', remove the shortest matching suffix (if any) from the value
            '%' => Ok(value.strip_suffix(pattern).unwrap_or(value).to_string()),
            // If the operation is invalid, return an error
            _ => Err(format!("Invalid operation: {op}")),
        },
    );
}

/// Processes pattern replacement operations for the given value and pattern.
///
/// This function takes a value and an optional pattern to apply the pattern replacement operation.
/// If the pattern is provided, the function replaces the first occurrence of the pattern in the value
/// with the replacement string, if provided. If no pattern or replacement is provided, the function
/// returns the original value.
///
/// The pattern and replacement strings are separated by a forward slash '/' character.
///
/// # Arguments
///
/// * `value` - A string slice containing the value on which the operation should be performed.
/// * `operation_data` - An `Option<&String>` containing the pattern and replacement for the replacement
///   operation. If the pattern is `None`, the value will be returned unchanged.
///
/// # Returns
///
/// * `Result<String, String>` - A `Result` containing either the processed value as a `String` on success
///   or an error message as a `String` on failure.
///
fn process_pattern_replacement(
    value: &str,
    operation_data: Option<&String>,
) -> Result<String, String> {
    return operation_data.map_or_else(
        // If operation_data is None, return the value unchanged
        || Ok(value.to_string()),
        // Otherwise, replace the pattern with the replacement string
        |replace_data| {
            // Split the pattern and replacement using the '/' character as a separator
            let mut parts = replace_data.splitn(2, '/');
            let pattern = parts.next().unwrap();
            let replacement = parts.next().unwrap_or("");

            // Replace all occurrences of the pattern with the replacement string
            Ok(value.replace(pattern, replacement))
        },
    );
}

/// Processes case conversion operations for the given value and conversion data.
///
/// This function takes an operation (`op`), a value, and an optional string for the case conversion
/// operation. The supported operations are:
/// * ',' - Convert the first character of the value to lowercase.
/// * '^' - Convert the first character of the value to uppercase.
///
/// If no conversion data is provided, the first character of the value will be converted to uppercase
/// by default.
///
/// # Arguments
///
/// * `op` - A character representing the case conversion operation (',' or '^').
/// * `value` - A string slice containing the value on which the operation should be performed.
/// * `operation_data` - An `Option<&String>` containing the conversion data for the operation.
///   If the operation data is `None`, the function will return the original value.
///
/// # Returns
///
/// * `Result<String, String>` - A `Result` containing either the processed value as a `String` on success
///   or an error message as a `String` on failure.
///
/// # Errors
///
/// This function will return an error if an invalid conversion is provided.
///
fn process_case_conversion(
    op: char,
    value: &str,
    operation_data: Option<&String>,
) -> Result<String, String> {
    return operation_data.map_or_else(
        // If operation_data is None, return the original value
        || Ok(value.to_string()),
        |conversion| match conversion.as_str() {
            // If conversion is empty, convert the first character to uppercase or lowercase
            "" => {
                if op == ',' {
                    Ok(value[..1].to_ascii_lowercase() + &value[1..])
                } else {
                    Ok(value[..1].to_ascii_uppercase() + &value[1..])
                }
            }
            // If conversion is ',', convert the value to lowercase
            "," => Ok(value.to_lowercase()),
            // If conversion is '^', convert the value to uppercase
            "^" => Ok(value.to_ascii_uppercase()),
            // If the conversion is invalid, return an error
            _ => Err(format!("Invalid conversion: {conversion}")),
        },
    );
}

/// Processes substring extraction operations for the given value and operation data.
///
/// This function takes a value, an optional string for the substring extraction operation data,
/// and the inner expression that the operation data came from. The operation data should be in the
/// format "start:len", where "start" is the starting index of the substring and "len" is the length
/// of the substring to extract. If "len" is omitted, the function will extract the substring from
/// "start" to the end of the value.
///
/// # Arguments
///
/// * `value` - A string slice containing the value on which the operation should be performed.
/// * `operation_data` - An `Option<&String>` containing the operation data for the substring extraction.
///   If the operation data is `None`, the function will return the original value.
/// * `inner_expr` - A string slice containing the inner expression that the operation data came from.
///
/// # Returns
///
/// * `Result<String, String>` - A `Result` containing either the extracted substring as a `String` on success
///   or an error message as a `String` on failure.
///
/// # Errors
///
/// This function will return an error if the start index or length is invalid.
///
fn process_substring_extraction(
    value: &str,
    operation_data: Option<&String>,
    inner_expr: &str,
) -> Result<String, String> {
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
        .unwrap()
        .parse::<usize>()
        .map_err(|_| format!("\"${{{inner_expr}}}\" - invalid start offset"))?;

    // Parse the second part (len) as an optional usize
    let len = parts
        .next()
        .map(str::parse)
        .transpose()
        .map_err(|_| "Invalid length")?;

    // Extract the substring from value, skipping 'start' characters and taking 'len' characters
    return Ok(value
        .chars()
        .skip(start)
        .take(len.unwrap_or(value.len() - start))
        .collect());
}

/// This function handles the application of certain flags on the result of a variable replacement. These
/// flags include `Fail`, `FailOnEmpty`, `FailOnUnset`, `NoReplaceUnset`, and `NoReplaceEmpty`. The function takes in
/// the result of a variable replacement, the variable name, the original variable string, and the flags
/// object. It then returns the modified result or an error message depending on the flag settings.
///
/// # Arguments
///
/// * `result` - A String containing the result of the variable replacement.
/// * `var_name` - A string slice containing the name of the environment variable being replaced.
/// * `original_variable` - A string slice containing the original variable string that was replaced.
/// * `flags` - A &Flags object containing the flag settings for the program.
///
/// # Returns
///
/// * Result<String, String> - A Result containing either the modified result as a String on success
/// or an error message as a String on failure.
///
/// # Errors
///
/// This function will return an error message if the `Fail` or `FailOnEmpty` flags are set and the result is empty,
/// if the `FailOnUnset` flag is set and the environment variable is not set, or if the `NoReplaceUnset` or `NoReplaceEmpty`
/// flags are set and the result is empty.
fn handle_flags_on_result(
    result: String,
    var_name: &str,
    original_variable: &str,
    flags: &Flags,
) -> Result<String, String> {
    // Check for Fail and FailOnUnset flags
    if result.is_empty() && (flags.is_flag_set(Flag::Fail) || flags.is_flag_set(Flag::FailOnUnset))
    {
        return Err(format!("environment variable '{var_name}' is not set"));
    }

    // Check for FailOnEmpty flag
    if result.is_empty() && (flags.is_flag_set(Flag::Fail) || flags.is_flag_set(Flag::FailOnEmpty))
    {
        return Err(format!("environment variable '{var_name}' is empty"));
    }

    // Check for NoReplaceUnset flag
    if result.is_empty()
        && (flags.is_flag_set(Flag::NoReplace) || flags.is_flag_set(Flag::NoReplaceUnset))
    {
        return Ok(original_variable.to_string());
    }

    // Check for NoReplaceEmpty flag
    if result.is_empty()
        && (flags.is_flag_set(Flag::NoReplace) || flags.is_flag_set(Flag::NoReplaceEmpty))
    {
        return Ok(original_variable.to_string());
    }

    // Return the modified result
    return Ok(result);
}

/// Processes an inner expression by parsing the variable name, any associated operation, and
/// performing the specified operation on the corresponding environment variable. This function
/// supports the following operations:
///
/// * `#` or `%` for pattern stripping
/// * `/` for pattern replacement
/// * `,` or `^` for case conversion
/// * `:` for default value or substring extraction
///
/// # Arguments
///
/// * `inner_expr` - A string slice containing the inner expression to be processed.
/// * `flags` - A `Flags` struct containing the flags to apply to the operation.
/// * `filters` - A `Filters` struct containing the filters to apply to the variable name.
///
/// # Returns
///
/// * `Result<String, String>` - A `Result` containing either the result of the operation as a `String` on success
///   or an error message as a `String` on failure.
///
/// # Errors
///
/// This function will return an error if the inner expression contains an invalid character, or if an
/// operation is specified but the corresponding environment variable does not exist or is not valid.
///
fn process_inner_expression(
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
            '#' | '%' => process_pattern_stripping(op, &value, operation_data.as_ref()),
            // Process '/' operation for pattern replacement
            '/' => process_pattern_replacement(&value, operation_data.as_ref()),
            // Process ',' and '^' operations for case conversion
            ',' | '^' => process_case_conversion(op, &value, operation_data.as_ref()),
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
                    process_substring_extraction(&value, operation_data.as_ref(), inner_expr)
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

    return Ok(colorize_text(colored, result, Color::Green));
}

/// Replaces variables in a given line of text.
///
/// This function replaces variables in a line of text with their corresponding values. Variables can be
/// of the form $VAR, ${VAR}, or ${VAR:-DEFAULT}. The function supports several flags for controlling
/// the behavior of variable replacement.
///
/// # Arguments
///
/// * line - A string slice containing the line of text to replace variables in.
/// * flags - A reference to a Flags object containing the flags to use during variable replacement.
/// * filters - A reference to a Filters object containing the filters to apply during variable replacement.
///
/// # Returns
///
/// * Result<String, String> - A Result containing the modified line of text as a String on success
/// or an error message as a String on failure.
///
/// # Errors
///
/// This function will return an error if a variable name is invalid or if an invalid flag is encountered.
///
fn replace_vars_in_line(line: &str, flags: &Flags, filters: &Filters) -> Result<String, String> {
    let mut new_line: String = String::with_capacity(line.len());
    let mut iter = line.chars().peekable();
    let colored = flags.is_flag_set(Flag::Color);

    let check_escape = !flags.is_flag_set(Flag::NoEscape);

    while let Some(c) = iter.next() {
        if c != '$' {
            new_line.push(c);
            continue;
        }

        let next_char = iter.peek();

        // Check if the current character is an escaped '$' and the next character is also '$'
        if check_escape && c == '$' && next_char == Some(&'$') {
            // Consume the next character (the second '$') in the iterator
            iter.next();

            // Add the escaped '$' to the new_line buffer
            new_line.push('$');

            // Check the character after the escaped '$'
            match iter.peek() {
                // If the next character doesn't match any of the specified cases and is not an alphabetic character
                Some(next)
                    if !matches!(*next, '_' | '$' | ' ' | '{') && !next.is_ascii_alphabetic() =>
                {
                    // Add the current character to the new_line buffer
                    new_line.push(c);
                }
                // If there is no next character, add another '$' to the new_line buffer
                None => new_line.push('$'),
                _ => (), // In any other case, do nothing
            }
            continue;
        }
        // match next character after the $
        match next_char {
            // Handles ${VAR} and ${VAR:-DEFAULT}
            Some('{') => {
                iter.next(); // skip the '{'

                // if next character is a number,
                // it is not a valid variable, eg. ${1VAR} or ${1VAR:-DEFAULT}
                if let Some(next) = iter.peek().filter(|next| next.is_ascii_digit()) {
                    // append ${ and the number ($ and { are skipped)
                    new_line.push_str(&format!("${{{next}"));
                    iter.next(); // skip the number
                    continue;
                }

                let mut brace_ended = false;
                let mut inner_expr: String = String::new();

                // read until the next '}' or the end of the line
                while let Some(&c) = iter.peek() {
                    if c == '}' {
                        iter.next(); // Consume '}'
                        brace_ended = true;
                        break;
                    }
                    inner_expr.push(c);
                    iter.next();
                }

                if !brace_ended {
                    // if the brace hasn't ended, add the characters and continue
                    new_line.push_str(&format!("${{{inner_expr}"));
                    continue;
                }

                // Process inner expression here
                let value = process_inner_expression(&inner_expr, flags, filters)?;

                new_line.push_str(&value);
            }

            // Handles $VAR and $VAR
            Some(next) if next.is_ascii_alphabetic() || next == &'_' => {
                let mut var_name: String = String::new();

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
                let original_variable = format!("${var_name}");

                if filters.matches(&var_name) == Some(false) {
                    new_line.push_str(&colorize_text(colored, original_variable, Color::Yellow));
                    continue;
                }

                let value: String = env::var(&var_name).unwrap_or_default();
                let result = handle_flags_on_result(value, &var_name, &original_variable, flags)?;

                new_line.push_str(&colorize_text(colored, result, Color::Green));
            }
            // Everything else
            _ => {
                new_line.push(c);
                continue;
            }
        }
    }

    return Ok(new_line);
}
/// Processes the input from a given Read instance and writes the result to a given Write instance.
///
/// This function reads input from a Read instance and replaces any variables found in the input
/// with their corresponding environment variable values, according to the specified Flags and Filters.
/// The resulting output is then written to a Write instance.
///
/// # Arguments
///
/// * input - A Read instance from which to read input data.
/// * output - A Write instance to which to write the resulting output data.
/// * flags - A reference to a Flags instance containing the flag settings for variable replacement.
/// * filters - A reference to a Filters instance containing the variable name filters for variable replacement.
///
/// # Returns
///
/// * Result<(), String> - A Result containing either a () on success or an error message as a String on failure.
///
/// # Errors
///
/// This function will return an error if any I/O operation fails, or if there is an error in replacing variables in the input data.
///
pub fn process_input<R: std::io::Read, W: std::io::Write>(
    input: R,
    mut output: W,
    flags: &Flags,
    filters: &Filters,
) -> Result<(), String> {
    let reader: BufReader<R> = BufReader::new(input);
    let mut buffer = String::new();
    let unbuffered_lines = flags
        .get(Flag::UnbufferedLines)
        .map_or(false, |f| f.value.unwrap_or(false));

    for line_res in read_lines(reader) {
        let line = line_res.map_err(|e| e.to_string())?;
        let replaced: Result<String, String> = replace_vars_in_line(&line, flags, filters);
        match replaced {
            Ok(out) => {
                // if unbuffered lines mode is enabled, write each line as soon as it's processed
                if unbuffered_lines {
                    if let Err(e) = output.write(out.as_bytes()) {
                        return Err(format!("failed to write to output: {e}"));
                    }
                    continue;
                }
                // if unbuffered lines mode is not enabled, append the line to the buffer
                buffer.push_str(&out);
            }
            Err(e) => return Err(format!("failed to replace variables: {e}")),
        }
    }

    // if unbuffered lines mode is not enabled, write the entire buffer to the output file at once
    if !unbuffered_lines {
        if let Err(e) = output.write_all(buffer.as_bytes()) {
            return Err(format!("failed to write to output: {e}"));
        }
    }

    // flush the output to ensure that all written data is actually written to the output stream
    if let Err(e) = output.flush() {
        return Err(format!("failed to flush output: {e}"));
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::Filter;
    use std::io::Cursor;

    // A dummy implementation of a writer that always fails to write and flush
    struct FailingWriter<'a> {
        writer: &'a mut dyn std::io::Write,
        fail_on_write: bool,
        fail_on_flush: bool,
    }

    impl<'a> FailingWriter<'a> {
        fn new(
            writer: &'a mut dyn std::io::Write,
            fail_on_write: bool,
            fail_on_flush: bool,
        ) -> FailingWriter<'a> {
            FailingWriter {
                writer,
                fail_on_write,
                fail_on_flush,
            }
        }
    }

    impl<'a> std::io::Write for FailingWriter<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.fail_on_write {
                self.fail_on_write = false;
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated write error",
                ))
            } else {
                self.writer.write(buf)
            }
        }

        fn flush(&mut self) -> std::io::Result<()> {
            if self.fail_on_flush {
                self.fail_on_flush = false;
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated flush error",
                ))
            } else {
                self.writer.flush()
            }
        }
    }

    #[test]
    fn test_read_lines() {
        // Test Unix line endings
        let input = "Hello $WORLD!\nHello $WORLD!\nHello $WORLD!\n";
        let expected = vec!["Hello $WORLD!\n", "Hello $WORLD!\n", "Hello $WORLD!\n"];
        let lines = read_lines(input.as_bytes());
        for (i, line_result) in lines.enumerate() {
            let line = line_result.unwrap();
            assert_eq!(line, expected[i]);
        }

        // Test Windows line endings
        let input = "Hello $WORLD!\r\nHello $WORLD!\r\nHello $WORLD!\r\n";
        let expected = vec![
            "Hello $WORLD!\r\n",
            "Hello $WORLD!\r\n",
            "Hello $WORLD!\r\n",
        ];
        let lines = read_lines(input.as_bytes());
        for (i, line_result) in lines.enumerate() {
            let line = line_result.unwrap();
            assert_eq!(line, expected[i]);
        }

        // Test Emojis
        let input = "Hello 😃 World!\nHello 😃 World!\nHello 😃 World!\n";
        let expected = vec![
            "Hello 😃 World!\n",
            "Hello 😃 World!\n",
            "Hello 😃 World!\n",
        ];
        let lines = read_lines(input.as_bytes());
        for (i, line_result) in lines.enumerate() {
            let line = line_result.unwrap();
            assert_eq!(line, expected[i]);
        }
    }

    #[test]
    fn test_read_lines_error() {
        let input = b"Hello \xF0 World!";
        let mut lines = read_lines(Cursor::new(&input[..]));
        let result = lines.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_process_pattern_stripping() {
        // Test '#' operation with operation_data
        let op = '#';
        let value = "example_value";
        let operation_data = Some("ex".to_string());
        let result = process_pattern_stripping(op, value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "ample_value");

        // Test '#' operation without operation_data
        let op = '#';
        let value = "example_value";
        let operation_data = None;
        let result = process_pattern_stripping(op, value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "example_value");

        // Test '%' operation with operation_data
        let op = '%';
        let value = "example_value";
        let operation_data = Some("_value".to_string());
        let result = process_pattern_stripping(op, value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "example");

        // Test '%' operation without operation_data
        let op = '%';
        let value = "example_value";
        let operation_data = None;
        let result = process_pattern_stripping(op, value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "example_value");

        // Test unreachable operation
        let op = '!';
        let value = "example_value";
        let operation_data = Some("ex".to_string());
        let result = process_pattern_stripping(op, value, operation_data.as_ref());
        assert!(result.is_err(), "Expected error from invalid operation");

    }

    #[test]
    fn test_process_pattern_replacement() {
        let value = "hello_world";

        // Test pattern replacement with operation_data
        let operation_data = Some("o/_".to_string());
        let result = process_pattern_replacement(value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "hell__w_rld");

        // Test pattern replacement with an empty replacement string
        let operation_data = Some("o/".to_string());
        let result = process_pattern_replacement(value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "hell_wrld");

        // Test pattern replacement without operation_data
        let operation_data = None;
        let result = process_pattern_replacement(value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "hello_world");
    }

    #[test]
    fn test_process_case_conversion() {
        // Test ',' operation with empty conversion
        let value = "HelloWorld";
        let operation_data = Some(String::new());
        let result = process_case_conversion(',', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "helloWorld");

        // Test '^' operation with empty conversion
        let value = "helloWorld";
        let operation_data = Some(String::new());
        let result = process_case_conversion('^', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "HelloWorld");

        // Test ',' operation with "," conversion
        let value = "HelloWorld";
        let operation_data = Some(",".to_string());
        let result = process_case_conversion(',', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "helloworld");

        // Test '^' operation with "^" conversion
        let value = "HelloWorld";
        let operation_data = Some("^".to_string());
        let result = process_case_conversion('^', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "HELLOWORLD");

        // Test ',' operation with invalid conversion
        let value = "HelloWorld";
        let operation_data = Some("invalid".to_string());
        let result = process_case_conversion(',', value, operation_data.as_ref());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid conversion: invalid"
        );

        // Test ',' operation without operation_data
        let value = "HELLOWORLD";
        let operation_data = None;
        let result = process_case_conversion(',', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "HELLOWORLD");

        // Test '^' operation without operation_data
        let value = "helloworld";
        let operation_data = None;
        let result = process_case_conversion('^', value, operation_data.as_ref());
        assert_eq!(result.unwrap(), "helloworld");
    }

    #[test]
    fn test_process_substring_extraction() {
        // Test without operation_data
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = None;
        let result = process_substring_extraction(value, operation_data, inner_expr);
        assert_eq!(result.unwrap(), "HelloWorld");

        // Test substring extraction with start and len
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("1:3".to_string());
        let result = process_substring_extraction(value, operation_data.as_ref(), inner_expr);
        assert_eq!(result.unwrap(), "ell");

        // Test substring extraction with start only
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("2".to_string());
        let result = process_substring_extraction(value, operation_data.as_ref(), inner_expr);
        assert_eq!(result.unwrap(), "lloWorld");

        // Test substring extraction with invalid start
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("Worl".to_string());
        let result = process_substring_extraction(value, operation_data.as_ref(), inner_expr);
        assert!(result.is_err());

        // Test substring extraction with invalid len
        let value = "HelloWorld";
        let inner_expr = "VAR:1:3";
        let operation_data = Some("2:invalid".to_string());
        let result = process_substring_extraction(value, operation_data.as_ref(), inner_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_flags_on_result() {
        let var_name = "VAR";
        let inner_expr = "VAR";
        let mut flags = Flags::default();
        let original_variable = "$VAR".to_string();

        // Test without any flags
        let result =
            handle_flags_on_result("Hello".to_string(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), "Hello");

        // Test with Fail flag and non-empty result
        flags
            .set(Flag::Fail, "--fail", true)
            .expect("Failed to set Fail flag");
        let result =
            handle_flags_on_result("Hello".to_string(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), "Hello");

        // Test with Fail flag and empty result
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert!(result.is_err());

        // Test with FailOnEmpty flag and empty result
        flags = Flags::default();
        flags
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .expect("Failed to set FailOnEmpty flag");

        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert!(result.is_err());

        // Test with FailOnUnset flag and unset variable
        flags = Flags::default();
        flags
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .expect("Failed to set FailOnUnset flag");
        let result = handle_flags_on_result(String::new(), "UNSET_VAR", &original_variable, &flags);
        assert!(result.is_err());

        // Test with NoReplace flag
        flags = Flags::default();
        flags
            .set(Flag::NoReplace, "--no-replace", true)
            .expect("Failed to set NoReplace flag");
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));

        // Test with NoReplaceEmpty flag and empty result
        flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .expect("Failed to set NoReplaceEmpty flag");
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));

        // Test with NoReplaceUnset flag and unset variable
        flags = Flags::default();
        flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .expect("Failed to set NoReplaceUnset flag");
        let result = handle_flags_on_result(String::new(), "UNSET_VAR", &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));
    }

    #[test]
    fn test_process_inner_expression() {
        let flags = Flags::default();
        let mut filters = Filters::default();
        env::set_var("TEST_VAR", "Hello, world!");

        // Test invalid operation
        let result = process_inner_expression("TEST_VAR=", &flags, &filters);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid operation: =");

        // Test basic variable replacement
        let result = process_inner_expression("TEST_VAR", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, world!");

        // Test invalid character in expression
        let result = process_inner_expression("TEST_VAR@", &flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid character in expression: @"
        );

        // Test pattern stripping (prefix)
        let result = process_inner_expression("TEST_VAR#H", &flags, &filters);
        assert_eq!(result.unwrap(), "ello, world!");

        // Test pattern stripping (suffix)
        let result = process_inner_expression("TEST_VAR%d!", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, worl");

        // Test pattern replacement
        let result = process_inner_expression("TEST_VAR/world/moon", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, moon!");

        // Test case conversion (first character to lowercase)
        let result = process_inner_expression("TEST_VAR,", &flags, &filters);
        assert_eq!(result.unwrap(), "hello, world!");

        // Test case conversion (first character to uppercase)
        env::set_var("TEST_VAR", "hello, world!");
        let result = process_inner_expression("TEST_VAR^", &flags, &filters);
        assert_eq!(result.unwrap(), "Hello, world!");

        // Test default value
        let result = process_inner_expression("UNSET_VAR:-default", &flags, &filters);
        assert_eq!(result.unwrap(), "default");

        // Test default value - empty string
        let result = process_inner_expression("UNSET_VAR:-", &flags, &filters);
        assert_eq!(result.unwrap(), "");

        // Test default value or substring extraction (substring extraction)
        let result = process_inner_expression("TEST_VAR:7:5", &flags, &filters);
        assert_eq!(result.unwrap(), "world");

        // Test Fail flag with an empty result
        let mut fail_flags = Flags::default();
        fail_flags
            .set(Flag::Fail, "--fail", true)
            .expect("Failed to set Fail flag");
        let result = process_inner_expression("UNSET_VAR", &fail_flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "environment variable 'UNSET_VAR' is not set"
        );

        // Test NoReplace flag
        let mut no_replace_flags = Flags::default();
        no_replace_flags
            .set(Flag::NoReplace, "--no-replace", true)
            .expect("Failed to set NoReplace flag");
        let result = process_inner_expression("NOT_FOUND_VARIALBE", &no_replace_flags, &filters);
        assert_eq!(result.unwrap(), "${NOT_FOUND_VARIALBE}");

        // Test _ => return Err(format!("Invalid operation: {op}")),
        //let result = process_inner_expression("TEST_VAR?invalid", &flags, &filters);
        //assert!(result.is_err());

        // Test return Ok(format!("${{{inner_expr}}}"));
        let filter_result = filters.add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter());
        assert!(filter_result.is_ok());
        let result = process_inner_expression("TEST_VAR", &flags, &filters);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "${TEST_VAR}");
    }

    #[test]
    fn test_replace_vars_in_line() {
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("VAR", "value");
        env::set_var("ANOTHER_VAR", "another_value");

        // Test character after dollar sign is invalid escaped
        let line = "this is a test line with invalid character after dollar sign $$1";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "this is a test line with invalid character after dollar sign $$1"
        );

        // Test character after dollar sign is invalid
        let line = "this is a test line with invalid character after dollar sign $1";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "this is a test line with invalid character after dollar sign $1"
        );

        // Test variable with double dollar sign at the end
        let line = "this is a test line with two dollar sign at the end of line $$";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "this is a test line with two dollar sign at the end of line $$"
        );

        // Test dollar sign at the end
        let line = "This is a dollar sign at the end: $";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a dollar sign at the end: $");

        // Test basic variable replacement
        let line = "This is a $VAR.";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a value.");

        // Test two variables in the same line
        let line = "$VAR and $ANOTHER_VAR";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "value and another_value");

        // Test escaped variable
        let line = "This is an escaped variable: $$VAR";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is an escaped variable: $VAR");

        // Test escaped variable at the end of the line
        let line = "This is an escaped variable at the end: $$VAR$";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is an escaped variable at the end: $VAR$"
        );

        // Test invalid variable
        let line = "This is an invalid variable: $1VAR";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is an invalid variable: $1VAR");

        // Test incomplete brace variable
        let line = "This is an incomplete brace variable: ${VAR";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is an incomplete brace variable: ${VAR"
        );

        // Test variable with default value
        let line = "This is a variable with a default value: ${UNSET_VAR:-default}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is a variable with a default value: default"
        );

        // Test variable with default but variable found
        let line = "This is a variable with a default value: ${VAR:-default}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is a variable with a default value: value"
        );

        // Test variable with substring extraction
        let line = "This is a variable with substring extraction: ${VAR:1:3}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is a variable with substring extraction: alu"
        );

        // Test invalid variables like ${1VAR}
        let line = "this is a test line with invalid variable ${1VAR}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "this is a test line with invalid variable ${1VAR}"
        );

        // Test invalid variables like ${1VAR:-DEFAULT}
        let line = "this is a test line with invalid variable ${1VAR:-DEFAULT}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "this is a test line with invalid variable ${1VAR:-DEFAULT}"
        );

        // Test braced var to upper
        let line = "This is a braced variable to upper: ${VAR^^}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a braced variable to upper: VALUE");

        // Test braced var to lower
        let line = "This is a braced variable to lower: ${VAR,,}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a braced variable to lower: value");

        // Test braced var first char to upper
        let line = "This is a braced variable first char to upper: ${VAR^}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is a braced variable first char to upper: Value"
        );

        // Test braced var first char to lower
        let line = "This is a braced variable first char to lower: ${VAR,}";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This is a braced variable first char to lower: value"
        );
    }

    #[test]
    fn test_process_input_basic() {
        let input = "This is a $VAR.";
        let mut output = Vec::new();
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("VAR", "value");

        process_input(Cursor::new(input.as_bytes()), &mut output, &flags, &filters)
            .expect("Failed to process input");
        assert_eq!(String::from_utf8(output).unwrap(), "This is a value.");
    }

    #[test]
    fn test_process_input_error() {
        let input = "This is a $VAR.";
        let mut output = Vec::new();
        let flags = Flags::default();
        let filters = Filters::default();

        // Test write error
        let mut failing_writer = FailingWriter::new(&mut output, true, false);
        let result = process_input(
            Cursor::new(input.as_bytes()),
            &mut failing_writer,
            &flags,
            &filters,
        );
        assert!(result.is_err());

        // Test error (caused by replace_vars_in_line)
        let input = Cursor::new("${HelloWorld,-}");
        let output = Cursor::new(Vec::new());

        let result = process_input(input, output, &flags, &filters);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_input_unbuffered_lines() {
        let input = "Line 1 $VAR1\nLine 2 $VAR2";
        let mut output = Vec::new();
        let mut flags = Flags::default();
        flags
            .set(Flag::UnbufferedLines, "--unbuffered-lines", true)
            .expect("Failed to set unbuffered lines flag");
        let filters = Filters::default();
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");

        let result = process_input(Cursor::new(input.as_bytes()), &mut output, &flags, &filters);
        assert!(result.is_ok());

        // Test unbuffered_lines write error
        let mut failing_writer = FailingWriter::new(&mut output, true, false);

        let result = process_input(
            Cursor::new(input.as_bytes()),
            &mut failing_writer,
            &flags,
            &filters,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "failed to write to output: Simulated write error"
        );
    }

    #[test]
    fn test_example_process_input_multiline() {
        use std::io::Cursor;

        let input = Cursor::new("Hello $WORLD!  \t \nHello $WORLD!  \n\tHello $WORLD!");
        let mut output = Cursor::new(Vec::new());
        let flags = Flags::default();
        let filters = Filters::default();

        process_input(Box::new(input), Box::new(&mut output), &flags, &filters).unwrap();

        assert_eq!(
            String::from_utf8(output.into_inner()).unwrap(),
            "Hello !  \t \nHello !  \n\tHello !"
        );
    }

    #[test]
    fn test_process_input_write_fail() {
        let input = "This is a $VAR.";
        let mut output = Vec::new();
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("VAR", "value");

        let mut failing_writer = FailingWriter::new(&mut output, true, false);

        let result = process_input(
            Cursor::new(input.as_bytes()),
            &mut failing_writer,
            &flags,
            &filters,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "failed to write to output: Simulated write error"
        );
    }

    #[test]
    fn test_process_input_flush_fail() {
        let input = "This is a $VAR.";
        let mut output = Vec::new();
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("VAR", "value");

        let mut failing_writer = FailingWriter::new(&mut output, false, true);

        let result = process_input(
            Cursor::new(input.as_bytes()),
            &mut failing_writer,
            &flags,
            &filters,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "failed to flush output: Simulated flush error"
        );
    }

    #[test]
    fn test_replace_vars_in_line_flags() {
        let line = "This is a $VAR.";
        let flags = Flags::default();
        let filters = Filters::default();
        env::set_var("VAR", "value");

        // Test default behavior
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a value.");

        // Test unbuffered_lines
        let mut flags = Flags::default();
        flags
            .set(Flag::UnbufferedLines, "--unbuffered-lines", true)
            .expect("Failed to set unbuffered lines flag");

        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This is a value.");

        // Test --no-replace-empty - simple var
        env::set_var("EMPTY_VAR", "");
        let line = "This following var is empty: $EMPTY_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .expect("Failed to set no replace empty flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This following var is empty: $EMPTY_VAR!");

        // Test --no-replace-empty - brace var
        env::set_var("EMPTY_VAR", "");
        let line = "This following var is empty: ${EMPTY_VAR}!";
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .expect("Failed to set no replace empty flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This following var is empty: ${EMPTY_VAR}!"
        );

        // Test --no-replace-unset - simple var
        let line = "This following var is unset: $UNSET_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .expect("Failed to set no replace unset flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This following var is unset: $UNSET_VAR!");

        // Test --no-replace-unset - brace var
        let line = "This following var is unset: ${UNSET_VAR}!";
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .expect("Failed to set no replace unset flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This following var is unset: ${UNSET_VAR}!"
        );

        // Test --no-replace-empty and --no-replace-unset
        let line =
            "This following var is unset: ${UNSET_VAR}! This following var is empty: $EMPTY_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .expect("Failed to set no replace empty flag");
        flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .expect("Failed to set no replace unset flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            "This following var is unset: ${UNSET_VAR}! This following var is empty: $EMPTY_VAR!"
        );

        // Test --fail-on-unset
        let line = "This following var is unset: $UNSET_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .expect("Failed to set fail on unset flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'UNSET_VAR' is not set"
        );

        // Test --fail-on-empty
        env::set_var("EMPTY_VAR", "");
        let line = "This following var is empty: $EMPTY_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .expect("Failed to set fail on empty flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'EMPTY_VAR' is empty"
        );

        // Test --fail-on-unset and --fail-on-empty
        env::set_var("EMPTY_VAR", "");
        let line =
            "This following var is unset: ${UNSET_VAR}! This following var is empty: $EMPTY_VAR!";
        let mut flags = Flags::default();
        flags
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .expect("Failed to set fail on unset flag");
        flags
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .expect("Failed to set fail on empty flag");
        let result = replace_vars_in_line(line, &flags, &filters);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'UNSET_VAR' is not set"
        );
    }

    #[test]
    fn test_replace_vars_in_line_escaping() {
        let flags = Flags::default();
        let filters = Filters::default();

        // Test escaping
        let line = "This is a $$ESCAPED_VAR.";
        let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());
        assert_eq!(result.unwrap(), "This is a $ESCAPED_VAR.");

        // Test escaping - brace var
        let line = "This is a $${ESCAPED_VAR}.";
        let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());
        assert_eq!(result.unwrap(), "This is a ${ESCAPED_VAR}.");

        // Test escaping - simple var
        let line = "This fi$$h should not escape!";
        let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());
        assert_eq!(result.unwrap(), "This fi$h should not escape!");

        let line = "This fi$$$$h should not escape!";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This fi$$h should not escape!");

        // Test escaping - simple var
        let line = "This pa$$$$ word should not escape!";
        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(result.unwrap(), "This pa$$ word should not escape!");
    }

    #[test]
    fn test_replace_vars_in_line_filters() {
        // Test prefixes - empty brace variable
        let line = "This is a ${PREFIX_VAR}.";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a .");

        // Test prefixes - empty simple variable
        let line = "This is a $PREFIX_VAR.";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a .");

        // Test prefixes - found brace variable
        env::set_var("PREFIX_VAR", "prefix");
        let line = "This is a \"${PREFIX_VAR}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"prefix\".");

        // Test prefixes - found simple variable
        env::set_var("PREFIX_VAR", "prefix");
        let line = "This is a \"$PREFIX_VAR\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"prefix\".");

        // Test prefixes - multiple prefixes
        env::set_var("PREFIX_VAR", "prefix");
        env::set_var("PFX_VAR", "pfx");
        let line = "This is a \"$PREFIX_VAR\"\n Her anoter \"$PFX_VAR\". This has var has no prefix: \"${NOT_FOUND}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Prefix, "--prefix", Some("PFX_VAR"), &mut [].iter())
            .expect("Failed to set prefix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"prefix\"\n Her anoter \"pfx\". This has var has no prefix: \"${NOT_FOUND}\".");

        // Test suffixes - empty brace variable
        let line = "This is a ${VAR_SUFFIX}.";
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a .");

        // Test suffixes - empty simple variable
        let line = "This is a $VAR_SUFFIX.";
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a .");

        // Test suffixes - found brace variable
        env::set_var("VAR_SUFFIX", "suffix");
        let line = "This is a \"${VAR_SUFFIX}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"suffix\".");

        // Test suffixes - found simple variable
        env::set_var("VAR_SUFFIX", "suffix");
        let line = "This is a \"$VAR_SUFFIX\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"suffix\".");

        // Test suffixes - multiple suffixes
        env::set_var("VAR_SUFFIX", "suffix");
        env::set_var("VAR_SFX", "sfx");
        let line = "This is a \"$VAR_SUFFIX\"\n Her anoter \"$VAR_SFX\". This has var has no suffix: \"${NOT_FOUND}\".";
        let mut filters = Filters::default();
        filters
            .add(
                Filter::Prefix,
                "--suffix",
                Some("VAR_SUFFIX"),
                &mut [].iter(),
            )
            .expect("Failed to set suffix filter");
        filters
            .add(Filter::Prefix, "--suffix", Some("VAR_SFX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"suffix\"\n Her anoter \"sfx\". This has var has no suffix: \"${NOT_FOUND}\".");

        // Test prefixes and suffixes - found brace variable
        env::set_var("PREFIX_VAR_SUFFIX", "prefix_suffix");
        let line = "This is a \"${PREFIX_VAR_SUFFIX}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"prefix_suffix\".");

        // Test prefixes and suffixes - found simple variable
        env::set_var("PREFIX_VAR_SUFFIX", "prefix_suffix");
        let line = "This is a \"$PREFIX_VAR_SUFFIX\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"prefix_suffix\".");

        // Test variables
        env::set_var("VAR", "value");
        let line = "This is a \"$VAR\".";
        let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());
        assert_eq!(result.unwrap(), "This is a \"value\".");

        // Test variables - empty variable
        env::set_var("VAR", "");
        let line = "This is a \"$VAR\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--variable", Some("VAR"), &mut [].iter())
            .expect("Failed to set variable filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"\".");

        // Test variables - empty variable --no-replace
        env::set_var("VAR", "");
        let line = "This is a \"$SPECIAL_VAR\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("VAR"), &mut [].iter())
            .expect("Failed to set variable filter");
        let result = replace_vars_in_line(line, &Flags::default(), &filters);
        assert_eq!(result.unwrap(), "This is a \"$SPECIAL_VAR\".");
    }

    #[test]
    fn test_colorize_text() {
        // Test default behavior
        let text = "This is a test.";
        let result = colorize_text(false, text.to_string(), Color::Green);
        assert_eq!(result, "This is a test.");

        // Test colorize
        let result = colorize_text(true, text.to_string(), Color::Green);
        assert_eq!(result, "\u{1b}[32mThis is a test.\u{1b}[0m".to_string());
    }
}
