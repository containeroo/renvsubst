pub const HELP_TEXT: &str = "Usage: renvsubst [FLAGS] [FILTERS] [INPUT] | -h | --help | --version

renvsubst will substitute all (bash-like) environment variables in the format of $VAR_NAME, ${VAR_NAME} or ${VAR_NAME:-DEFAULT_VALUE} with their corresponding values from the environment or the default value if provided. If the variable is not valid, it remains as is.
A valid variable name starts with a letter or underscore, followed by any combination of letters, numbers, or underscores.

General:
  -h, --help                       Show this help text.
      --version                    Show the version of the program.

Flags:
  --fail-on-unset                  Fails if an environment variable is not set.
  --fail-on-empty                  Fails if an environment variable is empty.
  --fail                           Alias for --fail-on-unset and --fail-on-empty.
  --no-replace-unset               Does not replace variables that are not set in the environment.
  --no-replace-empty               Does not replace variables that are set but empty in the environment.
  --no-replace                     Alias for --no-replace-unset and --no-replace-empty.
  --no-escape                      Disables escaping of variables with two dollar signs ($$).

When the same flag is provided multiple times, renvsubst will throw an error.

Filters:

  -p, --prefix[=PREFIX]...         Only replace variables with the specified prefix.
                                   Prefixes can be specified multiple times.
  -s, --suffix[=SUFFIX]...         Only replace variables with the specified suffix.
                                   Suffixes can be specified multiple times.
  -v, --variable[=VARIABLE]...     Specify the variables to replace. If not provided, all variables will be replaced.
                                   Variables can be specified multiple times.

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.
If multiple identical prefixes, suffixes or variables are provided, only one copy of each will be used.

Input:
The input can be passed via stdin. If no input is provided, the program will wait for input from the user.

Escaping:
To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($). The second dollar sign will be removed during substitution. Only valid variables must be escaped.

";
