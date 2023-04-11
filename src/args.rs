use crate::errors::ParseArgsError;
use crate::filters::{Filter, Filters};
use crate::flags::{Flag, Flags};
use crate::help::HELP_TEXT;
use crate::io::{InputOutput, IO};

/// The `Args` struct represents the parsed command-line arguments for the application.
///
/// * `io`: An `InputOutput` struct containing the input and output sources specified by the user.
/// * `version`: An `Option<String>` containing the version information, if the `--version` flag was used.
/// * `help`: An `Option<String>` containing the help information, if the `--help` flag was used.
/// * `flags`: A `Flags` struct containing the parsed command-line flags and their values.
/// * `filters`: A `Filters` struct containing the filters to be applied to the environment variables.
#[derive(Debug, Default)]
pub struct Args {
    pub io: InputOutput,
    pub version: Option<String>,
    pub help: Option<String>,
    pub flags: Flags,
    pub filters: Filters,
}

impl Args {
    /// Creates a new instance of `Args` with default values.
    ///
    /// * `io`: An empty `InputOutput` object with default values.
    /// * `version`: Set to `None` by default, as the `--version` flag is not set.
    /// * `help`: Set to `None` by default, as the `--help` flag is not set.
    /// * `flags`: A `Flags` object with default values.
    /// * `filters`: A `Filters` object with default values.
    ///
    /// Returns a new `Args` instance with the default configuration.
    fn new() -> Self {
        Args {
            io: InputOutput::default(),
            version: None,
            help: None,
            flags: Flags::default(),
            filters: Filters::default(),
        }
    }

    /// Expands combined short flags (e.g., `-abc`) into separate flags (e.g., `-a -b -c`).
    ///
    /// Takes a flag string `arg` and checks if it has a single hyphen prefix with multiple characters.
    /// If it does and there's no value assigned (e.g., `-a=value`), it expands the combined flag into
    /// separate flags. Otherwise, it returns the original flag with its value (if any).
    ///
    /// # Arguments
    ///
    /// * `arg`: A string slice that represents the input flag.
    ///
    /// # Returns
    ///
    /// A `Vec<(String, Option<&str>)>` containing the expanded flags (or the original flag) with their values.
    ///
    fn expand_combined_flags(arg: &str) -> Vec<(String, Option<&str>)> {
        let mut expanded_flags = Vec::new();

        // Extract flag prefix and flag body
        let (flag_prefix, flag_body) = ["--", "-"]
            .iter() // iterate over the prefixes
            .find_map(
                // find the first prefix that matches the start of the arg
                |prefix| {
                    arg.strip_prefix(prefix).map(
                        // if there is a match, return the prefix and the rest of the arg
                        |stripped| (prefix, stripped), // return the prefix and the rest of the arg
                    )
                },
            )
            .unwrap_or(
                // if there is no match, return an empty string for the prefix and the whole arg
                (&arg, ""),
            );

        // Check if there's an equal sign in the flag body, and separate the flag
        // body into the part before the equal sign and the value part
        let (flag_body_no_value, value) = match flag_body.find('=') {
            Some(index) => (&flag_body[..index], Some(&flag_body[index + 1..])),
            None => (flag_body, None),
        };

        // Determine whether the flag should be expanded (single hyphen with
        // multiple characters and no value)
        let should_expand = flag_prefix == &"-" && flag_body_no_value.len() > 1 && value.is_none();

        // Expand the flag if necessary, or add the flag with its value (if any)
        if should_expand {
            for c in flag_body_no_value.chars() {
                expanded_flags.push((format!("{flag_prefix}{c}"), None));
            }
        } else {
            expanded_flags.push((format!("{flag_prefix}{flag_body_no_value}",), value));
        }

        return expanded_flags;
    }

