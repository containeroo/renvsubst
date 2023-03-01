use std::{env, collections::HashSet};

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

/// Configuration flags that control the behavior of the variable substitution operation.
///
/// The `Flags` struct contains several boolean flags that control the behavior of the variable
/// substitution operation. These flags determine whether the program should fail if a variable
/// is unset or empty, whether the program should perform variable replacement if a variable is
/// unset or empty, and whether the program should escape special characters in the output.
///
/// # Fields
///
/// * `fail_on_unset`: If set to `true`, the program will fail if a variable is unset.
/// * `fail_on_empty`: If set to `true`, the program will fail if a variable is empty.
/// * `no_replace_unset`: If set to `true`, the program will not perform variable replacement
///                        if a variable is unset.
/// * `no_replace_empty`: If set to `true`, the program will not perform variable replacement
///                        if a variable is empty.
/// * `no_escape`: If set to `true`, the program will not escape special characters in the output.
///
/// # Examples
///
/// ```
/// // Create a new Flags instance with default values
/// let flags = Flags::default();
///
/// assert_eq!(flags.fail_on_unset, false);
/// assert_eq!(flags.fail_on_empty, false);
/// assert_eq!(flags.no_replace_unset, false);
/// assert_eq!(flags.no_replace_empty, false);
/// assert_eq!(flags.no_escape, false);
/// ```
#[derive(Default)]
pub struct Flags {
    pub fail_on_unset: bool,
    pub fail_on_empty: bool,
    pub no_replace_unset: bool,
    pub no_replace_empty: bool,
    pub no_escape: bool,
}

/// Configuration filters that control which variables will be replaced in the output.
///
/// The `Filters` struct contains several optional fields that control which variables will be
/// replaced in the output. These fields specify prefixes, suffixes, and variable names that the
/// program should search for in the input file and replace with their corresponding values.
///
/// # Fields
///
/// * `prefixes`: An optional vector of strings that specifies the variable prefixes to search for
///               in the input file. If set to `None`, the program will not search for variables
///               with a prefix.
/// * `suffixes`: An optional vector of strings that specifies the variable suffixes to search for
///               in the input file. If set to `None`, the program will not search for variables
///               with a suffix.
/// * `variables`: An optional vector of strings that specifies the exact variable names to search
///                for in the input file. If set to `None`, the program will not search for specific
///                variable names.
///
/// # Examples
///
/// ```
/// // Create a new Filters instance with default values
/// let filters = Filters::default();
///
/// assert_eq!(filters.prefixes, None);
/// assert_eq!(filters.suffixes, None);
/// assert_eq!(filters.variables, None);
/// ```
#[derive(Default)]
pub struct Filters {
    pub prefixes: Option<HashSet<String>>, // An optional vector of strings that specifies the variable prefixes to search for in the input file. If set to `None`, the program will not search for variables with a prefix.
    pub suffixes: Option<HashSet<String>>, // An optional vector of strings that specifies the variable suffixes to search for in the input file. If set to `None`, the program will not search for variables with a suffix.
    pub variables: Option<HashSet<String>>, // An optional vector of strings that specifies the exact variable names to search for in the input file. If set to `None`, the program will not search for specific variable names.
}

/// Command-line arguments that control the behavior of the variable substitution program.
///
/// The `Args` struct contains several fields that control the behavior of the variable substitution
/// program. These fields include options for displaying version and help information, specifying
/// input and output files, and configuring the behavior of the variable substitution operation.
///
/// # Fields
///
/// * `version`: An optional string that specifies the version number of the program. If set to
///              `Some(version)`, the program will display the version number and exit when the
///              `--version` flag is passed on the command line.
/// * `help`: An optional string that specifies the help text for the program. If set to `Some(help)`,
///           the program will display the help text and exit when the `--help` flag is passed on
///           the command line.
/// * `input_file`: An optional string that specifies the name of the input file. If set to `None`,
///                  the program will read from stdin. The `--input` flag can be used to specify a
///                  different input file.
/// * `output_file`: An optional string that specifies the name of the output file. If set to `None`,
///                   the program will write to stdout. The `--output` flag can be used to specify a
///                   different output file.
/// * `flags`: A `Flags` struct that controls the behavior of the variable substitution operation.
///
///     The `Flags` struct contains several boolean flags that control the behavior of the variable
///     substitution operation. These flags determine whether the program should fail if a variable
///     is unset or empty, whether the program should perform variable replacement if a variable is
///     unset or empty, and whether the program should escape special characters in the output.
///
/// * `filters`: A `Filters` struct that controls which variables will be replaced in the output.
///
///     The `Filters` struct contains several optional fields that control which variables will be
///     replaced in the output. These fields specify prefixes, suffixes, and variable names that the
///     program should search for in the input file and replace with their corresponding values.
///
/// # Examples
///
/// ```
/// // Create a new Args instance with default values
/// let args = Args::default();
///
/// assert_eq!(args.version, None);
/// assert_eq!(args.help, None);
/// assert_eq!(args.input_file, None);
/// assert_eq!(args.output_file, None);
/// assert_eq!(args.flags, Flags::default());
/// assert_eq!(args.filters, Filters::default());
/// ```
#[derive(Default)]
pub struct Args {
    pub version: bool,
    pub help: bool,
    pub input_file: Option<String>,
    pub output_file: Option<String>, // output file name, if provided
    pub flags: Flags,                // flags to control the behavior of the program
    pub filters: Filters,            // filters to control which variables will be replaced
}

/// Parses the command line arguments and returns an `Args` struct with the parsed values.
///
/// This function takes the command line arguments as input and parses them to create an instance
/// of the `Args` struct, which contains all of the program's configuration options. The struct's
/// fields are populated based on the command line arguments, and default values are used for any
/// fields that are not explicitly set.
///
/// If the `-h` flag is specified, the function returns early with a `show_help` field set to `true`.
/// If the `-v` flag is specified, the function returns early with a `show_version` field set to `true`.
///
/// # Arguments
///
/// None.
///
/// # Errors
///
/// Returns an error if the arguments are invalid, such as if an unknown flag is specified or
/// if a required argument is missing.
///
/// # Examples
///
/// ```
/// // Parse command line arguments and perform a substitution using the resulting configuration
/// let args = parse_args().unwrap_or_else(|e| {
///     eprintln!("{}", e);
///     std::process::exit(1);
/// });
///
///// print version and exit if requested
/// if args.version.is_some() {
///      println!("renvsubst {}", args.version.unwrap());
///     std::process::exit(0);
/// }
///
/// // print help and exit if requested
/// if args.help.is_some() {
///     println!("{}", args.help.unwrap());
///     std::process::exit(0);
/// }
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
    let mut variables: Option<HashSet<String>> = None;
    let mut suffixes: Option<HashSet<String>> = None;
    let mut prefixes: Option<HashSet<String>> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => {
                return Ok(Args {
                    help: true,
                    ..Default::default()
                })
            }
            "-v" => {
                return Ok(Args {
                    version: true,
                    ..Default::default()
                })
            }

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
                prefixes.get_or_insert_with(HashSet::new).insert(
                    args.next()
                        .ok_or_else(|| "ERROR: --prefix requires a prefix to be specified")?,
                );
            }
            "--suffix" => {
                suffixes.get_or_insert_with(HashSet::new).insert(
                    args.next()
                        .ok_or_else(|| "ERROR: --suffix requires a suffix to be specified")?,
                );
            }
            "--variable" => {
                variables.get_or_insert_with(HashSet::new).insert(
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
        help: false,
        version: false,
        input_file,
        output_file,
        flags,
        filters,
    });
}
