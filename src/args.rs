use crate::errors::ParseArgsError;
use crate::filters::{Filter, Filters};
use crate::flags::{Flag, Flags};
use crate::help::HELP_TEXT;
use crate::io::{InputOutput, IO};

/// A struct representing the command-line arguments for a program.
///
/// The `version` field is an optional `String` that can be used to display the version of the program.
///
/// The `help` field is an optional `String` that can be used to display help text for the program.
///
/// The `flags` field is a `Flags` struct that contains boolean flags that can be set by the user. The `Flags` struct is defined in the `flags` module.
///
/// The `filters` field is a `Filters` struct that contains optional filters for matching strings. The `Filters` struct is defined in the `filters` module.
#[derive(Debug, Default)]
pub struct Args {
    pub io: InputOutput,
    pub version: Option<String>,
    pub help: Option<String>,
    pub flags: Flags,
    pub filters: Filters,
}

impl Args {
    /// Creates a new instance of Args with all fields set to their default values.
    fn new() -> Self {
        Args {
            io: InputOutput::default(),
            version: None,
            help: None,
            flags: Flags::default(),
            filters: Filters::default(),
        }
    }

    /// Parses command line arguments and returns an `Args` struct with the parsed values.
    ///
    /// # Arguments
    ///
    /// `args` - An iterator over the command line arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::args::Args;
    ///
    /// let args = vec!["--prefix", "VAR_", "--suffix", "_SUFFIX"];
    /// let parsed_args = Args::parse(args.iter()).unwrap();
    /// assert_eq!(Args::parsed_args.filters.prefixes.unwrap().len(), 1);
    /// assert_eq!(Args::parsed_args.filters.suffixes.unwrap().len(), 1);
    /// ```
    ///
    /// # Flags
    ///
    /// - `--fail-on-unset` - Fail if an environment variable is not set.
    /// - `--fail-on-empty` - Fail if an environment variable is empty.
    /// - `--fail` - Alias for `--fail-on-unset` and `--fail-on-empty`. Fails if an environment variable is either not set or empty.
    /// - `--no-replace-unset` - Do not replace variables that are not set in the environment.
    /// - `--no-replace-empty` - Do not replace variables that are set but empty in the environment.
    /// - `--no-replace` - Alias for `--no-replace-unset` and `--no-replace-empty`. Does not replace variables that are either not set or empty in the environment.
    /// - `--no-escape` - Disable escaping of variables with two dollar signs (`$$`).
    ///  - `--unbuffer-lines` - Do not buffer lines. This is useful when using `renvsubst` in a pipeline.
    /// - `-h`, `--help` - Show the help text.
    /// - `-v`, `--version` - Show the version of the program.
    ///
    /// # Filters
    ///
    /// - `-p`, `--prefix [PREFIX]...` - Only replace variables with the specified prefix. Prefixes can be specified multiple times.
    /// - `-s`, `--suffix [SUFFIX]...` - Only replace variables with the specified suffix. Suffixes can be specified multiple times.
    /// - `-v`, `--variable [VARIABLE]...` - Specify the variables to replace. If not provided, all variables will be replaced. Variables can be specified multiple times.
    ///
    /// The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.
    ///
    /// # Errors
    ///
    /// This function returns a `ParseArgsError` if an error occurs during parsing.
    pub fn parse<I, T>(args: I) -> Result<Args, ParseArgsError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        // collect the arguments into a vector of strings
        let args: Vec<String> = args
            .into_iter()
            .map(|arg| arg.into().into_string())
            .filter_map(Result::ok)
            .collect();

        // create an iterator over the arguments
        let mut args = args.iter();

        let mut parsed_args = Self::new();

        while let Some(arg) = args.next() {
            // Split the argument by the first occurrence of '='
            let (flag_name, value) = match arg.find('=') {
                Some(index) => (&arg[0..index], Some(&arg[(index + 1)..arg.len()])),
                None => (arg.as_str(), None),
            };
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
                    parsed_args.io.set(IO::Input, arg, value, &mut args)?;
                }
                "-o" | "--output" => {
                    parsed_args.io.set(IO::Output, arg, value, &mut args)?;
                }

                // FLAGS
                "--fail-on-unset" => {
                    parsed_args.flags.set(Flag::FailOnUnset, true)?;
                }
                "--fail-on-empty" => {
                    parsed_args.flags.set(Flag::FailOnEmpty, true)?;
                }
                "--fail" => {
                    parsed_args.flags.set(Flag::Fail, true)?;
                }
                "--no-replace-unset" => {
                    parsed_args.flags.set(Flag::NoReplaceUnset, true)?;
                }
                "--no-replace-empty" => {
                    parsed_args.flags.set(Flag::NoReplaceEmpty, true)?;
                }
                "--no-replace" => {
                    parsed_args.flags.set(Flag::NoReplace, true)?;
                }
                "--no-escape" => {
                    parsed_args.flags.set(Flag::NoEscape, true)?;
                }
                "--unbuffer-lines" => {
                    parsed_args.flags.set(Flag::UnbufferedLines, true)?;
                }

                // FILTERS
                "-p" | "--prefix" => {
                    parsed_args
                        .filters
                        .add(Filter::Prefix, arg, value, &mut args)?;
                }
                "-s" | "--suffix" => {
                    parsed_args
                        .filters
                        .add(Filter::Suffix, arg, value, &mut args)?;
                }
                "-v" | "--variable" => {
                    parsed_args
                        .filters
                        .add(Filter::Variable, arg, value, &mut args)?;
                }
                // UNKNOWN
                _ => {
                    return Err(ParseArgsError::UnknownFlag(arg.to_string()));
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

        assert!(parsed_args.flags.get(Flag::NoReplaceEmpty).unwrap_or(false),);
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
    fn test_parse_unknown_flag2() {
        let args = vec!["--unknown-flag"];
        let parsed_args = Args::parse(args);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err(),
            ParseArgsError::UnknownFlag("--unknown-flag".to_owned())
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
        let parsed_args = Args::parse(args);

        assert!(parsed_args.is_ok());
        assert_eq!(
            parsed_args.unwrap().flags.get(Flag::UnbufferedLines),
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
        assert!(parsed_args.unwrap().flags.get(Flag::NoEscape).unwrap());
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
    fn test_input_space() {
        let args = vec!["--input", "input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some("input_file".to_string())
        );
    }

    #[test]
    fn test_input_equal() {
        let args = vec!["--input=input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some("input_file".to_string())
        );
    }

    #[test]
    fn test_input_equal_short() {
        let args = vec!["-i=input_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Input),
            Some("input_file".to_string())
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
            Some("output_file".to_string())
        );
    }

    #[test]
    fn test_output_equal() {
        let args = vec!["--output=output_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Output),
            Some("output_file".to_string())
        );
    }

    #[test]
    fn test_output_equal_short() {
        let args = vec!["-o=output_file"];
        let parsed_args = Args::parse(args).unwrap();
        assert_eq!(
            parsed_args.io.get(IO::Output),
            Some("output_file".to_string())
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
}
