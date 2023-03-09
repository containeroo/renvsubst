mod args;
mod file_io;
mod substitute;
mod utils;
use crate::args::{Args, HELP_TEXT};
use crate::file_io::{open_input_file, open_output_file};
use crate::substitute::perform_substitution;
use crate::utils::print_error;
use std::env;

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    // parse command line arguments
    let args = Args::parse(env::args().skip(1)).unwrap_or_else(|e| {
        print_error(&e.to_string());
        std::process::exit(1);
    });

    // print version and exit if requested
    if args.version {
        println!("renvsubst {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // print help and exit if requested
    if args.help {
        println!("{}", HELP_TEXT);
        std::process::exit(0);
    }

    // open input file, can be stdin
    let input_file = open_input_file(args.input_file).unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });

    // create output file, can be stdout
    let output_file = open_output_file(args.output_file).unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });

    perform_substitution(input_file, output_file, &args.flags, &args.filters).unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });
}
