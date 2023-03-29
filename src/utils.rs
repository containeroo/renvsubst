use colored::{Color, Colorize};

/// Prints an error message in red.
#[cfg(not(tarpaulin_include))]
pub fn print_error(error: &str) {
    eprintln!("{} {}", "ERROR:".red(), error.red());
}

#[must_use]
pub fn colorize_text(colored: bool, text: String, color: Color) -> String {
    if colored {
        return text.color(color).to_string();
    }
    return text;
}

/// List with all the parameters that can be used to start the program.
/// This is used to check if the value of a flag is another flag.
pub const START_PARAMETERS: &[&str] = &[
    "-i",
    "--input",
    "-o",
    "--output",
    "-h",
    "--help",
    "--version",
    "-u",
    "--fail-on-unset",
    "-e",
    "--fail-on-empty",
    "-f",
    "--fail",
    "-U",
    "--no-replace-unset",
    "-E",
    "--no-replace-empty",
    "-N",
    "--no-replace",
    "-x",
    "--no-escape",
    "-b",
    "--unbuffer-lines",
    "-p",
    "--prefix",
    "-s",
    "--suffix",
    "-v",
    "--variable",
    "-c",
    "--color",
];
