use colored::Colorize;

/// Prints an error message in red.
#[cfg(not(tarpaulin_include))]
pub fn print_error(error: &str) {
    eprintln!("{} {}", "ERROR:".red(), error.red());
}
