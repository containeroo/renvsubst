use std::collections::HashSet;

/// An error that occurs while parsing command-line arguments.
#[derive(Debug, PartialEq)]
pub enum ParseArgsError {
    /// An unknown flag was specified.
    UnknownFlag(String),

    /// A value is missing for a given flag.
    MissingValue(String),

    /// A mandatory parameter is missing.
    MissingMandatoryParameter(String),

    /// Two or more conflicting flags were specified.
    ConflictingFlags(String),

    /// Duplicate values were specified for a given flag.
    DuplicateValue(String),
}

impl std::fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFlag(flag) => return write!(f, "Unknown flag: {}", flag),
            Self::MissingValue(flag) => return write!(f, "Flag '{}' requires a value!", flag),
            Self::ConflictingFlags(flags) => {
                return write!(f, "Flags {} cannot be used together!", flags)
            }
            Self::MissingMandatoryParameter(param) => {
                return write!(f, "Missing mandatory parameter: {}", param)
            }
            Self::DuplicateValue(flag) => {
                return write!(f, "Flag '{}' cannot be specified more than once!", flag)
            }
        }
    }
}

impl std::error::Error for ParseArgsError {}

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

#[derive(Debug, Default)]
pub struct Args {
    pub version: bool,
    pub help: bool,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub flags: Flags,
    pub filters: Filters,
}

impl Args {
    fn new() -> Self {
        Args {
            version: false,
            help: false,
            input_file: None,
            output_file: None,
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
    fn validate_param_value(
        arg: &str,
        arg_value: Option<&str>,
        start_params: &HashSet<&'static str>,
    ) -> Result<String, ParseArgsError> {
        let value = arg_value.ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;

        if start_params.contains(value) {
            return Err(ParseArgsError::MissingValue(arg.to_owned()));
        }
        return Ok(value.to_string());
    }
    /// Parses the given arguments and returns an `Args` struct.
    ///
    /// # Arguments
    ///
    /// * `args` - An iterator over the arguments to parse. Each argument should be a string slice (`&str`).
    ///   Note that if you pass `env::args()`, you should skip the first argument, which is the name of the
    ///   program.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if there is an error parsing the arguments.
    ///
    /// # Example
    ///
    /// ```
    /// use args::Args;
    ///
    /// let custom_args = ["-i", "input.txt", "-o", "output.txt"];
    /// let parsed_custom_args = Args::parse(custom_args).unwrap();
    /// assert_eq!(parsed_args.input_file, Some("input.txt".to_owned()));
    /// assert_eq!(parsed_args.output_file, Some("output.txt".to_owned()));
    ///
    /// let parsed_os_args = Args::parse(env::args().skip(1)).unwrap();
    /// // you have to set the input and output files in the environment
    /// assert_eq!(parsed_args.input_file, Some("input.txt".to_owned()));
    /// assert_eq!(parsed_args.output_file, Some("output.txt".to_owned()));
    ///
    /// ```
    pub fn parse<I, T>(args: I) -> Result<Args, ParseArgsError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut args = args
            .into_iter()
            .map(|arg| arg.into().into_string())
            .filter_map(Result::ok);

        let mut parsed_args = Self::new();

        // a set of all start parameters
        // this is used to check if the next argument is a start parameter
        // eg. if the current argument is "--input" and the next argument is "--fail-on-unset",
        // then "--fail-on-unset" is a start parameter and "--input" has missing a value
        let start_params: HashSet<&'static str> = [
            "-h",
            "--help",
            "-v",
            "--version",
            "-i",
            "--input",
            "-o",
            "--output",
            "--fail-on-unset",
            "--fail-on-empty",
            "--fail",
            "--no-replace-unset",
            "--no-replace-empty",
            "--no-escape",
            "--prefix",
            "--suffix",
            "--variable",
        ]
        .iter()
        .cloned()
        .collect();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    parsed_args.help = true;
                    return Ok(parsed_args);
                }
                "-v" | "--version" => {
                    parsed_args.version = true;
                    return Ok(parsed_args);
                }
                "-i" | "--input" => {
                    // check if already set
                    if parsed_args.input_file.is_some() {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check if next argument is a start parameter
                    let input_arg = args
                        .next()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?;

                    parsed_args.input_file = Some(Self::validate_param_value(
                        &arg,
                        Some(&input_arg),
                        &start_params,
                    )?);
                }
                "-o" | "--output" => {
                    // check if already set
                    if parsed_args.output_file.is_some() {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check if next argument is a start parameter
                    let output_arg = args
                        .next()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;
                    let parsed_output =
                        Self::validate_param_value(&arg, Some(&output_arg), &start_params)?;
                    parsed_args.output_file = Some(parsed_output);
                }
                // flags
                "--fail-on-unset" => {
                    // check if already set
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-unset'",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_unset = true;
                }
                "--fail-on-empty" => {
                    // check if already set
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_empty = true;
                }
                "--fail" => {
                    // check if already set
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-unset'",
                            arg
                        )));
                    }
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_unset = true;
                    parsed_args.flags.fail_on_empty = true;
                }
                "--no-replace-unset" => {
                    // check if already set
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-unset'",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_unset = true;
                }
                "--no-replace-empty" => {
                    // check if already set
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_empty = true;
                }
                "--no-replace" => {
                    // check if already set
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    // check for conflicting flags
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-unset'",
                            arg
                        )));
                    }
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_unset = true;
                    parsed_args.flags.no_replace_empty = true;
                }
                "--no-escape" => {
                    // check if already set
                    if parsed_args.flags.no_escape {
                        return Err(ParseArgsError::DuplicateValue(arg));
                    }
                    parsed_args.flags.no_escape = true;
                }
                // Filters
                "--prefix" => {
                    let prefix_arg = args
                        .next()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;
                    Self::validate_param_value(&arg, Some(prefix_arg.as_str()), &start_params)?;
                    parsed_args
                        .filters
                        .prefixes
                        .get_or_insert_with(HashSet::new)
                        .insert(prefix_arg.to_string());
                }
                "--suffix" => {
                    let suffix_arg = args
                        .next()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;
                    Self::validate_param_value(&arg, Some(suffix_arg.as_str()), &start_params)?;
                    parsed_args
                        .filters
                        .suffixes
                        .get_or_insert_with(HashSet::new)
                        .insert(suffix_arg.to_string());
                }
                "--variable" => {
                    let variable_arg = args
                        .next()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.to_owned()))?;
                    Self::validate_param_value(&arg, Some(variable_arg.as_str()), &start_params)?;
                    parsed_args
                        .filters
                        .variables
                        .get_or_insert_with(HashSet::new)
                        .insert(variable_arg.to_string());
                }

                _ => {
                    return Err(ParseArgsError::UnknownFlag(arg));
                }
            }
        }

        // input is the only required argument
        if parsed_args.input_file.is_none() {
            return Err(ParseArgsError::MissingMandatoryParameter(
                "-i|--input".to_string(),
            ));
        }

        return Ok(parsed_args);
    }
}

