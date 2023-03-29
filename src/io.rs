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
    /// Sets the value of the input or output path based on the provided `IO` parameter and
    /// command-line arguments.
    ///
    /// # Arguments
    ///
    /// * `io`: An `IO` enum variant that specifies whether the input or output path should be
    ///   set.
    /// * `arg`: A `&str` representing the command-line flag argument that was used to set the
    ///   input or output path.
    /// * `value`: An optional `&str` representing the value that was provided for the command-line
    ///   flag. If no value is provided, the function tries to get the next argument from the
    ///   provided `iter`.
    /// * `iter`: A mutable reference to an iterator over the command-line arguments.
    ///
    /// # Returns
    ///
    /// A `Result<(), ParseArgsError>` that is `Ok(())` if the input or output path was successfully
    /// set, or an error message as a `ParseArgsError` if the value is missing or invalid.
    ///
    /// # Errors
    ///
    /// This function can return the following error messages:
    ///
    /// * `ParseArgsError::MissingValue(arg)` - When a value is missing for a command-line flag that
    ///   requires one.
    ///
    /// # Notes
    ///
    /// This function is used to set the value of the input or output path based on the provided
    /// `IO` parameter and command-line arguments. It takes an `io` parameter that specifies whether
    /// the input or output path should be set, an `arg` parameter that represents the command-line
    /// flag argument that was used to set the input or output path, a `value` parameter that
    /// represents the value that was provided for the command-line flag, and an `iter` parameter
    /// that is a mutable reference to an iterator over the command-line arguments.
    ///
    /// If no `value` is provided for the `arg`, the function tries to get the next argument from
    /// the provided `iter`. If the `value` or next argument is missing, the function returns a
    /// `ParseArgsError::MissingValue` error message. If the `value` or next argument is one of the
    /// start parameters (defined in the `START_PARAMETERS` constant), the function returns a
    /// `ParseArgsError::MissingValue` error message.
    ///
    /// If the `IO` parameter is `IO::Input`, the function sets the `input` field of the struct to
    /// the provided `value` or next argument. If the `IO` parameter is `IO::Output`, the function
    /// sets the `output` field of the struct to the provided `value` or next argument.
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

    /// Returns a reference to the value of the specified input/output `IO` option, if it has been set.
    ///
    /// # Arguments
    ///
    /// * `io`: An `IO` value indicating which input/output option to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<&String>` containing a reference to the value of the specified input/output option, if it has been set.
    /// If the specified option has not been set, the function returns `None`.
    ///
    #[must_use]
    pub fn get(&self, io: IO) -> Option<&String> {
        match io {
            IO::Input => self.input.as_ref(),
            IO::Output => self.output.as_ref(),
        }
    }
}

/// Opens the input file for reading and returns a boxed trait object that implements the
/// `Read` trait.
///
/// # Arguments
///
/// * `input_file`: An optional string containing the name of the input file to open. If
///   `None`, the function returns a boxed `std::io::Stdin` object.
///
/// # Returns
///
/// A `Result<Box<dyn Read>, String>` containing a boxed trait object that implements the
/// `Read` trait and is connected to the input file, or an error message as a string if the
/// file could not be opened.
///
/// # Errors
///
/// This function can return the following error message:
///
/// * `Failed to open input file: {error_message}` - When an error occurs while opening the
///   input file. The `{error_message}` placeholder is replaced with a string describing the
///   error that occurred.
///
/// # Notes
///
/// This function is used to open the input file for reading and return a boxed trait object
/// that implements the `Read` trait. If the `input_file` argument is `None`, the function
/// returns a boxed `std::io::Stdin` object, which can be used to read from the standard input
/// stream.
///
/// If `input_file` is a `Some` variant containing a string, the function attempts to open the
/// file with the given name and returns a boxed object that implements the `Read` trait and is
/// connected to the input file. If there is an error during file opening, the function returns
/// a `String` containing an error message describing the issue. If the `input_file` argument
/// is `-`, the function returns a boxed `std::io::Stdin` object, regardless of the system's
/// default input file handle.
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

/// Opens the output file for writing and returns a boxed trait object that implements the
/// `Write` trait.
///
/// # Arguments
///
/// * `output_file`: An optional string containing the name of the output file to open. If
///   `None`, the function returns a boxed `std::io::Stdout` object.
///
/// # Returns
///
/// A `Result<Box<dyn Write>, String>` containing a boxed trait object that implements the
/// `Write` trait and is connected to the output file, or an error message as a string if the
/// file could not be opened.
///
/// # Errors
///
/// This function can return the following error message:
///
/// * `Failed to create output file: {error_message}` - When an error occurs while creating the
///   output file. The `{error_message}` placeholder is replaced with a string describing the
///   error that occurred.
///
/// # Notes
///
/// This function is used to open the output file for writing and return a boxed trait object
/// that implements the `Write` trait. If the `output_file` argument is `None`, the function
/// returns a boxed `std::io::Stdout` object, which can be used to write to the standard output
/// stream.
///
/// If `output_file` is a `Some` variant containing a string, the function attempts to create
/// a new file with the given name and returns a boxed object that implements the `Write` trait
/// and is connected to the output file. If there is an error during file creation, the function
/// returns a `String` containing an error message describing the issue. If the `output_file`
/// argument is `-`, the function returns a boxed `std::io::Stdout` object, regardless of the
/// system's default output file handle.
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
