use crate::VERSION;
use std::env;

/// Template for the help text.
const HELP_TEXT: &str = "Usage: renvsubst [PARAMETERS] [FLAGS] [FILTERS]

renvsubst will substitute all (bash-like) environment variables in the format of $VAR_NAME, ${VAR_NAME} or ${VAR_NAME:-DEFAULT_VALUE} with their corresponding values from the environment or the default value if provided. If the variable is not valid, it remains as is.
A valid variable name starts with a letter or underscore, followed by any combination of letters, numbers, or underscores.

Parameters:
  -i [INPUT_FILE]                  Specify the input file. Use - to read from stdin.
                                   The input will be read line by line.
  -o [OUTPUT_FILE]                 Specify the output file. If not provided, the output will be written to stdout.

Flags:
  --fail-on-unset                  Fail if an environment variable is not set.
  --fail-on-empty                  Fail if an environment variable is empty.
  --strict                         Alias for --fail-on-unset and --fail-on-empty.
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

/// The `Flags` struct represents a set of command-line flags that modify the behavior of
/// `envsubst`. These flags control how the program handles unset and empty variables and
/// whether it performs variable substitution.
///
/// # Fields
///
/// * `fail_on_unset`: if true, `envsubst` fails if a variable is not defined in the environment.
/// * `fail_on_empty`: if true, `envsubst` fails if a variable is defined but its value is empty.
/// * `no_replace_unset`: if true, `envsubst` does not replace variables that are not defined
///   in the environment with an empty string.
/// * `no_replace_empty`: if true, `envsubst` does not replace variables that have an empty value
///   with an empty string.
/// * `no_escape`: if true, `envsubst` does not escape variables with two dollar signs ($$).
pub struct Flags {
    pub fail_on_unset: bool,
    pub fail_on_empty: bool,
    pub no_replace_unset: bool,
    pub no_replace_empty: bool,
    pub no_escape: bool,
}

impl Default for Flags {
    /// Returns a new instance of `Flags` with all fields set to their default values.
    ///
    /// * `fail_on_unset`: `false`
    /// * `fail_on_empty`: `false`
    /// * `no_replace_unset`: `false`
    /// * `no_replace_empty`: `false`
    /// * `no_escape`: `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::Flags;
    ///
    /// let flags = Flags::default();
    /// assert_eq!(flags.fail_on_unset, false);
    /// ```
    fn default() -> Self {
        Self {
            fail_on_unset: false,
            fail_on_empty: false,
            no_replace_unset: false,
            no_replace_empty: false,
            no_escape: false,
        }
    }
}

/// Represents a set of filters to apply to environment variables during substitution.
///
/// This struct includes an optional `prefix` and `suffix` to restrict which variables will be substituted. If a
/// `prefix` or `suffix` is specified, only environment variables whose names start with the `prefix` or end with
/// the `suffix` will be substituted. If a `prefix` and a `suffix` are both specified, only environment variables
/// whose names start with the `prefix` and end with the `suffix` will be substituted.
///
/// Additionally, this struct includes an optional list of `variables` to substitute. If the `variables` field is
/// present, only the specified environment variables will be substituted. If the `variables` field is `None`,
/// all environment variables will be substituted.
pub struct Filters {
    pub prefixes: Option<Vec<String>>,
    pub suffixes: Option<Vec<String>>,
    pub variables: Option<Vec<String>>,
}

impl Default for Filters {
    /// Returns a new instance of `Filters` with all fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::Filters;
    ///
    /// let filters = Filters::default();
    /// assert_eq!(filters.prefix, None);
    /// ```

    fn default() -> Self {
        Self {
            prefixes: None,
            suffixes: None,
            variables: None,
        }
    }
}

/// Represents the arguments passed to the program.
///
/// `input_file` is the name of the input file, if provided. If not, the program will read from stdin.
///
/// `output_file` is the name of the output file, if provided. If not, the program will write to stdout.
///
/// `flags` controls the behavior of the program. The following flags are supported:
///
/// * `fail_on_unset`: If set to `true`, the program will exit with an error if a variable is referenced that has not been set.
///
/// * `fail_on_empty`: If set to `true`, the program will exit with an error if a variable is set to an empty string.
///
/// * `no_replace_unset`: If set to `true`, the program will not replace variables that have not been set with their default values.
///
/// * `no_replace_empty`: If set to `true`, the program will not replace variables that are set to an empty string with their default values.
///
/// * `no_escape`: If set to `true`, the program will not treat "$$" as an escape sequence.
///
/// `filters` controls which variables will be replaced. The following filters are supported:
///
/// * `prefix`: Only variables with this prefix will be replaced.
///
/// * `suffix`: Only variables with this suffix will be replaced.
///
/// * `variables`: Only the variables specified in this list will be replaced.
pub struct Args {
    pub input_file: Option<String>,
    pub output_file: Option<String>, // output file name, if provided
    pub flags: Flags,                // flags to control the behavior of the program
    pub filters: Filters,            // filters to control which variables will be replaced
}

