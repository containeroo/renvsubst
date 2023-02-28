use std::fs::File;
use std::io::{BufReader, Read, Write};

/// Opens an input file and returns a boxed reader. If the input file is not
/// provided or is "-", stdin is used instead.
///
/// # Arguments
///
/// * `input_file` - An optional string containing the file path to open. If the string is `None` or `"-"`, then standard input is used instead.
///
/// # Returns
///
/// Returns a boxed `Read` trait object on success, or an error string on failure.
///
/// # Errors
///
/// This function returns an error string if the input file fails to open for any reason.
///
/// # Examples
///
/// ```
/// use std::io::Read;
///
/// let file_path = "input.txt";
/// let input_file = Some(file_path.to_string());
/// let mut reader = match open_input_file(input_file) {
///     Ok(reader) => reader,
///     Err(e) => panic!("Failed to open input file: {}", e),
/// };
/// let mut buffer = String::new();
/// reader.read_to_string(&mut buffer).expect("Failed to read input file");
/// ```
pub fn open_input_file(input_file: Option<String>) -> Result<Box<dyn Read>, String> {
  // check if file is None or "-"
  let file_name = match &input_file {
      Some(file) if file == "-" => return Ok(Box::new(std::io::stdin())),
      Some(file) => file,
      None => return Ok(Box::new(std::io::stdin())),
  };

  // open input file
  match File::open(file_name) {
      Ok(file) => Ok(Box::new(BufReader::new(file))),
      Err(e) => Err(format!("Failed to open input file '{}': {}", file_name, e)),
  }
}

/// Opens an output file and returns a boxed writer. If the output file is not
/// provided or is "-", stdout is used instead.
///
/// # Arguments
///
/// * `output_file` - An optional string containing the file path to create. If the string is `None` or `"-"`, then standard output is used instead.
///
/// # Returns
///
/// Returns a boxed `Write` trait object on success, or an error string on failure.
///
/// # Errors
///
/// This function returns an error string if the output file fails to create for any reason.
///
/// # Examples
///
/// ```
/// use std::io::Write;
///
/// let file_path = "output.txt";
/// let output_file = Some(file_path.to_string());
/// let mut writer = match open_output_file(output_file) {
///     Ok(writer) => writer,
///     Err(e) => panic!("Failed to create output file: {}", e),
/// };
/// writer.write_all(b"Hello, world!").expect("Failed to write to output file");
/// ```
pub fn open_output_file(output_file: Option<String>) -> Result<Box<dyn Write>, String> {
    // check if file is None or "-"
    let file_name = match &output_file {
        Some(file) if file == "-" => return Ok(Box::new(std::io::stdout())),
        Some(file) => file,
        None => return Ok(Box::new(std::io::stdout())),
    };

    // create output file
    match File::create(file_name) {
        Ok(file) => Ok(Box::new(file)),
        Err(e) => Err(format!("Failed to create output file '{}'. {}", file_name, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_input_file_with_empty() {
        let input = None;
        let result = open_input_file(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_input_file_with_existent_file() {
        let input_file = NamedTempFile::new().unwrap();
        let input_file_path = input_file.path().to_str().unwrap().to_string();
        let result = open_input_file(input_file_path.into());
        input_file.close().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_input_file_with_nonexistent_file() {
        let input = String::from("tests/nonexistent_file.txt");
        let result = open_input_file(input.into());
        assert!(result.is_err());
    }

    #[test]
    fn test_open_output_file_with_stdout() {
        let output = None;
        let result = open_output_file(output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_output_file_with_nonexistent_directory() {
        let output = Some(String::from("tests/nonexistent_folder/output_file.txt"));
        let result = open_output_file(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_output_file_with_existent_file() {
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path().to_str().unwrap().to_string();
        let result = open_output_file(Some(output_file_path));
        output_file.close().unwrap();
        assert!(result.is_ok());
    }

}
