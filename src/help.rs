pub const HELP_TEXT: &str = "Usage: renvsubst [FLAGS] [FILTERS] [INPUT] [OUTPUT] | [-h | --help | --version]

renvsubst is a command-line tool that substitutes variables in the format '$VAR_NAME' or '${VAR_NAME}' with their corresponding environment values. If a variable is invalid, it remains unaltered.
Valid variable names start with a letter or underscore and can be followed by any combination of letters, numbers, or underscores.

\"Braced variables\" ('${VAR_NAME}') support some bash string substitution functions, see below.

Short flags are available for many options and can be combined. For example, use '-ue' instead of '-u -e' or '--fail-on-unset --fail-on-empty'. See the list of flags and filters below for the complete list of short flags and their combinations.

Flags:
  -u, --fail-on-unset              Fails if an environment variable is not set.
  -e, --fail-on-empty              Fails if an environment variable is empty.
  -f, --fail                       Alias for --fail-on-unset and --fail-on-empty.
  -U, --no-replace-unset           Does not replace variables that are not set in the environment.
  -E, --no-replace-empty           Does not replace variables that are set but empty in the environment.
  -N, --no-replace                 Alias for --no-replace-unset and --no-replace-empty.
  -x, --no-escape                  Disables escaping of variables with two dollar signs ($$).
  -b, --unbuffer-lines             Do not buffer lines before printing.
                                   Saves memory, but may impact performance.
  -c, --color                      Colorize the output if stdout is a terminal.
                                   Green for found variables, yellow for default values,
                                   and red for not found variables. Use '--no-replace-unset'
                                   to show not found variables; otherwise, they won't be displayed.

When the same flag is provided multiple times, renvsubst will throw an error.

Filters:
  -p, --prefix[=PREFIX]...         Only replace variables with the specified prefix.
                                   Prefixes can be specified multiple times.
  -s, --suffix[=SUFFIX]...         Only replace variables with the specified suffix.
                                   Suffixes can be specified multiple times.
  -v, --variable[=VARIABLE]...     Specify the variables to replace. If not provided,
                                   all variables will be replaced.
                                   Variables can be specified multiple times.

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.
If multiple identical prefixes, suffixes, or variables are provided, only one copy of each will be used.

Input:
  -i, --input[=FILE]               Input file path. Use '-' to read from stdin.
                                   Defaults to stdin if omitted.

Output:
  -o, --output[=FILE]              Output file path. Use '-' to write to stdout.
                                   Defaults to stdout if omitted.

General:
  -h, --help                       Show this help text.
      --version                    Show the version of the program.

Substitution functions:
  ${VAR:-default}                  Set '$VAR' to 'default' if '$VAR' is unset.
  ${VAR,}                          Change the first character of '$VAR' to lowercase.
  ${VAR,,}                         Change all characters of '$VAR' to lowercase.
  ${VAR^}                          Change the first character of '$VAR' to uppercase.
  ${VAR^^}                         Change all characters of '$VAR' to uppercase.
  ${VAR/pattern/replacement}       Replace first 'pattern' with 'replacement' in VAR.
  ${VAR//pattern/replacement}      Replace all 'pattern' with 'replacement' in VAR.
  ${VAR/#pattern/replacement}      Replace 'pattern' with 'replacement' if VAR starts with it.
  ${VAR/%pattern/replacement}      Replace 'pattern' with 'replacement' if VAR ends with it.
  ${VAR:offset}                    Shift '$VAR' by 'n' characters from the start.
  ${VAR:offset:length}             Shift '$VAR' by 'n' characters with a maximum length of 'len'.
  ${VAR#pattern}                   Remove the shortest match of 'pattern' from the start of '$VAR'.
  ${VAR%pattern}                   Remove the shortest match of 'pattern' from the end of '$VAR'.

Colors:
  Green represents variables that were successfully substituted.
  Yellow denotes the use of default values.
  Blue indicates variables where a string substitution took place.
  Magenta indicates \"ignored\" variables, which had no filter applied.
  Red represents variables that could not be substituted.

Escaping:
To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($). The second dollar sign will be removed during substitution. Only valid variables must be escaped.

";
