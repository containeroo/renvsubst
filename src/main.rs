mod args;
mod errors;
mod filters;
mod flags;
mod help;
mod substitute;
mod utils;

use crate::args::Args;
use crate::substitute::process_input;
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
    let parsed_args = Args::parse(args).map_err(|e| e.to_string())?;

    if let Some(version) = &parsed_args.version {
        println!("{version}");
        return Ok(());
    }

    if let Some(help) = &parsed_args.help {
        println!("{help}");
        return Ok(());
    }

    process_input(
        Box::new(std::io::stdin()),
        Box::new(std::io::stdout()),
        &parsed_args.flags,
        &parsed_args.filters,
    )
}

#[cfg(not(tarpaulin_include))]
fn main() {
    let args = std::env::args()
        .skip(1) // skip(1) to skip the program name
        .collect::<Vec<String>>();

    match run(&args) {
        Ok(_) => (),
        Err(err) => {
            print_error(&err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_version() {
        let args = vec![String::from("--version")];
        let output = run(&args).unwrap();

        // The run function should return Ok(()) when the --version flag is provided
        assert_eq!(output, ());
    }

    #[test]
    fn test_run_help() {
        let args = vec![String::from("--help")];
        let output = run(&args).unwrap();

        assert_eq!(output, ());
    }

    #[test]
    fn test_example() {
        let args = vec![String::from("--version")];
        let result = run(&args);
        assert!(result.is_ok());
    }
}