pub const HELP_TEXT: &str = "Usage: renvsubst [PARAMETERS] [FLAGS] [FILTERS]

renvsubst will substitute all (bash-like) environment variables in the format of $VAR_NAME, ${VAR_NAME} or ${VAR_NAME:-DEFAULT_VALUE} with their corresponding values from the environment or the default value if provided. If the variable is not valid, it remains as is.
A valid variable name starts with a letter or underscore, followed by any combination of letters, numbers, or underscores.

Parameters:
  -i|--input [INPUT_FILE]          Specify the input file. Use - to read from stdin.
                                   The input will be read line by line.
  -o|--output [OUTPUT_FILE]        Specify the output file. If not provided, the output will be written to stdout.

Flags:
  --fail-on-unset                  Fail if an environment variable is not set.
  --fail-on-empty                  Fail if an environment variable is empty.
  --fail                           Alias for --fail-on-unset and --fail-on-empty.
                                   Fails if an environment variable is either not set or empty.
  --no-replace-unset               Do not replace variables that are not set in the environment.
  --no-replace-empty               Do not replace variables that are set but empty in the environment.
  --no-replace                     Alias for --no-replace-unset and --no-replace-empty.
                                   Does not replace variables that are either not set or empty in the environment.
  --no-escape                      Disable escaping of variables with two dollar signs ($$).
  -h|--help                        Show this help text.
  -v|--version                     Show the version of the program.

Filters:

  --prefix [PREFIX]...             Only replace variables with the specified prefix.
                                   Prefixes can be specified multiple times.
  --suffix [SUFFIX]...             Only replace variables with the specified suffix.
                                   Suffixes can be specified multiple times.
  --variable [VARIABLE]...         Specify the variables to replace. If not provided, all variables will be replaced.
                                   Variables can be specified multiple times.

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.

Escaping:
To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($). The second dollar sign will be removed during substitution. Only valid variables must be escaped.

