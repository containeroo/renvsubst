use crate::VERSION;
use std::env;

const HELP_TEXT: &str = "Usage: renvsubst [PARAMETERS] [FLAGS] [FILTERS]

{MESSAGE}

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
  --prefix [PREFIX]                Only replace variables with the specified prefix.
  --suffix [SUFFIX]                Only replace variables with the specified suffix.
  --variable [VARIABLE]...         Specify the variables to replace. If not provided, all variables will be replaced.
                                   Variables can be specified multiple times.

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.

Escaping:
To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($). The second dollar sign will be removed during substitution. Only valid variables must be escaped.

";

const DEFAULT_TEXT: &str = "renvsubst will substitute all (bash-like) environment variables in the format of $VAR_NAME, ${VAR_NAME} or ${VAR_NAME:-DEFAULT_VALUE} with their corresponding values from the environment or the default value if provided. If the variable is not valid, it remains as is.\nA valid variable name starts with a letter or underscore, followed by any combination of letters, numbers, or underscores.";

// struct to store flags
pub struct Flags {
    pub fail_on_unset: bool,
    pub fail_on_empty: bool,
    pub no_replace_unset: bool,
    pub no_replace_empty: bool,
    pub no_escape: bool,
}

// struct to store optional filters
pub struct Filters {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub variables: Option<Vec<String>>,
}

// struct to store command line arguments
pub struct Args {
    pub input_file: Option<String>,
    pub output_file: Option<String>, // output file name, if provided
    pub flags: Flags,                // flags to control the behavior of the program
    pub filters: Filters,            // filters to control which variables will be replaced
}

// function to parse command line arguments
pub fn get_args() -> Args {
    let mut args = env::args();

    // check if arguments was passed
    if args.len() == 1 {
        println!("{}", HELP_TEXT.replace("{MESSAGE}", DEFAULT_TEXT));
        std::process::exit(1);
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
    let mut suffix: Option<String> = None;
    let mut prefix: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => {
                println!("{}", HELP_TEXT.replace("{MESSAGE}", DEFAULT_TEXT));
                std::process::exit(0);
            }
            "-v" => {
                println!("version {}", VERSION);
                std::process::exit(0);
            }
            "--fail-on-unset" => fail_on_unset = true,
            "--fail-on-empty" => fail_on_empty = true,
            "--strict" => strict = true, // alias for --fail-on-unset and --fail-on-empty
            "--no-replace-unset" => no_replace_unset = true,
            "--no-replace-empty" => no_replace_empty = true,
            "--no-replace" => no_replace = true, // alias for --no-replace-unset and --no-replace-empty
            "--no-escape" => no_escape = true,
            "-o" => {
                output_file = Some(args.next().unwrap_or_else(|| {
                    eprintln!(
                        "{}",
                        HELP_TEXT.replace(
                            "{MESSAGE}",
                            "ERROR: -o requires an output file to be specified"
                        )
                    );
                    std::process::exit(1);
                }))
            }
            "-i" => {
                input_file = Some(args.next().unwrap_or_else(|| {
                    eprintln!(
                    "{}",
                    HELP_TEXT.replace(
                        "{MESSAGE}",
                        "ERROR: -i requires an input file to be specified or - to read from stdin"
                    )
                );
                    std::process::exit(1);
                }))
            }
            "--prefix" => {
                prefix = Some(args.next().unwrap_or_else(|| {
                    eprintln!(
                        "{}",
                        HELP_TEXT.replace(
                            "{MESSAGE}",
                            "ERROR: --prefix requires a prefix to be specified"
                        )
                    );
                    std::process::exit(1);
                }))
            }
            "--suffix" => {
                suffix = Some(args.next().unwrap_or_else(|| {
                    eprintln!(
                        "{}",
                        HELP_TEXT.replace(
                            "{MESSAGE}",
                            "ERROR: --suffix requires a suffix to be specified"
                        )
                    );
                    std::process::exit(1);
                }))
            }
            "--variable" => {
                // push variable to vector
                variables
                    .get_or_insert_with(Vec::new)
                    .push(args.next().unwrap_or_else(|| {
                        eprintln!(
                            "{}",
                            HELP_TEXT.replace(
                                "{MESSAGE}",
                                "ERROR: --variable requires a variable to be specified"
                            )
                        );
                        std::process::exit(1);
                    }))
            }
            _ => {
                // If the argument is not known, write an error message and exit
                eprintln!(
                    "{}",
                    HELP_TEXT.replace("{MESSAGE}", &format!("ERROR: Unknown flag: {}", arg))
                );
                std::process::exit(1);
            }
        }
    }

    // fail if --fail-on-unset and --no-replace-unset are used together
    if fail_on_unset && no_replace_unset {
        eprintln!(
            "{}",
            HELP_TEXT.replace(
                "{MESSAGE}",
                "ERROR: --fail-on-unset cannot be used with --no-replace-unset"
            )
        );
        std::process::exit(1);
    }

    // fail if --fail-on-empty and --no-replace-empty are used together
    if fail_on_empty && no_replace_empty {
        eprintln!(
            "{}",
            HELP_TEXT.replace(
                "{MESSAGE}",
                "ERROR: --fail-on-empty cannot be used with --no-replace-empty"
            )
        );
        std::process::exit(1);
    }

    // --strict implies --fail-on-unset and --fail-on-empty
    if strict && !(fail_on_unset || fail_on_empty) {
        eprintln!(
            "{}",
            HELP_TEXT.replace(
                "{MESSAGE}",
                "ERROR: --strict cannot be used without --fail-on-unset or --fail-on-empty"
            )
        );
        std::process::exit(1);
    }

    // --strict implies --fail-on-unset and --fail-on-empty
    if strict {
        fail_on_unset = true;
        fail_on_empty = true;
    }

    // --no-replace implies --no-replace-unset and --no-replace-empty
    if no_replace && (no_replace_unset || no_replace_empty) {
        eprintln!(
            "{}",
            HELP_TEXT.replace(
                "{MESSAGE}",
                "ERROR: --no-replace cannot be used with --fail-on-unset or --fail-on-empty"
            )
        );
        std::process::exit(1);
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
        prefix,
        suffix,
        variables,
    };

    // Return the parsed arguments as a struct
    Args {
        input_file,
        output_file,
        flags,
        filters,
    }
}
