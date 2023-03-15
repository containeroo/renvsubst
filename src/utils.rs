use colored::Colorize;

/// Prints an error message in red.
pub fn print_error(error: &str) {
    eprintln!("{} {}", "ERROR:".red(), error.red());
}