    /// Parses the command-line arguments and returns an `Args` struct with the parsed values.
    ///
    /// The function takes an iterable of arguments, converts them to strings, and iterates over them.
    /// It expands combined short flags (e.g., `-abc`), and then matches each flag with its
    /// corresponding action (setting a value, enabling a flag, or adding a filter). If an unknown
    /// flag is encountered, a `ParseArgsError` is returned.
    ///
    /// # Type Parameters
    ///
    /// * `I`: An iterable that can be converted into an iterator of type `T`.
    /// * `T`: A type that can be converted into `std::ffi::OsString` and cloned.
    ///
    /// # Arguments
    ///
    /// * `args`: An iterable of command-line arguments.
    ///
    /// # Returns
    ///
    /// A `Result<Args, ParseArgsError>` containing the parsed command-line arguments in an `Args`
    /// struct, or a `ParseArgsError` if there was an issue during parsing.
    ///
    /// # Errors
    ///
    /// This function can return the following errors:
    ///
    /// * `ParseArgsError::UnknownFlag(flag)` - When an unknown flag is encountered.
    /// * `ParseArgsError::MissingValue(flag)` - When a flag requires a value, but it is not provided.
    /// * `ParseArgsError::InvalidValue(flag, value)` - When a flag requires a specific value format, but the provided value is invalid.
    ///
    pub fn parse<I, T>(args: I) -> Result<Args, ParseArgsError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        // Collect the arguments into a vector of strings
        let args: Vec<String> = args
            .into_iter() // Create an iterator over the arguments
            .map(
                // Convert the arguments into OsStrings and then into strings
                |arg| arg.into().into_string(),
            )
            .filter_map(
                // Filter out any arguments that could not be converted into strings
                Result::ok,
            )
            .collect();

        // Create an iterator over the arguments
        let mut args = args.iter();

        let mut parsed_args = Self::new();

        while let Some(arg) = args.next() {
            for (flag, value) in Self::expand_combined_flags(arg) {
                let flag_name = flag.as_str();

                match flag_name {
                    "-h" | "--help" => {
                        parsed_args.help = Some(HELP_TEXT.to_string());
                        return Ok(parsed_args);
                    }
                    "--version" => {
                        parsed_args.version = Some(env!("CARGO_PKG_VERSION").to_string());
                        return Ok(parsed_args);
                    }
                    // INPUT / OUTPUT
                    "-i" | "--input" => {
                        parsed_args.io.set(IO::Input, flag_name, value, &mut args)?;
                    }
                    "-o" | "--output" => {
                        parsed_args
                            .io
                            .set(IO::Output, flag_name, value, &mut args)?;
                    }

                    // FLAGS
                    "-u" | "--fail-on-unset" => {
                        parsed_args.flags.set(Flag::FailOnUnset, flag_name, true)?;
                    }
                    "-e" | "--fail-on-empty" => {
                        parsed_args.flags.set(Flag::FailOnEmpty, flag_name, true)?;
                    }
                    "-f" | "--fail" => {
                        parsed_args.flags.set(Flag::Fail, flag_name, true)?;
                    }
                    "-U" | "--no-replace-unset" => {
                        parsed_args
                            .flags
                            .set(Flag::NoReplaceUnset, flag_name, true)?;
                    }
                    "-E" | "--no-replace-empty" => {
                        parsed_args
                            .flags
                            .set(Flag::NoReplaceEmpty, flag_name, true)?;
                    }
                    "-N" | "--no-replace" => {
                        parsed_args.flags.set(Flag::NoReplace, flag_name, true)?;
                    }
                    "-x" | "--no-escape" => {
                        parsed_args.flags.set(Flag::NoEscape, flag_name, true)?;
                    }
                    "-b" | "--unbuffer-lines" => {
                        parsed_args
                            .flags
                            .set(Flag::UnbufferedLines, flag_name, true)?;
                    }
                    "-c" | "--color" => {
                        parsed_args.flags.set(Flag::Color, flag_name, true)?;
                    }

                    // FILTERS
                    "-p" | "--prefix" => {
                        parsed_args
                            .filters
                            .add(Filter::Prefix, flag_name, value, &mut args)?;
                    }
                    "-s" | "--suffix" => {
                        parsed_args
                            .filters
                            .add(Filter::Suffix, flag_name, value, &mut args)?;
                    }
                    "-v" | "--variable" => {
                        parsed_args
                            .filters
                            .add(Filter::Variable, flag_name, value, &mut args)?;
                    }
                    // UNKNOWN
                    _ => return Err(ParseArgsError::UnknownFlag(flag)),
                }
            }
        }

