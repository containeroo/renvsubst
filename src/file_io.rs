use std::fs::File;
use std::io::{BufReader, Read, Write};

// function to open input file, if provided, otherwise use stdin
pub fn open_input_file(input_file: Option<String>) -> Result<Box<dyn Read>, String> {
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
pub fn open_output_file(output_file: Option<String>) -> Result<Box<dyn Write>, String> {
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
