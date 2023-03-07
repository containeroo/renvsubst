use std::collections::HashSet;

/// Prints an error message in red.
pub fn print_error(error: &str) {
    eprintln!("\x1B[31mERROR:\x1B[0m {}", error);
}

#[derive(Debug, Default)]
pub struct Flags {
    pub fail_on_unset: bool,
    pub fail_on_empty: bool,
    pub no_replace_unset: bool,
    pub no_replace_empty: bool,
    pub no_escape: bool,
}

#[derive(Debug, Default)]
pub struct Filters {
    pub prefixes: Option<HashSet<String>>,
    pub suffixes: Option<HashSet<String>>,
    pub variables: Option<HashSet<String>>,
}
