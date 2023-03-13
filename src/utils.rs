/// Prints an error message in red.
pub fn print_error(error: &str) {
    eprintln!("\x1B[31mERROR:\x1B[0m {error}");
}