/// Parses the command-line arguments and returns a struct representing the arguments.
///
/// # Arguments
///
/// None.
///
/// # Returns
///
/// A `Result` with the parsed arguments as a struct on success, or an error message on failure.
///
/// # Examples
///
/// ```rust
/// use crate::parse_args;
///
/// let args = parse_args().unwrap();
/// ```
pub fn parse_args() -> Result<Args, String> {
    let mut args = env::args().peekable();

    // check if arguments was passed
    if args.len() == 1 {
        return Err(HELP_TEXT.to_string());
    }

    args.next(); // skip program name

    let mut input_file = None;
    let mut output_file = None;
    let mut fail_on_empty: bool = false;
    let mut fail_on_unset = false;
    let mut strict: bool = false; // intermediate variable. If set, fail_on_unset and fail_on_empty will be set to true
    let mut no_replace_unset: bool = false;
    let mut no_replace_empty: bool = false;
    let mut no_replace: bool = false; // intermediate variable. If set, no_replace_unset and no_replace_empty will be set to true
    let mut no_escape: bool = false;
    let mut variables: Option<Vec<String>> = None;
    let mut suffixes: Option<Vec<String>> = None;
    let mut prefixes: Option<Vec<String>> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => return Err(HELP_TEXT.to_string()),
            "-v" => return Err(format!("version {}", VERSION)),

            "-o" => {
                output_file = Some(args.next().unwrap_or_else(|| {
                    "ERROR: -o requires an output file to be specified or - to write to stdout"
                        .to_string()
                }));
            }
            "-i" => {
                input_file = Some(args.next().unwrap_or_else(|| {
                    "ERROR: -i requires an input file to be specified or - to read from stdin"
                        .to_string()
                }));
            }

            // flags
            "--fail-on-unset" => fail_on_unset = true,
            "--fail-on-empty" => fail_on_empty = true,
            "--strict" => strict = true, // alias for --fail-on-unset and --fail-on-empty
            "--no-replace-unset" => no_replace_unset = true,
            "--no-replace-empty" => no_replace_empty = true,
            "--no-replace" => no_replace = true, // alias for --no-replace-unset and --no-replace-empty
            "--no-escape" => no_escape = true,

            // filters
            "--prefix" => {
                prefixes.get_or_insert_with(Vec::new).push(
                    args.next()
                        .ok_or_else(|| "ERROR: --prefix requires a prefix to be specified")?,
                );
            }
            "--suffix" => {
                suffixes.get_or_insert_with(Vec::new).push(
                    args.next()
                        .ok_or_else(|| "ERROR: --suffix requires a suffix to be specified")?,
                );
            }
            "--variable" => {
                variables.get_or_insert_with(Vec::new).push(
                    args.next()
                        .ok_or_else(|| "ERROR: --variable requires a variable to be specified")?,
                );
            }
            // unknown argument
            _ => {
                return Err(format!("ERROR: Unknown flag: {}", arg));
            }
        }
    }

    // fail if --fail-on-unset and --no-replace-unset are used together
    if fail_on_unset && no_replace_unset {
        return Err("ERROR: --fail-on-unset cannot be used with --no-replace-unset".to_string());
    }

    // fail if --fail-on-empty and --no-replace-empty are used together
    if fail_on_empty && no_replace_empty {
        return Err("ERROR: --fail-on-empty cannot be used with --no-replace-empty".to_string());
    }

    // --strict implies --fail-on-unset and --fail-on-empty
    if strict && (fail_on_unset || fail_on_empty) {
        return Err(
            "ERROR: --strict cannot be used with --fail-on-unset or --fail-on-empty".to_string(),
        );
    }

    // --strict implies --fail-on-unset and --fail-on-empty
    if strict {
        fail_on_unset = true;
        fail_on_empty = true;
    }

    // --no-replace implies --no-replace-unset and --no-replace-empty
    if no_replace && (no_replace_unset || no_replace_empty) {
        return Err(
            "ERROR: --no-replace cannot be used with --fail-on-unset or --fail-on-empty"
                .to_string(),
        );
    }

    // set no_replace_unset and no_replace_empty to true if no_replace is used
    if no_replace {
        no_replace_unset = true;
        no_replace_empty = true;
    }

    let flags = Flags {
        fail_on_unset,
        fail_on_empty,
        no_replace_unset,
        no_replace_empty,
        no_escape,
    };

    let filters = Filters {
        prefixes,
        suffixes,
        variables,
    };

    // Return the parsed arguments as a struct
    return Ok(Args {
        input_file,
        output_file,
        flags,
        filters,
    });
}
