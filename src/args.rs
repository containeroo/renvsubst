use std::{collections::HashSet, env};

#[derive(Debug)]
pub enum ParseArgsError {
    UnknownFlag(String),
    MissingValue(String),
    ConflictingFlags(String),
}

impl std::fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFlag(flag) => write!(f, "Unknown flag: {}", flag),
            Self::MissingValue(flag) => write!(f, "Flag {} requires a value", flag),
            Self::ConflictingFlags(flags) => write!(f, "Flags {} cannot be used together", flags),
        }
    }
}

impl std::error::Error for ParseArgsError {}

#[derive(Debug, Default)]
pub struct Flags {
    #[doc = "If set to `true`, the program will fail if a variable is unset."]
    pub fail_on_unset: bool,
    #[doc = "If set to `true`, the program will fail if a variable is empty."]
    pub fail_on_empty: bool,
    #[doc = "If set to `true`, the program will not perform variable replacement if a variable is unset."]
    pub no_replace_unset: bool,
    #[doc = "If set to `true`, the program will not perform variable replacement if a variable is empty."]
    pub no_replace_empty: bool,
    #[doc = "If set to `true`, the program will not escape special characters in the output."]
    pub no_escape: bool,
}

#[derive(Debug, Default)]
pub struct Filters {
    #[doc = "An optional vector of strings that specifies the variable prefixes to search for in the input file. If set to `None`, the program will not search for variables with a prefix."]
    pub prefixes: Option<HashSet<String>>,
    #[doc = "An optional vector of strings that specifies the variable suffixes to search for in the input file. If set to `None`, the program will not search for variables with a suffix."]
    pub suffixes: Option<HashSet<String>>,
    #[doc = "An optional vector of strings that specifies the exact variable names to search for in the input file. If set to `None`, the program will not search for specific variable names."]
    pub variables: Option<HashSet<String>>,
}

#[derive(Debug, Default)]
pub struct Args {
    #[doc = "An optional string that specifies the version number of the program. If set to `Some(version)`, the program will display the version number and exit when the `--version` flag is passed on the command line."]
    pub version: bool,
    #[doc = "An optional string that specifies the help text for the program. If set to `Some(help)`, the program will display the help text and exit when the `--help` flag is passed on the command line."]
    pub help: bool,
    #[doc = "An optional string that specifies the name of the input file. If set to `None`, the program will read from stdin. The `--input` flag can be used to specify a different input file."]
    pub input_file: Option<String>,
    #[doc = "An optional string that specifies the name of the output file. If set to `None`, the program will write to stdout. The `--output` flag can be used to specify a different output file."]
    pub output_file: Option<String>,
    #[doc = "A `Flags` struct that controls the behavior of the variable substitution operation."]
    pub flags: Flags,
    #[doc = "A `Filters` struct that controls which variables will be replaced in the output."]
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

    pub fn parse_args() -> Result<Args, ParseArgsError> {
        let mut args = env::args().skip(1);
        let mut parsed_args = Self::new();

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
                    parsed_args.input_file = Some(
                        args.next()
                            .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?,
                    );
                }
                "-o" | "--output" => {
                    parsed_args.output_file = Some(
                        args.next()
                            .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?,
                    );
                }
                "--fail-on-unset" => {
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --no-replace-unset",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_unset = true;
                }
                "--fail-on-empty" => {
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --no-replace-empty",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_empty = true;
                }
                "--fail" => {
                    if parsed_args.flags.no_replace_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --no-replace-unset",
                            arg
                        )));
                    }
                    if parsed_args.flags.no_replace_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --no-replace-empty",
                            arg
                        )));
                    }
                    parsed_args.flags.fail_on_unset = true;
                    parsed_args.flags.fail_on_empty = true;
                }
                "--no-replace-unset" => {
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --fail-on-unset",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_unset = true;
                }
                "--no-replace-empty" => {
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --fail-on-empty",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_empty = true;
                }
                "--no-replace" => {
                    if parsed_args.flags.fail_on_unset {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --fail-on-unset",
                            arg
                        )));
                    }
                    if parsed_args.flags.fail_on_empty {
                        return Err(ParseArgsError::ConflictingFlags(format!(
                            "{} and --fail-on-empty",
                            arg
                        )));
                    }
                    parsed_args.flags.no_replace_unset = true;
                    parsed_args.flags.no_replace_empty = true;
                }
                "--no-escape" => {
                    parsed_args.flags.no_escape = true;
                }
                "--prefix" => {
                    parsed_args
                        .filters
                        .prefixes
                        .get_or_insert_with(HashSet::new)
                        .insert(
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?,
                        );
                }
                "--suffix" => {
                    parsed_args
                        .filters
                        .suffixes
                        .get_or_insert_with(HashSet::new)
                        .insert(
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?,
                        );
                }
                "--variable" => {
                    parsed_args
                        .filters
                        .variables
                        .get_or_insert_with(HashSet::new)
                        .insert(
                            args.next()
                                .ok_or_else(|| ParseArgsError::MissingValue(arg.clone()))?,
                        );
                }
                _ => {
                    return Err(ParseArgsError::UnknownFlag(arg));
                }
            }
        }

        return Ok(parsed_args);
    }

}


/// Template for the help text.
pub const HELP_TEXT: &str = "Usage: renvsubst [PARAMETERS] [FLAGS] [FILTERS]

renvsubst will substitute all (bash-like) environment variables in the format of $VAR_NAME, ${VAR_NAME} or ${VAR_NAME:-DEFAULT_VALUE} with their corresponding values from the environment or the default value if provided. If the variable is not valid, it remains as is.
A valid variable name starts with a letter or underscore, followed by any combination of letters, numbers, or underscores.

Parameters:
  -i [INPUT_FILE]                  Specify the input file. Use - to read from stdin.
                                   The input will be read line by line.
  -o [OUTPUT_FILE]                 Specify the output file. If not provided, the output will be written to stdout.

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
  -h                               Show this help text.
  -v                               Show the version of the program.

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