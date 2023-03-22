mod args;
mod env_subst;
mod errors;
mod filters;
mod flags;
mod help;
mod io;
mod utils;

use crate::args::Args;
use crate::env_subst::process_input;
use crate::flags::Flag;
use crate::io::{open_input, open_output, IO};
use crate::utils::print_error;

/// Executes the main logic of the application based on the given command-line arguments.
///
/// This function is responsible for parsing the command-line arguments, handling
/// version and help flags, and calling the `perform_substitution` function with
/// the appropriate input and output streams as well as the parsed flags and filters.
///
/// # Arguments
///
/// * `args` - A slice of strings representing the command-line arguments passed to the program.
///
/// # Returns
///
/// * `Ok(())` - If the application successfully executes its tasks.
/// * `Err(String)` - If there is an error during execution, containing a description of the error.
///
/// # Examples
///
/// ```
/// let args = vec![String::from("--version")];
/// let result = run(&args);
/// assert!(result.is_ok());
/// ```
fn run(args: &[String]) -> Result<(), String> {
    let mut parsed_args = Args::parse(args).map_err(|e| e.to_string())?;

    if let Some(version) = &parsed_args.version {
        println!("{version}");
        return Ok(());
    }

    if let Some(help) = &parsed_args.help {
        println!("{help}");
        return Ok(());
    }

    // create input and output streams
    let input = open_input(parsed_args.io.get(IO::Input))?;
    let output = open_output(parsed_args.io.get(IO::Output))?;

    // check if output is stdout and disable color if so
    // otherwise, the content in the output file will be wrong
    if parsed_args
        .io
        .get(IO::Output)
        .map(|s| s != String::from("-"))
        .unwrap_or(false)
    {
        parsed_args.flags.update(Flag::Color, false);
    }

    process_input(input, output, &parsed_args.flags, &parsed_args.filters)
}

#[cfg(not(tarpaulin_include))]
fn main() {
    let args = std::env::args()
        .skip(1) // skip(1) to skip the program name
        .collect::<Vec<String>>();

    run(&args).unwrap_or_else(|err| {
        print_error(&err);
        std::process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_run_version() {
        let args = vec![String::from("--version")];
        assert!(run(&args).is_ok());
    }

    #[test]
    fn test_run_help() {
        let args = vec![String::from("--help")];
        assert!(run(&args).is_ok());
    }

    #[test]
    fn test_example() {
        let args = vec![String::from("--version")];
        let result = run(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_input_file() {
        // create a temp file
        let input_file = NamedTempFile::new().unwrap();
        let input_file_path = input_file.path().to_str().unwrap().to_string();

        // add some text to input file
        let mut input = File::create(&input_file_path).unwrap();
        let buf = b"Hello, world!";
        input.write_all(buf).unwrap();

        // create a temp file for output
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path().to_str().unwrap().to_string();

        let args = vec![
            String::from("--input"),
            input_file_path,
            String::from("--output"),
            output_file_path,
        ];

        let result = run(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_input_error() {
        let args = vec![
            String::from("--input"),
            String::from("nonexistent_file.txt"),
        ];
        let result = run(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_output_file() {
        // create a temp file for input
        let input_file = NamedTempFile::new().unwrap();
        let input_file_path = input_file.path().to_str().unwrap().to_string();

        // add some text to input file
        let mut file = File::create(&input_file_path).unwrap();
        let buf = b"Hello, ${NOT_FOUND_VAR:-world}!";
        file.write_all(buf).unwrap();

        // create a temp file for output
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path().to_str().unwrap().to_string();

        let args = vec![
            String::from("--input"),
            input_file_path,
            String::from("--output"),
            output_file_path.clone(),
        ];
        let result = run(&args);

        // read output file
        let file = File::open(output_file_path).unwrap();
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();

        assert!(result.is_ok()); // check if run() was successful
        assert_eq!(contents, "Hello, world!"); // check if output file contains the correct text
    }

    #[test]
    fn test_run_output_error() {
        let args = vec![
            String::from("--output"),
            String::from("/nonexistent_dir/output.txt"),
        ];
        let result = run(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_colore_and_file() {
        // create a temp file for input
        let input_file = NamedTempFile::new().unwrap();
        let input_file_path = input_file.path().to_str().unwrap().to_string();

        // add some text to input file
        let mut file = File::create(&input_file_path).unwrap();
        let buf = b"Hello, ${NOT_FOUND_VAR:-world}!";
        file.write_all(buf).unwrap();

        // create a temp file for output
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path().to_str().unwrap().to_string();

        let args = vec![
            String::from("--input"),
            input_file_path,
            String::from("--output"),
            output_file_path.clone(),
            String::from("--color"),
        ];
        let result = run(&args);

        // read output file
        let file = File::open(output_file_path).unwrap();
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();

        assert!(result.is_ok()); // check if run() was successful
        assert_eq!(contents, "Hello, world!"); // check if output file contains the correct text
    }

}
