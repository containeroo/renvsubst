use crate::flags::{Flag, Flags};
use colored::{Color, Colorize};

/// Prints an error message in red.
#[cfg(not(tarpaulin_include))]
pub fn print_error(error: &str) {
    eprintln!("{} {}", "ERROR:".red(), error.red());
}

pub fn colorize_text(colored: bool, text: String, color: Color) -> String {
    if colored {
        return text.color(color).to_string();
    }
    return text;
}

/// This function handles the application of certain flags on the result of a variable replacement. These
/// flags include `Fail`, `FailOnEmpty`, `FailOnUnset`, `NoReplaceUnset`, and `NoReplaceEmpty`. The function takes in
/// the result of a variable replacement, the variable name, the original variable string, and the flags
/// object. It then returns the modified result or an error message depending on the flag settings.
///
/// # Arguments
///
/// * `result` - A String containing the result of the variable replacement.
/// * `var_name` - A string slice containing the name of the environment variable being replaced.
/// * `original_variable` - A string slice containing the original variable string that was replaced.
/// * `flags` - A &Flags object containing the flag settings for the program.
///
/// # Returns
///
/// * Result<String, String> - A Result containing either the modified result as a String on success
/// or an error message as a String on failure.
///
/// # Errors
///
/// This function will return an error message if the `Fail` or `FailOnEmpty` flags are set and the result is empty,
/// if the `FailOnUnset` flag is set and the environment variable is not set, or if the `NoReplaceUnset` or `NoReplaceEmpty`
/// flags are set and the result is empty.
pub fn handle_flags_on_result(
    result: String,
    var_name: &str,
    original_variable: &str,
    flags: &Flags,
) -> Result<String, String> {
    // Check for Fail and FailOnUnset flags
    if result.is_empty() && (flags.is_flag_set(Flag::Fail) || flags.is_flag_set(Flag::FailOnUnset))
    {
        return Err(format!("environment variable '{var_name}' is not set"));
    }

    // Check for FailOnEmpty flag
    if result.is_empty() && (flags.is_flag_set(Flag::Fail) || flags.is_flag_set(Flag::FailOnEmpty))
    {
        return Err(format!("environment variable '{var_name}' is empty"));
    }

    // Check for NoReplaceUnset flag
    if result.is_empty()
        && (flags.is_flag_set(Flag::NoReplace) || flags.is_flag_set(Flag::NoReplaceUnset))
    {
        return Ok(colorize_text(
            flags.is_flag_set(Flag::Color),
            original_variable.to_string(),
            Color::Red,
        ));
    }

    // Check for NoReplaceEmpty flag
    if result.is_empty()
        && (flags.is_flag_set(Flag::NoReplace) || flags.is_flag_set(Flag::NoReplaceEmpty))
    {
        return Ok(colorize_text(
            flags.is_flag_set(Flag::Color),
            original_variable.to_string(),
            Color::Red,
        ));
    }

    // Return the modified result
    return Ok(result);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_text() {
        // Test default behavior
        let text = "This is a test.";
        let result = colorize_text(false, text.to_string(), Color::Green);
        assert_eq!(result, "This is a test.");

        // Test colorize
        let result = colorize_text(true, text.to_string(), Color::Green);
        assert_eq!(result, "\u{1b}[32mThis is a test.\u{1b}[0m".to_string());
    }

    #[cfg(test)]
    #[test]
    fn test_handle_flags_on_result() {
        let var_name = "VAR";
        let inner_expr = "VAR";
        let mut flags = Flags::default();
        let original_variable = "$VAR".to_string();

        // Test without any flags
        let result =
            handle_flags_on_result("Hello".to_string(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), "Hello");

        // Test with Fail flag and non-empty result
        flags
            .set(Flag::Fail, "--fail", true)
            .expect("Failed to set Fail flag");
        let result =
            handle_flags_on_result("Hello".to_string(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), "Hello");

        // Test with Fail flag and empty result
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert!(result.is_err());

        // Test with FailOnEmpty flag and empty result
        flags = Flags::default();
        flags
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .expect("Failed to set FailOnEmpty flag");

        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert!(result.is_err());

        // Test with FailOnUnset flag and unset variable
        flags = Flags::default();
        flags
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .expect("Failed to set FailOnUnset flag");
        let result = handle_flags_on_result(String::new(), "UNSET_VAR", &original_variable, &flags);
        assert!(result.is_err());

        // Test with NoReplace flag
        flags = Flags::default();
        flags
            .set(Flag::NoReplace, "--no-replace", true)
            .expect("Failed to set NoReplace flag");
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));

        // Test with NoReplaceEmpty flag and empty result
        flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .expect("Failed to set NoReplaceEmpty flag");
        let result = handle_flags_on_result(String::new(), var_name, &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));

        // Test with NoReplaceUnset flag and unset variable
        flags = Flags::default();
        flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .expect("Failed to set NoReplaceUnset flag");
        let result = handle_flags_on_result(String::new(), "UNSET_VAR", &original_variable, &flags);
        assert_eq!(result.unwrap(), format!("${inner_expr}"));
    }
}
