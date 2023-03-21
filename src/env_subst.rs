use crate::filters::Filters;
use crate::flags::{Flag, Flags};
use std::env;
use std::io::{BufRead, BufReader};

/// Returns an iterator that yields lines read from a buffered reader.
///
/// This function takes an `impl BufRead` and returns an iterator that yields
/// `std::io::Result<String>` for each line read from the buffered reader. It handles
/// any errors that may occur while reading the lines and stops when reaching the end of the input.
///
/// # Arguments
///
/// * `input` - A mutable buffered reader (`impl BufRead`) from which the lines are read.
///
/// # Returns
///
/// Returns an iterator (`impl Iterator<Item = std::io::Result<String>>`) that yields
/// `std::io::Result<String>` for each line read from the input.
///
/// # Examples
///
/// ```
/// use std::io::BufReader;
/// use std::io::Cursor;
///
/// let data = "line1\nline2\nline3";
/// let cursor = Cursor::new(data);
/// let buf_reader = BufReader::new(cursor);
///
/// let lines = read_lines(buf_reader);
///
/// for line in lines {
///     println!("{}", line.unwrap());
/// }
/// ```
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

/// Retrieves the value of the environment variable specified by `var_name`, and returns it as a `String`.
/// If the variable is not set, the function checks if `default_value` is set, and returns it if it is.
/// If `default_value` is not set, the function returns the value of `original_variable`.
///
/// The function also supports additional configuration options through the `flags` argument.
///
/// # Arguments
///
/// * `var_name`: A string slice that represents the name of the environment variable to retrieve.
/// * `default_value`: A string slice that represents the default value to be used if the environment variable is not set.
/// * `original_variable`: A string slice that represents the original value of the variable, to be used as a fallback.
/// * `flags`: A reference to a `Flags` struct that contains various configuration options.
///
/// # Configuration Options
///
/// The `flags` struct has the following fields:
///
/// * `fail_on_empty`: If `true`, the function returns an error if the environment variable is set to an empty string.
/// * `fail_on_unset`: If `true`, the function returns an error if the environment variable is not set.
/// * `no_replace_empty`: If `true`, the function returns the value of `original_variable` if the environment variable is set to an empty string.
/// * `no_replace_unset`: If `true`, the function returns the value of `original_variable` if the environment variable is not set.
///
/// # Returns
///
/// Returns a `Result` that contains either the variable value as a `String`, or an error message as a `String`.
///
/// # Examples
///
/// ```
/// use my_crate::get_env_value;
///
/// let var_value = get_env_value(
///   "MY_VAR",
///   "default_value",
///   "${MY_VAR}",
///   &Flags::default(),
/// );
///
/// match var_value {
///   Ok(value) => println!("The value of MY_VAR is {}", value),
///   Err(err) => eprintln!("Error: {}", err),
/// }
/// ```
fn get_env_value(
    var_name: &str,
    default_value: &str,
    original_variable: &str,
    flags: &Flags,
) -> Result<String, String> {
    match env::var(var_name) {
        // If the variable value is empty, and the default value is empty,
        // and the `fail_on_empty` flag is set, return an error.
        Ok(value)
            if value.is_empty()
                && default_value.is_empty()
                && flags.get(Flag::FailOnEmpty).unwrap_or(false) =>
        {
            return Err(format!("environment variable '{var_name}' is empty"))
        }

        // If the variable value is empty, and the default value is not empty,
        // return the default value.
        Ok(value) if value.is_empty() && !default_value.is_empty() => {
            return Ok(default_value.to_owned())
        }

        // If the variable value is empty, and the `no_replace_empty` flag is set,
        // return the original variable value.
        Ok(value) if value.is_empty() && flags.get(Flag::NoReplaceEmpty).unwrap_or(false) => {
            return Ok(original_variable.to_owned())
        }

        // If the variable value is not empty, return the variable value.
        Ok(value) => return Ok(value),

        // If the environment variable is not set, and the default value is not empty,
        // return the default value.
        Err(_) if !default_value.is_empty() => return Ok(default_value.to_owned()),

        // If the environment variable is not set, and the `fail_on_unset` flag is set,
        // return an error.
        Err(_) if flags.get(Flag::FailOnUnset).unwrap_or(false) => {
            return Err(format!("environment variable '{var_name}' is not set"))
        }

        // If the environment variable is not set, and the `no_replace_unset` flag is set,
        // return the original variable value.
        Err(_) if flags.get(Flag::NoReplaceUnset).unwrap_or(false) => {
            return Ok(original_variable.to_owned())
        }

        // If none of the above conditions are met, return an empty string.
        // This is wanted behavior, as we don't want to replace the variable if it's not set.
        Err(_) => return Ok(String::new()),
    }
}

