use std::{collections::HashSet, env};

/// An error that occurs while parsing command-line arguments.
#[derive(Debug)]
pub enum ParseArgsError {
    /// An unknown flag was specified.
    UnknownFlag(String),

    /// A value is missing for a given flag.
    MissingValue(String),

    /// A mandatory parameter is missing.
    MissingMandatoryParameter(String),

    /// Two or more conflicting flags were specified.
    ConflictingFlags(String),
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

    /// Parses command-line arguments and returns an `Args` struct with the parsed values.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if there is an error in parsing the arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate:args:Args;
    ///
    /// let args = Args::parse()?;
    /// println!("{:?}", args);
    /// ```
    pub fn parse() -> Result<Args, ParseArgsError> {
        let mut args = env::args().skip(1).peekable();
        let mut parsed_args = Self::new();

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
                    let arg_clone = arg.clone();
                    let input_arg = args
                        .peek()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg))?;
                    // check if the next argument is a start parameter
                    if start_params.contains(input_arg.as_str()) {
                        return Err(ParseArgsError::MissingValue(arg_clone));
                    }
                    parsed_args
                        .filters
                        .prefixes
                        .get_or_insert_with(HashSet::new)
                        .insert(input_arg.to_string());

                    args.next(); // skip the next argument since it is a valid prefix
                }
                "-o" | "--output" => {
                    let arg_clone = arg.clone();
                    let output_arg = args
                        .peek()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg))?;
                    // check if the next argument is a start parameter
                    if start_params.contains(output_arg.as_str()) {
                        return Err(ParseArgsError::MissingValue(arg_clone));
                    }
                    parsed_args
                        .filters
                        .suffixes
                        .get_or_insert_with(HashSet::new)
                        .insert(output_arg.to_string());

                    args.next(); // skip the next argument since it is a valid prefix
                }
                "--fail-on-unset" => {
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-unset'",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_unset = true;
                }
                "--fail-on-empty" => {
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--no-replace-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_empty = true;
                }
                "--fail" => {
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
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-unset'",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_unset = true;
                }
                "--no-replace-empty" => {
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "'{}' and '--fail-on-empty'",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_empty = true;
                }
                "--no-replace" => {
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
                    parsed_args.flags.no_escape = true;
                }
                "-p" | "--prefix" => {
                    let arg_clone = arg.clone();
                    let prefix_arg = args
                        .peek()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg))?;
                    // check if the next argument is a valid prefix
                    if start_params.contains(prefix_arg.as_str()) {
                        return Err(ParseArgsError::MissingValue(arg_clone));
                    }
                    parsed_args
                        .filters
                        .prefixes
                        .get_or_insert_with(HashSet::new)
                        .insert(prefix_arg.to_string());

                    args.next(); // skip the next argument since it is a valid prefix
                }
                "--suffix" => {
                    let arg_clone = arg.clone();
                    let suffix_arg = args
                        .peek()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg))?;
                    // check if the next argument is a valid suffix
                    if start_params.contains(suffix_arg.as_str()) {
                        return Err(ParseArgsError::MissingValue(arg_clone));
                    }
                    parsed_args
                        .filters
                        .suffixes
                        .get_or_insert_with(HashSet::new)
                        .insert(suffix_arg.to_string());

                    // skip the next argument since it is a valid suffix
                    args.next();
                }
                "--variable" => {
                    let arg_clone = arg.clone();
                    let variable_arg = args
                        .peek()
                        .ok_or_else(|| ParseArgsError::MissingValue(arg))?;
                    // check if the next argument is a valid variable
                    if start_params.contains(variable_arg.as_str()) {
                        return Err(ParseArgsError::MissingValue(arg_clone));
                    }
                    parsed_args
                        .filters
                        .variables
                        .get_or_insert_with(HashSet::new)
                        .insert(variable_arg.to_string());

                    args.next(); // skip the next argument since it is a valid prefix
                }
                _ => {
                    return Err(ParseArgsError::UnknownFlag(arg));
                }
            }
        }

        // input is the only required argument
        if parsed_args.input_file.is_none() {
            return Err(ParseArgsError::MissingMandatoryParameter(
                "'-i|--input'".to_string(),
            ));
        }

        return Ok(parsed_args);
    }
}

/// Help text for the renvsubst.
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
