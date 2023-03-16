use crate::errors::ParseArgsError;
use crate::filters::Filters;
use crate::flags::{Flag, Flags};
use crate::help::HELP_TEXT;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Args {
    pub version: Option<String>,
    pub help: Option<String>,
    pub flags: Flags,
    pub filters: Filters,
}

impl Args {
    fn new() -> Self {
        Args {
            version: None,
            help: None,
            flags: Flags::default(),
            filters: Filters::default(),
        }
    }

    /// Validates the value of a given parameter against a set of allowed values.
    ///
    /// # Arguments
    ///
    /// * `arg` - A string slice representing the name of the parameter being validated.
    /// * `arg_value` - An optional string slice representing the value of the parameter being validated.
    /// * `start_params` - A reference to a `HashSet` containing the allowed values for the parameter.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if the value of the parameter is not in the allowed set of values.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::args::{Args, ParseArgsError};
    ///
    /// let start_params = vec!["foo", "bar"].into_iter().collect();
    /// let result = Args::validate_param_value("--input", Some("baz"), &start_params);
    /// assert_eq!(result.unwrap_err(), ParseArgsError::MissingValue("--input".to_owned()));
    ///
    /// let result = Args::validate_param_value("--input", Some("foo"), &start_params);
    /// assert_eq!(result.unwrap(), "foo".to_owned());
    /// ```
    #[cfg(not(tarpaulin_include))]
    fn validate_param_value(
        arg: &str,
        arg_value: Option<&str>,
        start_params: &HashSet<&'static str>,
    ) -> Result<String, ParseArgsError> {
        let value = arg_value.ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;

        // check if the value is in the allowed set of values
        if start_params.contains(value) {
            return Err(ParseArgsError::MissingValue(arg.to_owned()));
        }
        return Ok(value.to_string());
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
    /// use renvsubst::cli::Args;
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

        // a set of all start parameters
        // this is used to check if the next argument is a start parameter
        // eg. if the current argument is "--prefix" and the next argument is "--fail-on-unset",
        // then "--fail-on-unset" is a start parameter and "--prefix" has missing a value
        let start_params: HashSet<&'static str> = [
            "-h",
            "--help",
            "--version",
            "--fail-on-unset",
            "--fail-on-empty",
            "--fail",
            "--no-replace-unset",
            "--no-replace-empty",
            "--no-escape",
            "--unbuffer-lines",
            "-p",
            "--prefix",
            "-s",
            "--suffix",
            "-v",
            "--variable",
        ]
        .iter()
        .copied()
        .collect();

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
                    // no check for already added prefixes needed, because the HashSet will ignore duplicates
                    // check if the value is provided with the flag (eg. "--prefix=PREFIX")
                    let prefix_arg = value
                        .map(|value| Ok(value.to_string())) // if the value is provided with the flag, use it
                        .unwrap_or_else(|| {
                            // if not, get the next argument as the value
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))
                                .map(|s| s.to_string()) // convert the value to a string
                        })?;
                    // check if the value is valid
                    Self::validate_param_value(arg, Some(&prefix_arg), &start_params)?;

                    // add to prefixes
                    parsed_args
                        .filters
                        .prefixes
                        .get_or_insert_with(HashSet::new)
                        .insert(prefix_arg);
                }
                "-s" | "--suffix" => {
                    // no check for already added suffixes needed, because the HashSet will ignore duplicates
                    // check if the value is provided with the flag (eg. "--suffix=SUFFIX")
                    let suffix_arg = value
                        .map(|value| Ok(value.to_string())) // if the value is provided with the flag, use it
                        .unwrap_or_else(|| {
                            // if not, get the next argument as the value
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))
                                .map(|s| s.to_string()) // convert the value to a string
                        })?;
                    // check if the value is valid
                    Self::validate_param_value(arg, Some(&suffix_arg), &start_params)?;

                    // add to suffixes
                    parsed_args
                        .filters
                        .suffixes
                        .get_or_insert_with(HashSet::new)
                        .insert(suffix_arg);
                }
                "-v" | "--variable" => {
                    // no check for already added variables needed, because the HashSet will ignore duplicates
                    // check if the value is provided with the flag (eg. "--variable=VARIABLE")
                    let variable_arg = value
                        .map(|value| Ok(value.to_string())) // if the value is provided with the flag, use it
                        .unwrap_or_else(|| {
                            // if not, get the next argument as the value
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))
                                .map(|s| s.to_string()) // convert the value to a string
                        })?;
                    // check if the value is valid
                    Self::validate_param_value(arg, Some(&variable_arg), &start_params)?;

                    // add to variables
                    parsed_args
                        .filters
                        .variables
                        .get_or_insert_with(HashSet::new)
                        .insert(variable_arg);
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
        let args = vec!["--prefix", "prefix-"];
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
        assert!(parsed_args.unwrap().flags.get(Flag::NoEscape).unwrap())
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
        assert_eq!(
            parsed_args
                .filters
                .prefixes
                .unwrap()
                .contains("prefix_value"),
            true
        );
    }
}
