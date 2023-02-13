mod args;
mod file_io;
mod substitute;
use crate::file_io::{open_input_file, open_output_file};

use args::{get_args, Args};
use substitute::perform_substitution;

const VERSION: &str = "0.1.6";

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let Args {
        input_file,
        output_file,
        flags,
        filters,
    } = get_args();

    // open input file, can be stdin
    let input_file = open_input_file(input_file).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });

    // create output file, can be stdout
    let output_file = open_output_file(output_file).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });

    perform_substitution(input_file, output_file, &flags, &filters).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    });
}
