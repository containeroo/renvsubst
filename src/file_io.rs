use crate::errors::ParseArgsError;
use crate::utils::START_PARAMETERS;
use std::fs::File;
use std::io::{BufReader, Read, Write};

#[derive(Debug, Default)]
pub struct InputOutput {
    input: Option<String>,
    output: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum IO {
    Input,
    Output,
}

impl InputOutput {
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

    pub fn get(&self, io: IO) -> Option<String> {
        match io {
            IO::Input => self.input.clone(),
            IO::Output => self.output.clone(),
        }
    }
}

// function to open input file, if provided, otherwise use stdin
pub fn open_input(input_file: Option<String>) -> Result<Box<dyn Read>, String> {
    // check if file is None or "-"
    let file = match input_file {
        Some(file) if file == "-" => return Ok(Box::new(std::io::stdin())),
        Some(file) => file,
        None => return Ok(Box::new(std::io::stdin())),
    };

    // open input file
    match File::open(file) {
        Ok(file) => Ok(Box::new(BufReader::new(file))),
        Err(e) => Err(format!("Failed to open input file: {}", e)),
    }
}

// function to open output file, if provided, otherwise use stdout
pub fn open_output(output_file: Option<String>) -> Result<Box<dyn Write>, String> {
    // check if file is None or "-"
    let file = match output_file {
        Some(file) if file == "-" => return Ok(Box::new(std::io::stdout())),
        Some(file) => file,
        None => return Ok(Box::new(std::io::stdout())),
    };

    // create output file
    match File::create(file) {
        Ok(file) => Ok(Box::new(file)),
        Err(e) => Err(format!("Failed to create output file: {}", e)),
    }
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
