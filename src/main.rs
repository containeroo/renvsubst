mod args;
mod filters;
mod flags;
mod substitute;
mod utils;
use crate::args::{Args, HELP_TEXT};
use crate::substitute::perform_substitution;
use crate::utils::print_error;
use std::env;
mod errors;

fn main() {
    // parse command line arguments
    let parsed_args = Args::parse(env::args().skip(1)).unwrap_or_else(|e| {
        print_error(&e.to_string());
        std::process::exit(1);
    });

    // print version and exit if requested
    if parsed_args.version {
        println!("renvsubst {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // print help and exit if requested
    if parsed_args.help {
        println!("{HELP_TEXT}");
        std::process::exit(0);
    }

    perform_substitution(
        Box::new(std::io::stdin()),
        Box::new(std::io::stdout()),
        &parsed_args.flags,
        &parsed_args.filters,
    )
    .unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });
}