/// Processes a single line of text and replaces all instances of environment variables with their values.
///
/// # Arguments
///
/// * `line`: A string slice that represents the line of text to process.
/// * `flags`: A reference to a `Flags` struct that contains various configuration options.
/// * `filters`: A reference to a `Filters` struct that contains the filter criteria for which variables to replace.
///
/// # Returns
///
/// Returns a `Result` that contains the processed line of text as a `String`, or an error message as a `String`.
///
/// # Environment Variables
///
/// Environment variables are specified in the text using the `$VAR` or `${VAR}` syntax, where `VAR` is the name of the variable.
/// The `${VAR}` syntax can also include a default value, such as `${VAR:-DEFAULT}`, which specifies that if the `VAR` variable is not set, the default value `DEFAULT` should be used instead.
///
/// # Configuration Options
///
/// The `flags` struct has the following fields:
///
/// * `fail_on_empty`: If `true`, the function returns an error if an environment variable is set to an empty string.
/// * `fail_on_unset`: If `true`, the function returns an error if an environment variable is not set.
/// * `no_replace_empty`: If `true`, the function does not replace variables that are set to an empty string.
/// * `no_replace_unset`: If `true`, the function does not replace variables that are not set.
/// * `no_escape`: If `true`, the function does not treat `$$` as an escape sequence for a literal `$`.
///
/// The `filters` struct has the following fields:
///
/// * `prefix`: A string slice that represents the prefix that variable names must have in order to be replaced.
/// * `suffix`: A string slice that represents the suffix that variable names must have in order to be replaced.
/// * `variables`: A vector of string slices that represents the variable names that must be replaced.
///
/// If none of these fields are set, all variables are replaced.
///
/// # Examples
///
/// ```
/// let line = "Hello, ${NAME:-User}! How are you, ${NAME}?";
///
/// let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), "Hello, User! How are you, ?");
/// ```
fn replace_vars_in_line(line: &str, flags: &Flags, filters: &Filters) -> Result<String, String> {
    let mut new_line: String = String::with_capacity(line.len());
    let mut iter = line.chars().peekable();

    while let Some(c) = iter.next() {
        if c != '$' {
            new_line.push(c);
            continue;
        }

        let next_char = iter.peek();

        if !flags.get(Flag::NoEscape).unwrap_or(false) && next_char == Some(&'$') {
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
                .map_or(false, |c| c.is_ascii_alphabetic() || c == &'_')
            {
                new_line.push(c);
            }

            continue;
        }

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
                    new_line.push_str(&format!("${{{next}"));
                    iter.next(); // skip the number
                    continue;
                }

                let mut default_value_found = false;
                let mut default_error = false;

                while let Some(c) = iter.next() {
                    // check if possible default value => :
                    if c == ':' && !default_value_found {
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
                    new_line.push_str(&format!("${{{var_name}"));

                    // append the "broken" :
                    new_line.push(':');
                    continue; // continue to the next character
                }

                if !brace_ended {
                    // append everything that was iterated over
                    new_line.push_str(&format!("${{{var_name}"));
                    if default_value_found {
                        new_line.push_str(&format!(":-{default_value}"));
                    }
                    continue;
                }

                original_variable.push_str(&format!("${{{var_name}"));

                if default_value_found {
                    original_variable.push_str(&format!(":-{default_value}"));
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
                original_variable = format!("${var_name}");
            }
            // Everything else
            _ => {
                new_line.push(c);
                continue;
            }
        }

        if filters.matches(&var_name) == Some(false) {
            new_line.push_str(&original_variable);
            continue;
        }

        match get_env_value(&var_name, &default_value, &original_variable, flags) {
            Ok(val) => new_line.push_str(&val),
            Err(err) => return Err(err),
        }
    }

    return Ok(new_line);
}

