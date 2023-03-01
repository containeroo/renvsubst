mod args;
mod file_io;
mod substitute;
use crate::args::parse_args;
use crate::file_io::{open_input_file, open_output_file};
use crate::substitute::perform_substitution;

const VERSION: &str = "0.2.0";

/// The entry point of the program.
///
/// This function parses the command line arguments, opens the input and output files
/// (or uses stdin/stdout if not specified), and performs a substitution operation on
/// the input data according to the provided flags and filters.
///
/// # Examples
///
/// ```
/// // Perform a substitution on the contents of `input.txt`, writing the result to `output.txt`
/// renvsubst -i input.txt -o output.txt
/// ```
fn main() {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    // print version and exit if requested
    if args.version.is_some() {
        println!("renvsubst {}", args.version.unwrap());
        std::process::exit(0);
    }

    // print help and exit if requested
    if args.help.is_some() {
        println!("{}", args.help.unwrap());
        std::process::exit(0);
    }

    // open input file, can be stdin
    let input_file = open_input_file(args.input_file).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });

    // create output file, can be stdout
    let output_file = open_output_file(args.output_file).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });

    perform_substitution(input_file, output_file, &args.flags, &args.filters).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });
}