";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let args = vec![
            "-i",
            "input.txt",
            "--no-replace-empty",
            "--prefix",
            "prefix-",
            "-o",
            "output.txt",
        ];

        let parsed_args = Args::parse(args).unwrap();

        assert_eq!(parsed_args.input_file, Some("input.txt".to_owned()));
        assert_eq!(parsed_args.output_file, Some("output.txt".to_owned()));
        assert_eq!(parsed_args.flags.no_replace_empty, true);
        assert_eq!(
            parsed_args.filters.prefixes.unwrap().contains("prefix-"),
            true
        );
    }

    #[test]
    fn test_parse_no_input_file_value_long() {
        let args = vec!["-o", "output_file.txt", "--fail-on-unset", "--input"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--input".to_string())
        );
    }

    #[test]
    fn test_parse_no_input_file_value_short() {
        let args = vec!["-o", "output_file.txt", "--fail-on-unset", "-i"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("-i".to_string())
        );
    }

    #[test]
    fn test_parse_no_input_file_argument() {
        let args = vec!["-o", "output_file.txt", "--fail-on-unset"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingMandatoryParameter("-i|--input".to_string())
        );
    }

    #[test]
    fn test_parse_unknown_flag() {
        let args = vec![
            "--invalid-flag",
            "-i",
            "input_file.txt",
            "-o",
            "output_file.txt",
        ];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::UnknownFlag("--invalid-flag".to_string())
        );
    }

    #[test]
    fn test_parse_help_flag() {
        let args = vec!["-h"];
        let result = Args::parse(args).unwrap();
        assert_eq!(result.help, true);
    }

    #[test]
    fn test_parse_version_flag() {
        let args = vec!["-v"];
        let result = Args::parse(args).unwrap();
        assert_eq!(result.version, true);
    }

    #[test]
    fn test_parse_conflicting_flags() {
        let args = vec![
            "--fail-on-unset",
            "--no-replace-unset",
            "-i",
            "input_file.txt",
            "-o",
            "output_file.txt",
        ];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "'--no-replace-unset' and '--fail-on-unset'".to_string()
            )
        );
    }

    #[test]
    fn test_parse_conflicting_flags2() {
        let args = vec![
            "--no-replace-unset",
            "--fail-on-unset",
            "-i",
            "input_file.txt",
            "-o",
            "output_file.txt",
        ];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::ConflictingFlags(
                "'--fail-on-unset' and '--no-replace-unset'".to_string()
            )
        );
    }

    #[test]
    fn test_parse_missing_value_prefix() {
        let args = vec!["--prefix", "-i", "input_file.txt", "-o", "output_file.txt"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--prefix".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_suffix() {
        let args = vec!["--suffix", "-i", "input_file.txt", "-o", "output_file.txt"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--suffix".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_output_long() {
        let args = vec!["--output", "-i", "input_file.txt"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--output".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_output_short_begin() {
        let args = vec!["-o", "-i", "input_file.txt"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("-o".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_output_short_middle() {
        let args = vec!["-i", "input_file.txt", "-o"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("-o".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_output_short_end() {
        let args = vec!["-i", "input_file.txt", "-o"];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("-o".to_string())
        );
    }

    #[test]
    fn test_parse_missing_value_variable() {
        let args = vec![
            "--variable",
            "-i",
            "input_file.txt",
            "-o",
            "output_file.txt",
        ];
        let result = Args::parse(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
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
    fn test_filters_with_prefixes() {
        let mut filters = Filters::default();
        filters
            .prefixes
            .get_or_insert_with(HashSet::new)
            .insert("prefix1".to_string());
        filters
            .prefixes
            .get_or_insert_with(HashSet::new)
            .insert("prefix2".to_string());
        assert!(filters.suffixes.is_none());
        assert!(filters.variables.is_none());
        assert_eq!(filters.prefixes.unwrap().len(), 2);
    }

    #[test]
    fn test_filters_with_suffixes() {
        let mut filters = Filters::default();
        filters
            .suffixes
            .get_or_insert_with(HashSet::new)
            .insert("suffix1".to_string());
        filters
            .suffixes
            .get_or_insert_with(HashSet::new)
            .insert("suffix2".to_string());
        assert!(filters.prefixes.is_none());
        assert!(filters.variables.is_none());
        assert_eq!(filters.suffixes.unwrap().len(), 2);
    }

    #[test]
    fn test_filters_with_variables() {
        let mut filters = Filters::default();
        filters
            .variables
            .get_or_insert_with(HashSet::new)
            .insert("variable1".to_string());
        filters
            .variables
            .get_or_insert_with(HashSet::new)
            .insert("variable2".to_string());
        assert!(filters.prefixes.is_none());
        assert!(filters.suffixes.is_none());
        assert_eq!(filters.variables.unwrap().len(), 2);
    }
}
