use crate::filters::Filters;
use crate::flags::{Flag, Flags};
use crate::utils::{colorize_text, handle_flags_on_result};
use crate::variable_expansion::process_inner_expression;
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
                    new_line.push_str(&colorize_text(colored, original_variable, Color::Magenta));
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
    use colored::{Color, Colorize};
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
    fn test_read_lines_unix_endings() {
        // Test Unix line endings
        let input = "Hello $WORLD!\nHello $WORLD!\nHello $WORLD!\n";
        let expected = vec!["Hello $WORLD!\n", "Hello $WORLD!\n", "Hello $WORLD!\n"];
        let lines = read_lines(input.as_bytes());
        for (i, line_result) in lines.enumerate() {
            let line = line_result.unwrap();
            assert_eq!(line, expected[i]);
        }
    }

    #[test]
    fn test_read_lines_windows_endings() {
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
    }

    #[test]
    fn test_read_lines_emojis() {
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
        let input = Cursor::new("${HelloWorld$-}");
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
    fn test_replace_vars_in_line_filter_variable_substitution() {
        // Test multiple filters with pattern_stripping
        env::set_var("VAR", "value");
        let line = "This is a \"${VAR%ue}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("VAR"), &mut [].iter())
            .expect("Failed to set variable filter");
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let mut flags = Flags::default();
        let f = flags.set(Flag::Color, "-c", true);
        assert!(f.is_ok());

        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            format!("This is a \"{}\".", "val".color(Color::Blue))
        );
    }

    #[test]
    fn test_replace_vars_in_line_filter_pattern_stripping_suffix_not_found() {
        // Test multiple filters with pattern_stripping
        env::set_var("VAR", "value");
        let line = "This is a \"${VAR%notfound}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("VAR"), &mut [].iter())
            .expect("Failed to set variable filter");
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let mut flags = Flags::default();
        let f = flags.set(Flag::Color, "-c", true);
        assert!(f.is_ok());

        let result = replace_vars_in_line(line, &flags, &filters);
        assert_eq!(
            result.unwrap(),
            format!("This is a \"{}\".", "value".color(Color::Red))
        );
    }

    #[test]
    fn test_replace_vars_in_line_filter_pattern_stripping_variable_not_found() {
        // Test multiple filters with pattern_stripping
        let line = "This is a \"${UNSET_VAR%notfound}\".";
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("VAR"), &mut [].iter())
            .expect("Failed to set variable filter");
        filters
            .add(Filter::Prefix, "--prefix", Some("PREFIX"), &mut [].iter())
            .expect("Failed to set prefix filter");
        filters
            .add(Filter::Suffix, "--suffix", Some("SUFFIX"), &mut [].iter())
            .expect("Failed to set suffix filter");
        let mut flags = Flags::default();
        let f = flags.set(Flag::Color, "-c", true);
        assert!(f.is_ok());

        let result = replace_vars_in_line(line, &flags, &filters);
        println!("{}", result.clone().unwrap().to_string());
        assert_eq!(
            result.unwrap(),
            format!("This is a \"{}\".", "${UNSET_VAR}".color(Color::Magenta)),
        );
    }
}