        return Ok(parsed_args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let args = vec!["--no-replace-empty", "--prefix", "prefix-"];
        let parsed_args = Args::parse(args).unwrap();

        assert_eq!(
            parsed_args
                .flags
                .get(Flag::NoReplaceEmpty)
                .and_then(|f| f.value),
            Some(true)
        );

        assert!(parsed_args.filters.prefixes.unwrap().contains("prefix-"),);
    }

    #[test]
    fn test_parse_unknown_flag() {
        let args = vec!["--invalid-flag"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::UnknownFlag("--invalid-flag".to_string())
        );
    }

    #[test]
    fn test_parse_help_flag() {
        let args = vec!["-h"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.help.is_some());
    }

    #[test]
    fn test_parse_conflicting_flags() {
        let args = vec!["--fail-on-unset", "--no-replace-unset"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--fail-on-unset".to_string(),
            )
        );
    }

    #[test]
    fn test_parse_conflicting_flags2() {
        let args = vec!["--no-replace-unset", "--fail-on-unset"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--no-replace-unset".to_string()
            )
        );
    }

    #[test]
    fn test_parse_unbuffered_lines() {
        let args = vec!["--unbuffer-lines"];
        let parsed_args = Args::parse(args).unwrap();

        assert_eq!(
            parsed_args
                .flags
                .get(Flag::UnbufferedLines)
                .and_then(|f| f.value),
            Some(true)
        );
    }

    #[test]
    fn test_parse_missing_value_prefix() {
        let args = vec!["--prefix"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::MissingValue("--prefix".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_suffix() {
        let args = vec!["--suffix"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::MissingValue("--suffix".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_variable() {
        let args = vec!["--variable"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::MissingValue("--variable".to_string())
        );
    }

    #[test]
    fn test_filters_new() {
        let filters = Filters::default();

        assert!(filters.prefixes.is_none());
        assert!(filters.suffixes.is_none());
        assert!(filters.variables.is_none());
    }

    #[test]
    fn test_filters_prefix_space() {
        let args = vec!["--prefix", "prefix-", "--no-replace-empty"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.prefixes.unwrap().contains("prefix-"),);
    }

    #[test]
    fn test_filters_prefix_equal() {
        let args = vec!["--prefix=prefix-"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.prefixes.unwrap().contains("prefix-"),);
    }

    #[test]
    fn test_filters_multiple_same_prefix_equal() {
        let args = vec!["--prefix=prefix-", "--prefix=prefix-"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.prefixes.unwrap().contains("prefix-"),);
    }

    #[test]
    fn test_filters_suffix_space() {
        let args = vec!["--suffix", "-suffix"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.suffixes.unwrap().contains("-suffix"),);
    }

    #[test]
    fn test_filters_suffix_equal() {
        let args = vec!["--suffix=-suffix"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.suffixes.unwrap().contains("-suffix"),);
    }

    #[test]
    fn test_filters_variable_space() {
        let args = vec!["--variable", "VAR"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.variables.unwrap().contains("VAR"));
    }

    #[test]
    fn test_filters_variable_equal() {
        let args = vec!["--variable=VAR"];
        let parsed_args = Args::parse(args).unwrap();

        assert!(parsed_args.filters.variables.unwrap().contains("VAR"));
    }

    #[test]
    fn test_flags_fail_fail_on_empty() {
        let args = vec!["--fail", "--fail-on-empty"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("--fail-on-empty".to_string(), "--fail".to_string())
        );
    }

    #[test]
    fn test_flags_fail_fail_on_unset() {
        let args = vec!["--fail", "--fail-on-unset"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("--fail-on-unset".to_string(), "--fail".to_string())
        );
    }

    #[test]
    fn test_flags_no_replace_no_replace_unset() {
        let args = vec!["--no-replace", "--no-replace-unset"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string()
            )
        );
    }

    #[test]
    fn test_flags_no_replace_no_replace_empty() {
        let args = vec!["--no-replace", "--no-replace-empty"];
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string()
            )
        );
    }

    #[test]
    fn test_version() {
        let args = vec!["--version"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args.unwrap().version.is_some());
    }

    #[test]
    fn test_no_escape() {
        let args = vec!["--no-escape"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert_eq!(
            parsed_args
                .unwrap()
                .flags
                .get(Flag::NoEscape)
                .and_then(|f| f.value),
            Some(true)
        );
    }

    #[test]
    fn test_parse_prefix_equal() {
        let args = vec!["--prefix=prefix-"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .prefixes
            .unwrap()
            .contains("prefix-"));
    }

    #[test]
    fn test_parse_prefix_space() {
        let args = vec!["--prefix", "prefix-"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .prefixes
            .unwrap()
            .contains("prefix-"));
    }

    #[test]
    fn test_parse_prefix_multiple() {
        let args = vec!["--prefix", "prefix-", "--prefix", "prefix-"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .prefixes
            .unwrap()
            .contains("prefix-"));
    }

    #[test]
    fn test_parse_suffix_equal() {
        let args = vec!["--suffix=-suffix"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .suffixes
            .unwrap()
            .contains("-suffix"));
    }

    #[test]
    fn test_parse_suffix_space() {
        let args = vec!["--suffix", "-suffix"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .suffixes
            .unwrap()
            .contains("-suffix"));
    }

    #[test]
    fn test_parse_suffix_multiple() {
        let args = vec!["--suffix", "-suffix", "--suffix", "-suffix"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .suffixes
            .unwrap()
            .contains("-suffix"));
    }

    #[test]
    fn test_parse_variable_equal() {
        let args = vec!["--variable=VAR"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .variables
            .unwrap()
            .contains("VAR"));
    }

    #[test]
    fn test_parse_variable_space() {
        let args = vec!["--variable", "VAR"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .variables
            .unwrap()
            .contains("VAR"));
    }

    #[test]
    fn test_parse_variable_multiple() {
        let args = vec!["--variable", "VAR", "--variable", "VAR"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());
        assert!(parsed_args
            .unwrap()
            .filters
            .variables
            .unwrap()
            .contains("VAR"));
    }

    #[test]
    fn test_parse_variable_multiple_different() {
        let args = vec!["--variable", "VAR", "--variable", "VAR2"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());

        let parsed_args = parsed_args.unwrap(); // Assign the unwrapped value to a variable
        let variables = &parsed_args.filters.variables.unwrap();

        assert!(variables.contains("VAR"));
        assert!(variables.contains("VAR2"));
    }

    #[test]
    fn test_parse_variable_multiple_different_order() {
        let args = vec!["--variable", "VAR2", "--variable", "VAR"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_ok());

        let parsed_args = parsed_args.unwrap(); // Assign the unwrapped value to a variable
        let variables = &parsed_args.filters.variables.unwrap();

        assert!(variables.contains("VAR"));
        assert!(variables.contains("VAR2"));
    }

    #[test]
    fn test_parse_prefix_arg() {
        let args = vec!["-p", "prefix_value"];
        let parsed_args = Args::parse(args).unwrap();
        assert!(parsed_args
            .filters
            .prefixes
            .unwrap()
            .contains("prefix_value"));
    }

    #[test]
    fn test_parse_shorts_fail_unset_fail_empty() {
        let args = vec!["-uexb"];
        let parsed_args = Args::parse(args);
        let result = parsed_args.unwrap();
        assert_eq!(
            result.flags.get(Flag::FailOnUnset).and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result.flags.get(Flag::FailOnEmpty).and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result
                .flags
                .get(Flag::UnbufferedLines)
                .and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result.flags.get(Flag::NoEscape).and_then(|f| f.value),
            Some(true)
        );
    }

    #[test]
    fn test_parse_shorts_long() {
        let args = vec!["-uex", "--unbuffer-lines"];
        let parsed_args = Args::parse(args);
        let result = parsed_args.unwrap();
        assert_eq!(
            result.flags.get(Flag::FailOnUnset).and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result.flags.get(Flag::FailOnEmpty).and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result
                .flags
                .get(Flag::UnbufferedLines)
                .and_then(|f| f.value),
            Some(true)
        );
        assert_eq!(
            result.flags.get(Flag::NoEscape).and_then(|f| f.value),
            Some(true)
        );
    }

    #[test]
    fn test_parse_shorts_unwknown() {
        let args = vec!["-uen"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::UnknownFlag("-n".to_string())
        );
    }

    #[test]
    fn test_parse_shorts_conflict() {
        let args = vec!["-uef"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("-f".to_string(), "-u".to_string())
        );
    }

    #[test]
    fn test_parse_shorts_long_mixed_conflict() {
        let args = vec!["-ue", "--fail"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("--fail".to_string(), "-u".to_string())
        );
    }

    #[test]
    fn test_input_space() {
        let args = vec!["--input", "input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some(&String::from("input_file")),
        );
    }

    #[test]
    fn test_input_equal() {
        let args = vec!["--input=input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some(&String::from("input_file")),
        );
    }

    #[test]
    fn test_input_equal2() {
        let args = vec!["--input=-"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(parsed_args.io.get(IO::Input), Some(&String::from("-")),);
    }

    #[test]
    fn test_input_equal_short() {
        let args = vec!["-i=input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some(&String::from("input_file")),
        );
    }

    #[test]
    fn test_input_equal_equal_in_value_short() {
        let args = vec!["-i=input=file.txt"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some(&String::from("input=file.txt")),
        );
    }

    #[test]
    fn test_input_error() {
        let args = vec!["--input"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::MissingValue("--input".to_string())
        );
    }

    #[test]
    fn test_output_space() {
        let args = vec!["--output", "output_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Output),
            Some(&String::from("output_file")),
        );
    }

    #[test]
    fn test_output_equal() {
        let args = vec!["--output=output_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Output),
            Some(&String::from("output_file")),
        );
    }

    #[test]
    fn test_output_equal_short() {
        let args = vec!["-o=output_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Output),
            Some(&String::from("output_file")),
        );
    }

    #[test]
    fn test_output_error() {
        let args = vec!["--output"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::MissingValue("--output".to_string())
        );
    }

    #[test]
    fn test_flags_conflict_short_long() {
        let args = vec!["-U", "--fail-on-unset"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("--fail-on-unset".to_string(), "-U".to_string())
        );
    }

    #[test]
    fn test_flags_other_conflict_short_long() {
        let args = vec!["--unbuffer-lines", "-U", "--fail-on-unset"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags("--fail-on-unset".to_string(), "-U".to_string())
        );
    }

    #[test]
    fn test_flags_other_conflict_long_long() {
        let args = vec!["--unbuffer-lines", "--no-replace-unset", "--fail-on-unset"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--no-replace-unset".to_string()
            )
        );
    }

    #[test]
    fn test_no_replace_duplicate() {
        let args = vec!["--no-replace-unset", "--no-replace-unset"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::DuplicateFlag("--no-replace-unset".to_string())
        );
    }

    #[test]
    fn test_color() {
        let args = vec!["--color"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.flags.get(Flag::Color).and_then(|f| f.value),
            Some(true)
        );
    }
}
