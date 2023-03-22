use crate::errors::ParseArgsError;
use crate::utils::START_PARAMETERS;
use std::fs::File;
use std::io::{Read, Write};

/// A struct representing input/output file paths.
///
/// The `InputOutput` struct contains two fields, `input` and `output`, which are
/// both of type `Option<String>`. If a value is set for `input`, it represents the path
/// to an input file. If a value is set for `output`, it represents the path to an output file.
///
/// If either of these fields is `None`, it means that there is no input or output file
/// associated with the struct.
#[derive(Debug, Default)]
pub struct InputOutput {
    input: Option<String>,
    output: Option<String>,
}

/// IO is an enumeration representing the different types of input and output.
///
/// The available types are:
/// * `Input`: Represents the input type.
/// * `Output`: Represents the output type.
///
/// The enum derives the following traits: `Debug`, `PartialEq`, `Eq`, `Hash`, `Copy`, and `Clone`.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum IO {
    /// Input type
    Input,
    /// Output type
    Output,
}

impl InputOutput {
    /// Sets the input or output stream.
    ///
    /// # Arguments
    ///
    /// * `io`: The input or output type.
    /// * `arg`: The command line argument name.
    /// * `value`: The command line argument value, if provided.
    /// * `iter`: A mutable iterator over the command line arguments.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if the value is missing or if the argument is already set.
    pub fn set(
        &mut self,
        io: IO,
        arg: &str,
        value: Option<&str>,
        iter: &mut std::slice::Iter<String>,
    ) -> Result<(), ParseArgsError> {
        let flag_arg: String = value.map_or_else(
            // if no value is provided... (was not --input=input)
            || {
                // if not, get the next argument as the value
                iter.next()
                    .map(std::string::ToString::to_string) // convert the value to a string
                    // return an error if the value is missing
                    .ok_or_else(|| ParseArgsError::MissingValue(arg.to_string()))
            },
            |s| Ok(s.to_string()), // return the value if it exists
        )?;

        if START_PARAMETERS.contains(&flag_arg.as_str()) {
            return Err(ParseArgsError::MissingValue(arg.to_string()));
        }

        match io {
            IO::Input => self.input = Some(flag_arg),
            IO::Output => self.output = Some(flag_arg),
        }
        Ok(())
    }

    /// Returns the corresponding input or output value for a given `IO` variant.
    ///
    /// # Arguments
    ///
    /// * `io`: An `IO` variant specifying whether to retrieve the input or output value.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the corresponding input or output value. If the input/output
    /// value is not set, `None` is returned.
    pub fn get(&self, io: IO) -> Option<String> {
        match io {
            IO::Input => self.input.clone(),
            IO::Output => self.output.clone(),
        }
    }
}

/// Opens an input file, or returns stdin if no input file is provided or "-" is specified.
/// Returns a boxed `dyn Read` instance on success, or an error message on failure.
///
/// # Arguments
///
/// * `input_file`: An optional `String` containing the path to the input file, or "-" to specify stdin.
///
/// # Errors
///
/// Returns an error message as a `String` if the input file cannot be opened.
///
pub fn open_input(input_file: Option<String>) -> Result<Box<dyn Read>, String> {
    let input: Box<dyn Read> = match input_file {
        Some(file) if file == "-" => Box::new(std::io::stdin()),
        Some(file) => {
            Box::new(
                // open the file with the given name
                File::open(file).map_err(
                    // if there is an error, convert it to a string and return it
                    |e| format!("Failed to open input file: {e}"),
                )?,
            )
        }
        None => Box::new(std::io::stdin()),
    };

    return Ok(input);
}

/// Opens the output file specified in the command-line arguments, or returns `stdout` if none was specified.
///
/// # Arguments
///
/// * `output_file`: An optional `String` that contains the path to the output file. If this is `None` or `"-"`, the function returns `stdout`.
///
/// # Returns
///
/// * `Result<Box<dyn Write>, String>`: A `Result` that contains a boxed `Write` trait object representing the opened file, or an error message as a `String` if the file couldn't be opened or created.
///
pub fn open_output(output_file: Option<String>) -> Result<Box<dyn Write>, String> {
    let output: Box<dyn Write> = match output_file {
        Some(file) if file == "-" => Box::new(std::io::stdout()),
        Some(file) => {
            Box::new(
                // create a new file with the given name
                File::create(file).map_err(
                    // if there is an error, convert it to a string and return it
                    |e| format!("Failed to create output file: {e}"),
                )?,
            )
        }
        None => Box::new(std::io::stdout()),
    };

    return Ok(output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_input_with_empty() {
        let input = None;
        let result = open_input(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_input_with_existent_file() {
        let input_file = NamedTempFile::new().unwrap();
        let input_file_path = input_file.path().to_str().unwrap().to_string();
        let result = open_input(input_file_path.into());
        input_file.close().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_input_with_nonexistent_file() {
        let input = String::from("tests/nonexistent_file.txt");
        let result = open_input(input.into());
        assert!(result.is_err());
    }

    #[test]
    fn test_open_output_with_stdout() {
        let output = None;
        let result = open_output(output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_output_with_nonexistent_directory() {
        let output = Some(String::from("tests/nonexistent_folder/output_file.txt"));
        let result = open_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_output_with_existent_file() {
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path().to_str().unwrap().to_string();
        let result = open_output(Some(output_file_path));
        output_file.close().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_input_output_set_input_missing_value_error() {
        let mut io = InputOutput::default();
        let args = vec!["--input".to_string(), "value".to_string()];
        let mut iter = args.iter();

        let result = io.set(IO::Input, "--input", None, &mut iter);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--input".to_string())
        );
    }

    #[test]
    fn test_input_output_set_output_missing_value_error() {
        let mut io = InputOutput::default();
        let args = vec!["--output".to_string(), "value".to_string()];
        let mut iter = args.iter();

        let result = io.set(IO::Input, "--output", None, &mut iter);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--output".to_string())
        );
    }
}