/// Perform variable substitution on the input file and write the result to the output file.
///
/// The function reads from the provided `input` and writes the processed output to the provided `output`.
/// The `flags` parameter controls how the substitution is performed (e.g. whether to fail on unset variables),
/// and the `filters` parameter specifies which variables to substitute (e.g. only those with a certain prefix).
///
/// # Arguments
///
/// * `input` - A boxed `std::io::Read` trait object that represents the input to be read.
/// * `output` - A boxed `std::io::Write` trait object that represents the output to be written.
/// * `flags` - A reference to a `Flags` struct that specifies how the variable substitution should be performed.
/// * `filters` - A reference to a `Filters` struct that specifies which variables should be substituted.
///
/// # Returns
///
/// The function returns a `Result` with an empty tuple `()` if the substitution is successful.
/// If an error occurs during the variable substitution or file writing, a `String` with a descriptive error message is returned.
///
/// # Environment Variables
///
/// Environment variables are specified in the text using the `$VAR` or `${VAR}` syntax, where `VAR` is the name of the variable.
/// The `${VAR}` syntax can also include a default value, such as `${VAR:-DEFAULT}`, which specifies that if the `VAR` variable is not set, the default value `DEFAULT` should be used instead.
///
/// # Configuration Options
///
/// The `flags` struct has the following fields:
///
/// * `fail_on_empty`: If `true`, the function returns an error if an environment variable is set to an empty string.
/// * `fail_on_unset`: If `true`, the function returns an error if an environment variable is not set.
/// * `no_replace_empty`: If `true`, the function does not replace variables that are set to an empty string.
/// * `no_replace_unset`: If `true`, the function does not replace variables that are not set.
/// * `no_escape`: If `true`, the function does not treat `$$` as an escape sequence for a literal `$`.
///
/// The `filters` struct has the following fields:
///
/// * `prefix`: A string slice that represents the prefix that variable names must have in order to be replaced.
/// * `suffix`: A string slice that represents the suffix that variable names must have in order to be replaced.
/// * `variables`: A vector of string slices that represents the variable names that must be replaced.
///
/// If none of these fields are set, all variables are replaced.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use renvsubst::{process_input, Flags, Filters};
///
/// let input = Cursor::new("Hello $WORLD!");
/// let mut output = Cursor::new(Vec::new());
/// let flags = Flags::default();
/// let filters = Filters::default();
///
/// process_input(Box::new(input), Box::new(&mut output), &flags, &filters).unwrap();
///
/// assert_eq!(String::from_utf8(output.into_inner()).unwrap(), "Hello !\n");
/// ```
pub fn process_input<R: std::io::Read, W: std::io::Write>(
    input: R,
    mut output: W,
    flags: &Flags,
    filters: &Filters,
) -> Result<(), String> {
    let reader: BufReader<R> = BufReader::new(input);
    let mut buffer = String::new();
    let unbuffered_lines = flags.get(Flag::UnbufferedLines).unwrap_or_default();

    for line in read_lines(reader) {
        let line: String = line.unwrap();
        let replaced: Result<String, String> = replace_vars_in_line(&line, flags, filters);
        match replaced {
            Ok(out) => {
                // if unbuffered lines mode is enabled, write each line as soon as it's processed
                if unbuffered_lines {
                    if let Err(e) = output.write(out.as_bytes()) {
                        return Err(format!("Failed to write to output: {e}"));
                    }
                    continue;
                }
                // if unbuffered lines mode is not enabled, append the line to the buffer
                buffer.push_str(&out);
            }
            Err(e) => return Err(format!("Failed to replace variables: {e}")),
        }
    }

    // if unbuffered lines mode is not enabled, write the entire buffer to the output file at once
    if !unbuffered_lines {
        if let Err(e) = output.write_all(buffer.as_bytes()) {
            return Err(format!("Failed to write to output: {e}"));
        }
    }

    // flush the output to ensure that all written data is actually written to the output stream
    if let Err(e) = output.flush() {
        return Err(format!("Failed to flush output: {e}"));
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
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
    fn test_replace_vars_in_line_regular_var_found() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        env::set_var("REGULAR_VAR_FOUND", "value");
        let line = "$REGULAR_VAR_FOUND".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_one_new_line_at_end() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        let line = "this is a line\n".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this is a line\n".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_zwo_new_line_at_end() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        let line = "this is a line\n\n".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this is a line\n\n".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_starting_dash() {
        // description: regular variable with starting dash
        // test: $_REGULAR_VAR_FOUND_WITH_DASH
        // env: _REGULAR_VAR_FOUND_WITH_DASH=value
        // result: value
        env::set_var("_REGULAR_VAR_FOUND_WITH_DASH", "value");
        let line = "$_REGULAR_VAR_FOUND_WITH_DASH".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_not_found_fail_on_unset() {
        // description: regular variable not found
        // test: $REGULAR_VAR_NOT_FOUND
        // env: -
        // result: -
        let line = "$REGULAR_VAR_NOT_FOUND".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'REGULAR_VAR_NOT_FOUND' is not set"
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_not_found() {
        // description: regular variable not found
        // test: $REGULAR_VAR_NOT_FOUND
        // env: -
        // result: -
        let line = "$REGULAR_VAR_NOT_FOUND".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok(String::new()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_found() {
        // description: braces variable found
        // test: ${BRACES_VAR_FOUND}
        // env: BRACES_VAR_FOUND=value
        // result: value
        env::set_var("BRACES_VAR_FOUND", "value");
        let line = "${BRACES_VAR_FOUND}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_found_starting_dash() {
        // description: braces variable found with starting dash
        // test: ${_BRACES_VAR_WITH_DASH}
        // env: _BRACES_VAR_WITH_DASH=value
        // result: value
        env::set_var("_BRACES_VAR_WITH_DASH", "value");
        let line = "${_BRACES_VAR_WITH_DASH}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_found_long_value() {
        // description: regular variable found
        // test: $REGULAR_VAR_FOUND
        // env: REGULAR_VAR_FOUND=value
        // result: value
        env::set_var(
            "REGULAR_VAR_LONG_FOUND",
            "valuevaluevaluevaluevaluevaluevaluevaluevaluevaluevalue",
        );
        let line = "$REGULAR_VAR_LONG_FOUND".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("valuevaluevaluevaluevaluevaluevaluevaluevaluevaluevalue".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_not_found() {
        // description: braces variable not found
        // test: ${BRACES_VAR_NOT_FOUND}
        // env: unset
        // result: -
        let line = "${BRACES_VAR_NOT_FOUND}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok(String::new()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_not_found_fail_on_unset() {
        // description: braces variable not found
        // test: ${BRACES_VAR_NOT_FOUND}
        // env: unset
        // result: -
        let line = "${BRACES_VAR_NOT_FOUND}".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "environment variable 'BRACES_VAR_NOT_FOUND' is not set"
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_default() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT:-default}
        // env: unset
        // result: default
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT:-default}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("default".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_broken_default() {
        // description: braces variable with broken default value
        // test: "Hello, ${NAME:Worl:-d}!"
        // env: unset
        // result: "Hello, ${NAME:Worl:-d}!"
        let input = "Hello, ${NAME:Worl:-d}!";
        let flags = Flags::default();
        let filters = Filters::default();

        let result = replace_vars_in_line(input, &flags, &filters);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, ${NAME:Worl:-d}!");
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_colon_in_default() {
        // description: braces variable with colon inside default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa:ult}
        // env: unset
        // result: defa:ult
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa:ult}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("defa:ult".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_dollar_in_default() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa$ult}
        // env: unset
        // result: defa$ult
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa$ult}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("defa$ult".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_braces_in_default() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT:-defa}ult}
        // env: unset
        // result: default}
        let line = "${B:-defa}ult}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("default}".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_var() {
        // description: braces variable with default value, use variable
        // test: ${BRACES_VAR_DEFAULT_USE_VAR:-default}
        // env: BRACES_VAR_DEFAULT_USE_VAR=value
        // result: value
        env::set_var("BRACES_VAR_DEFAULT_USE_VAR", "value");
        let line = "${BRACES_VAR_DEFAULT_USE_VAR:-default}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_var_dash() {
        // description: braces variable with default value, use variable
        // test: ${_BRACES_VAR_DEFAULT_USE_VAR:-default}
        // env: _BRACES_VAR_DEFAULT_USE_VAR=value
        // result: value
        env::set_var("_BRACES_VAR_DEFAULT_USE_VAR", "value");
        let line = "${_BRACES_VAR_DEFAULT_USE_VAR:-default}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_default_dash() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT_DASH:-_default}
        // env: BRACES_VAR_DEFAULT_USE_DEFAULT_DASH=value
        // result: value
        env::set_var("BRACES_VAR_DEFAULT_USE_DEFAULT_DASH", "value");
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT_DASH:-default}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_default_use_default_empty() {
        // description: braces variable with default value, use default
        // test: ${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-}
        // env: unset
        // result: -
        let line = "${BRACES_VAR_DEFAULT_USE_DEFAULT_EMPTY:-}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok(String::new()));
    }

    #[test]
    fn test_replace_vars_in_line_escape_text_double_dollar_invalid_var() {
        // description: escape text, double dollar, invalid var
        // test: i like cas$$ not so much!
        // env: -
        // result: i like cas$$ not so much!
        let line = "i like cas$$ not so much!".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_replace_vars_in_line_escape_text_double_dollar_invald_var_no_escape_true() {
        // description: escape text, double dollar, no escape true
        // test: i like cas$$ not so much!
        // env: -
        // result: i like cas$ not so much!
        let line = "i like cas$$ not so much!".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_replace_vars_in_line_escape_var_double_dollar_valid_var() {
        // description: escape variable, double dollar, valid var
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$word
        let line = "I have a pa$$word".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("I have a pa$word".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_escape_var_double_dollar_no_replace_unset() {
        // description: escape variable, double dollar, no replace unset
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$word
        let line = "I have a pa$$word".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok("I have a pa$word".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_escape_text_single_dollar_no_escape_true() {
        // description: escape text, single dollar, no escape
        // test: this $ is a dollar sign
        // env: -
        // result: this $ is a dollar sign
        let line = "this $ is a dollar sign".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_replace_vars_in_line_escape_var_double_dollar_no_escape() {
        // description: escape variable, double dollar, no escape
        // test: I have a pa$$word
        // env: -
        // result: I have a pa$$word
        let line = "I have a pa$$word".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok("I have a pa$".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_escape_text_single_dollar_no_escape_false() {
        // description: escape text, single dollar, no escape
        // test: this $ is a dollar sign
        // env: -
        // result: this $ is a dollar sign
        let line = "this $ is a dollar sign".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok(line));
    }

    #[test]
    fn test_replace_vars_in_line_broken_var_braces_end() {
        // description: broken variable, braces end
        // test: this variable $BROKEN_VAR_BRACES_END} is broken
        // env: BROKEN_VAR_BRACES_END=value
        // result: this variable value} is broken
        env::set_var("BROKEN_VAR_BRACES_END", "value");
        let line = "this variable $BROKEN_VAR_BRACES_END} is broken".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this variable value} is broken".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_broken_var_braces_begin() {
        // description: broken variable, braces begin
        // test: this variable ${BROKEN_VAR_BRACES_BEGIN is broken
        // env: BROKEN_VAR_BRACES_BEGIN=value
        // result: this variable ${BROKEN_VAR_BRACES_BEGIN is broken
        env::set_var("BROKEN_VAR_BRACES_BEGIN", "value");
        let line = "this variable ${BROKEN_VAR_BRACES_END is broken".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("this variable ${BROKEN_VAR_BRACES_END is broken".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_invalid_regular_var_digit_begin() {
        // description: invalid regular variable, digit begin
        // test: this $1INVALID_VAR_DIGIT_BEGIN is not valid
        // env: -
        // result: this $1INVALID_VAR_DIGIT_BEGIN is not valid
        let line = "this $1INVALID_VAR_DIGIT_BEGIN is not valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("this $1INVALID_VAR_DIGIT_BEGIN is not valid".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_invalid_braces_var_digit_begin() {
        // description: invalid braces variable, digit begin
        // test: this ${1INVALID_VAR_DIGIT_BEGIN} is not valid
        // env: -
        // result: this ${1INVALID_VAR_DIGIT_BEGIN} is not valid
        let line = "this ${1INVALID_VAR_DIGIT_BEGIN} is not valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("this ${1INVALID_VAR_DIGIT_BEGIN} is not valid".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_digit_middle() {
        // description: valid regular variable, digit middle
        // test: this $VALID_REGULAR_VAR_1_DIGIT_MIDDLE is valid
        // env: VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_1_DIGIT_MIDDLE", "value");
        let line = "this $VALID_REGULAR_VAR_1_DIGIT_MIDDLE is valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_digit_end() {
        // description: valid regular variable, digit end
        // test: this $VALID_REGULAR_VAR_DIGIT_END_1 is valid
        // env: VALID_REGULAR_VAR_DIGIT_END_1=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_DIGIT_END_1", "value");
        let line = "this $VALID_REGULAR_VAR_DIGIT_END_1 is valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_digit_middle() {
        // description: valid braces variable, digit middle
        // test: this ${VALID_REGULAR_VAR_1_DIGIT_MIDDLE} is valid
        // env: VALID_REGULAR_VAR_1_DIGIT_MIDDLE=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_1_DIGIT_MIDDLE", "value");
        let line = "this ${VALID_REGULAR_VAR_1_DIGIT_MIDDLE} is valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_digit_end() {
        // description: valid braces variable, digit end
        // test: this ${VALID_REGULAR_VAR_DIGIT_END_1} is valid
        // env: VALID_REGULAR_VAR_DIGIT_END_1=value
        // result: this value is valid
        env::set_var("VALID_REGULAR_VAR_DIGIT_END_1", "value");
        let line = "this ${VALID_REGULAR_VAR_DIGIT_END_1} is valid".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("this value is valid".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_end() {
        // description: valid braces variable, end of line
        // test: braces var at the end ${VALID_BRACES_VAR_END}
        // env: VALID_BRACES_VAR_END=value
        // result: braces var at the end value
        env::set_var("VALID_BRACES_VAR_END", "value");
        let line = "braces var at the end ${VALID_BRACES_VAR_END}".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("braces var at the end value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_begin() {
        // description: valid braces variable, begin of line
        // test: ${VALID_BRACES_VAR_BEGIN} braces var at the begin
        // env: VALID_BRACES_VAR_BEGIN=value
        // result: value braces var at the begin
        env::set_var("VALID_BRACES_VAR_BEGIN", "value");
        let line = "${VALID_BRACES_VAR_BEGIN} braces var at the begin".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value braces var at the begin".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_end() {
        // description: valid regular variable, at end of line
        // test: regular var at the end $VALID_REGULAR_VAR_END
        // env: VALID_REGULAR_VAR_END=value
        // result: regular var at the end value
        env::set_var("VALID_REGULAR_VAR_END", "value");
        let line = "regular var at the end $VALID_REGULAR_VAR_END".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("regular var at the end value".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_begin() {
        // description: valid regular variable, at begin of line
        // test: $VALID_REGULAR_VAR_BEGIN regular var at the begin
        // env: VALID_REGULAR_VAR_BEGIN=value
        // result: value regular var at the begin
        env::set_var("VALID_REGULAR_VAR_BEGIN", "value");
        let line = "$VALID_REGULAR_VAR_BEGIN regular var at the begin".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(result, Ok("value regular var at the begin".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_fail_on_unset() {
        // description: valid regular variable, fail on empty
        // test: $VALID_REGULAR_VAR_FAIL_ON_UNSET
        // env:
        // result:
        let line = "$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_fail_on_unset() {
        // description: valid braces variable, fail on unset
        // test: ${VALID_BRACES_VAR_FAIL_ON_UNSET}
        // env:
        // result:
        let line = "${VALID_BRACES_VAR_FAIL_ON_UNSET}".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_fail_on_empty() {
        // description: valid regular variable, fail on empty
        // test: $VALID_REGULAR_VAR_BEGIN
        // env: VALID_REGULAR_VAR_BEGIN=""
        // result: -
        env::set_var("VALID_REGULAR_VAR_FAIL_ON_EMPTY", "");
        let line = "$VALID_REGULAR_VAR_FAIL_ON_EMPTY".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_fail_on_empty() {
        // description: valid braces variable, fail on empty
        // test: $VALID_REGULAR_VAR_BEGIN regular var at the begin
        // env: VALID_REGULAR_VAR_BEGIN=""
        // result: -
        env::set_var("VALID_REGULAR_VAR_FAIL_ON_EMPTY", "");
        let line = "${VALID_REGULAR_VAR_FAIL_ON_EMPTY}".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_no_replace_unset() {
        // description: valid regular variable, no replace on unset
        // test: $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET
        // env:
        // result: $VALID_REGULAR_VAR_NO_REPLACE_ON_UNSET
        let line = "$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok("$VALID_REGULAR_VAR_FAIL_ON_UNSET".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_no_replace_unset() {
        // description: valid braces variable, no replace on unset
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET}
        // env:
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_UNSET}
        let line = "${VALID_REGULAR_VAR_FAIL_ON_UNSET}".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok("${VALID_REGULAR_VAR_FAIL_ON_UNSET}".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_valid_regular_var_no_replace_empty() {
        // description: valid regular variable, no replace on empty
        // test: $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY
        // env: VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY=""
        // result: $VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY
        env::set_var("VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY", "");
        let line = "$VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(
            result,
            Ok("$VALID_REGULAR_VAR_NO_REPLACE_ON_EMPTY".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_valid_braces_var_no_replace_empty() {
        // description: valid braces variable, no replace on empty
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}
        // env: VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY=""
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}
        env::set_var("VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY", "");
        let line = "${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(
            result,
            Ok("${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY}".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_invalid_braces_var_default_end() {
        // description: invalid braces variable, default at the end
        // test: ${IVALID_BRACES_VAR_DEFAULT_END:-
        // env: -
        // result: ${IVALID_BRACES_VAR_DEFAULT_END:-
        let line = "${IVALID_BRACES_VAR_DEFAULT_END:-".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(result, Ok("${IVALID_BRACES_VAR_DEFAULT_END:-".to_string()));
    }

    #[test]
    fn test_replace_vars_in_line_invalid_braces_var_broken_default_end() {
        // description: invalid braces variable, default at the end
        // test: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:
        // env: -
        // result: ${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:
        let line = "${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(
            result,
            Ok("${VALID_BRACES_VAR_NO_REPLACE_ON_EMPTY:".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_dollar_end() {
        // description: only one dollar sign at the end of line
        // test: this is a test line with only one dollar sign at the end of line $
        // env: -
        // result: this is a test line with only one dollar sign at the end of line $
        let line = "this is a test line with only one dollar sign at the end of line $".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("this is a test line with only one dollar sign at the end of line $".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_double_dollar_end() {
        // description: two dollar sign at the end of line
        // test: this is a test line with two dollar sign at the end of line $$
        // env: -
        // result: this is a test line with two dollar sign at the end of line $$
        let line = "this is a test line with two dollar sign at the end of line $$".to_string();
        let result = replace_vars_in_line(&line, &Flags::default(), &Filters::default());
        assert_eq!(
            result,
            Ok("this is a test line with two dollar sign at the end of line $$".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_double_dollar_end_escape_true() {
        // description: double dollar sign at the end of line, no escape true
        // test: this is a test line with two dollar sign at the end of line $$
        // env: -
        // result: this is a test line with two dollar sign at the end of line $$
        let line = "this is a test line with two dollar sign at the end of line $$".to_string();
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();
        let result = replace_vars_in_line(&line, &flags, &Filters::default());
        assert_eq!(
            result,
            Ok("this is a test line with two dollar sign at the end of line $$".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_prefix() {
        // description: regular variable with prefix
        // test: this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.
        // env: ENV1=env1, TEST_VAR1=test_var1
        // result:this $ENV1 has a prefix. This test_var1 has a prefix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        let line = "this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                // insert a prefix to test the prefix filter as HashSet
                prefixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This test_var1 has a prefix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_prefix() {
        // description: braces variable with prefix
        // test: this $ENV1 has a prefix. This $TEST_VAR1 has a prefix.
        // env: ENV1=env1, TEST_VAR1=test_var1
        // result:this $ENV1 has a prefix. This test_var1 has a prefix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        let line = "this $ENV1 has a no prefix. This ${TEST_VAR1} has a valid prefix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                prefixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a no prefix. This test_var1 has a valid prefix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_suffix() {
        // description: regular variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $ENV1 has a prefix. This $VAR1_TEST has a suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                suffixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_multiple_suffix() {
        // description: regular variable with multiple suffix
        // test: this $ENV1 has no suffix. This "$VAR_FIRST" has a suffix. And this "${VAR_SECOND}" has another suffix.
        // env: ENV1=env1, VAR_FIRST=first suffix, VAR_SECOND=second suffix
        // result: this this $ENV1 has a prefix. This "first suffix" has a suffix. And this "second suffix" has another suffix.
        env::set_var("ENV1", "env1");
        env::set_var("VAR_FIRST", "first suffix");
        env::set_var("VAR_SECOND", "second suffix");
        let line = "this this $ENV1 has no suffix. This \"$VAR_FIRST\" has a suffix. And this \"${VAR_SECOND}\" has another suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                suffixes: Some(HashSet::from_iter(vec![
                    "FIRST".to_string(),
                    "SECOND".to_string(),
                ])),
                ..Filters::default()
            },
        );
        assert_eq!(
          result,
          Ok("this this $ENV1 has no suffix. This \"first suffix\" has a suffix. And this \"second suffix\" has another suffix.".to_string())
      );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_multiple_prefix() {
        // description: regular variable with multiple prefix
        // test: this $ENV1 has no prefix. This "$FIRST_VAR" has a prefix. And this "${SECOND_VAR}" has another suffix.
        // env: ENV1=env1, FIRST_VAR=first prefix, SECOND_VAR=second prefix
        // result: this this $ENV1 has no prefix. This "first suffix" has a suffix. And this "second suffix" has another suffix.
        env::set_var("ENV1", "env1");
        env::set_var("FIRST_VAR", "first prefix");
        env::set_var("SECOND_VAR", "second prefix");
        let line = "this this $ENV1 has no prefix. This \"$FIRST_VAR\" has a prefix. And this \"${SECOND_VAR}\" has another prefix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                prefixes: Some(HashSet::from_iter(vec![
                    "FIRST".to_string(),
                    "SECOND".to_string(),
                ])),
                ..Filters::default()
            },
        );
        assert_eq!(
          result,
          Ok("this this $ENV1 has no prefix. This \"first prefix\" has a prefix. And this \"second prefix\" has another prefix.".to_string())
      );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $ENV1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                suffixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this $ENV1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_no_prefix_valid_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                suffixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this $TEST_VAR1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_valid_prefix_no_suffix() {
        // description: braces variable with suffix
        // test: this $ENV1 has a prefix. This $VAR1_TEST has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                prefixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this test_var1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_braces_var_valid_no_prefix_valid_suffix() {
        // description: braces variable with suffix
        // test: this var $ENV1 should not be touched. this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.
        // env: ENV1=env1, VAR1_TEST=var1_var
        // result:this $ENV1 has a prefix. This test_var1 has a suffix. This var1_test has a suffix.
        env::set_var("ENV1", "env1");
        env::set_var("TEST_VAR1", "test_var1");
        env::set_var("VAR1_TEST", "var1_test");
        let line = "this var $ENV1 should not be touched. this $TEST_VAR1 has a prefix. This ${VAR1_TEST} has a suffix.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                suffixes: Some(HashSet::from_iter(vec!["TEST".to_string()])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("this var $ENV1 should not be touched. this $TEST_VAR1 has a prefix. This var1_test has a suffix.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_list_variables() {
        // description: regular variable with a list of variables
        // test: Only ENV1 and ENV2 should be replaced. ENV3 should not be replaced.
        // env: ENV1=env1, ENV2=env2
        // result: Only env1 and env2 should be replaced. ENV2 should not be replaced.
        env::set_var("ENV1", "env1");
        env::set_var("ENV2", "env2");
        env::set_var("ENV3", "env4");
        let line =
            "Only $ENV1 and $ENV2 should be replaced. $ENV3 should not be replaced.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                variables: Some(HashSet::from_iter(vec![
                    "ENV1".to_string(),
                    "ENV2".to_string(),
                ])),
                ..Filters::default()
            },
        );
        assert_eq!(
            result,
            Ok("Only env1 and env2 should be replaced. $ENV3 should not be replaced.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_list_variables_prefix_suffix_not_found() {
        // description: all filter set, non matches
        // test: $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.
        // env: -
        // result: $PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.
        let line = "$PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                variables: Some(HashSet::from_iter(vec![
                    "ENV1".to_string(),
                    "ENV2".to_string(),
                ])),
                prefixes: Some(HashSet::from_iter(vec!["BAD_PREFIX".to_string()])),
                suffixes: Some(HashSet::from_iter(vec!["BAD_SUFFIX".to_string()])),
            },
        );
        assert_eq!(
            result,
            Ok("$PREFIX_ENV1 and $ENV2_SUFFIX and $VAR should not be replaced.".to_string())
        );
    }

    #[test]
    fn test_replace_vars_in_line_regular_var_all_filter_match() {
        // description: all filter set, all match
        // test: ${PREFIX_VAR_SUFFIX}
        // env: -
        // result: prefix var suffix
        env::set_var("PREFIX_VAR_SUFFIX", "prefix var suffix");
        let line = "${PREFIX_VAR_SUFFIX}".to_string();
        let result = replace_vars_in_line(
            &line,
            &Flags::default(),
            &Filters {
                variables: Some(HashSet::from_iter(vec!["PREFIX_VAR_SUFFIX".to_string()])),
                prefixes: Some(HashSet::from_iter(vec!["PREFIX".to_string()])),
                suffixes: Some(HashSet::from_iter(vec!["SUFFIX".to_string()])),
            },
        );
        assert_eq!(result, Ok("prefix var suffix".to_string()));
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
        let result = get_env_value(var_name, default_value, original_var, &Flags::default());
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
        let result = get_env_value(var_name, default_value, original_var, &Flags::default());
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
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = get_env_value(var_name, default_value, original_var, &flags);
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
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();
        let result = get_env_value(var_name, default_value, original_var, &flags);
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
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();
        let result = get_env_value(var_name, default_value, original_var, &flags);
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
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();
        let result = get_env_value(var_name, default_value, original_var, &flags);
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
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();

        let result = get_env_value(var_name, default_value, original_var, &flags);
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
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();

        let result = get_env_value(var_name, default_value, original_var, &flags);
        // check if the result is an error
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_brace_variable_default_no_replace_empty_false() {
        // description: brace variable with default an no-replace-empty not set
        // test: ${VAR:-default}
        // env: -
        // result: default
        let var_name = "BRACE_VARIABLE_DEFAULT_NO_REPLACE_EMPTY_FALSE";
        let original_var = "${BRACE_VARIABLE_DEFAULT_NO_REPLACE_EMPTY_FALSE:-default}";
        let default_value = "default";
        let result = get_env_value(var_name, default_value, original_var, &Flags::default());

        assert_eq!(result, Ok("default".to_string()));
    }

    #[test]
    fn test_example_replace_vars_in_line() {
        // description: replace variables in line
        // test: Hello, ${NAME:-User}! How are you, ${NAME}?
        // env: unset
        // result: Hello, User! How are you, ?
        let line = "Hello, ${NAME:-User}! How are you, ${NAME}?";
        let result = replace_vars_in_line(line, &Flags::default(), &Filters::default());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, User! How are you, ?");
    }

    #[test]
    fn test_example_get_env_value() {
        // description: get environment variable value
        // test: unset
        // env: unset
        // result: The value of MY_VAR is default_value
        let var_value = get_env_value("MY_VAR", "default_value", "${MY_VAR}", &Flags::default());

        //match var_value {
        //    Ok(value) => println!("The value of MY_VAR is {value}"),
        //    Err(err) => eprintln!("Error: {err}"),
        //}
        assert!(var_value.is_ok());
        assert_eq!(var_value.unwrap(), "default_value");
    }

    #[test]
    fn test_example_process_input() {
        use std::io::Cursor;

        let input = Cursor::new("Hello $WORLD!");
        let mut output = Cursor::new(Vec::new());
        let flags = Flags::default();
        let filters = Filters::default();

        process_input(Box::new(input), Box::new(&mut output), &flags, &filters).unwrap();

        assert_eq!(String::from_utf8(output.into_inner()).unwrap(), "Hello !");
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
    fn test_example_process_input_multiline_unbuffered() {
        use std::io::Cursor;

        let input = Cursor::new("Hello $WORLD! 🙃 \t \nHello $WORLD! 🎂 \n\t❤️Hello $WORLD!");
        let mut output = Cursor::new(Vec::new());
        let mut flags = Flags::default();
        flags.set(Flag::UnbufferedLines, true).unwrap();
        let filters = Filters::default();

        process_input(Box::new(input), Box::new(&mut output), &flags, &filters).unwrap();

        assert_eq!(
            String::from_utf8(output.into_inner()).unwrap(),
            "Hello ! 🙃 \t \nHello ! 🎂 \n\t❤️Hello !"
        );
    }

    #[test]
    fn test_example_process_input_error() {
        use std::io::Cursor;

        let input = Cursor::new("Hello $WORLD!");
        let mut output = Cursor::new(Vec::new());
        let mut flags = Flags::default();

        let f = flags.set(Flag::FailOnUnset, true);
        assert!(f.is_ok());

        let filters = Filters::default();
        let result = process_input(Box::new(input), Box::new(&mut output), &flags, &filters);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_env_value_default_value() {
        let var_name = "EMPTY_VAR_NAME";
        let original_variable = "${EMPTY_VAR_NAME:default}";
        let default_value = "default";
        let flags = Flags::default();

        env::set_var("EMPTY_VAR_NAME", "");
        let result = get_env_value(var_name, default_value, original_variable, &flags);

        assert_eq!(result, Ok("default".to_string()));
    }

    #[test]
    fn test_read_lines() {
        let input = "Hello $WORLD!\nHello $WORLD!\nHello $WORLD!\n";
        let expected = vec!["Hello $WORLD!\n", "Hello $WORLD!\n", "Hello $WORLD!\n"];
        let lines = read_lines(input.as_bytes());
        for (i, line_result) in lines.enumerate() {
            let line = line_result.unwrap();
            assert_eq!(line, expected[i]);
        }
    }

    #[test]
    fn test_read_lines_windows_ending() {
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
    fn test_read_lines_example() {
        use std::io::BufReader;
        use std::io::Cursor;

        let data = "line1\nline2\nline3";
        let cursor = Cursor::new(data);
        let buf_reader = BufReader::new(cursor);

        let lines = read_lines(buf_reader);

        for line in lines {
            // assert if line can unwrap
            line.unwrap();
        }
    }

    #[test]
    fn test_process_line_buffered_lines_write_error() {
        let input = "Hello $WORLD!  \nHello $WORLD!  \nHello $WORLD!";
        let mut output = Cursor::new(Vec::new());
        let mut flags = Flags::default();
        flags.set(Flag::UnbufferedLines, true).unwrap();

        let result = process_input(
            Box::new(input.as_bytes()),
            &mut FailingWriter::new(&mut output, true, false),
            &flags,
            &Filters::default(),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Failed to write to output: Simulated write error"
        );
    }

    #[test]
    fn test_read_lines_emojies() {
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
    fn test_process_input_write_output_error() {
        let input = "Hello $WORLD!  \nHello $WORLD!  \nHello $WORLD!";
        let mut output = Cursor::new(Vec::new());

        let result = process_input(
            Box::new(input.as_bytes()),
            &mut FailingWriter::new(&mut output, true, false),
            &Flags::default(),
            &Filters::default(),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Failed to write to output: Simulated write error"
        );
    }

    #[test]
    fn test_process_input_flush_output_error() {
        let input = "Hello $WORLD!  \nHello $WORLD!  \nHello $WORLD!";
        let mut output = Cursor::new(Vec::new());

        let result = process_input(
            Box::new(input.as_bytes()),
            &mut FailingWriter::new(&mut output, false, true),
            &Flags::default(),
            &Filters::default(),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Failed to flush output: Simulated flush error"
        );
    }

    #[test]
    fn test_process_input_large() {
        let mut input = String::new();
        for _i in 0..50_000 {
            input.push_str(&format!("
              This is a \"$FILE_NAME\" file.
              It has more than \"${{AMOUNT}}\" different variables.
              You can also use \"${{UNSET_VARIABLE:-default}}\" values inside variables like \"${{UNSET_VARIABLE:-default}}\".
              Here are more variable like \"${{PREFIXED_VARIABLE_1}}\" and \"${{VARIABLE_1_SUFFIXED}}\".
              Here are more \"$PREFIXED_VARIABLE_2\" and \"$VARIABLE_2_SUFFIXED\" variables!
              Here are other prefixed \"$prefixed_VARIABLE_3\" and suffixed \"$VARIABLE_3_suffixed\" variables!
              Or you can escape Text with two dollar signs ($$) like fi$$h => fi$h.
              "
            ));
        }
        //println!("input size in MB: {}", input.len() / 1024 / 1024);

        let mut filter: Filters = Default::default();

        let mut prefixes = HashSet::new();
        for i in 0..1000 {
            // append to prefixes set
            prefixes.insert(format!("PREFIXED_{i}"));
        }
        filter.prefixes = Some(prefixes);

        let mut suffixes = HashSet::new();
        for i in 0..512 {
            // append to prefixes set
            suffixes.insert(format!("_SUFFIXED{i}"));
        }
        filter.suffixes = Some(suffixes);

        let mut variables = HashSet::new();
        for i in 0..512 {
            // append to prefixes set
            variables.insert(format!("VARIABLE{i}"));
        }
        filter.variables = Some(variables);

        let result = process_input(
            Box::new(input.as_bytes()),
            Cursor::new(Vec::new()),
            &Flags::default(),
            &Filters::default(),
        );

        assert!(result.is_ok());

        // add more assertions here to test the expected behavior of `process_input` given the large input and filters
    }
}
