# renvsubst

A command line utility to substitute (bash-like) variables in the format of `$VAR_NAME`, `${VAR_NAME}` or `${VAR_NAME:-DEFAULT_VALUE}` with their corresponding values from the environment or the default value if provided.
A valid variable name is a string that starts with a letter or an underscore, followed by any number of letters, numbers, or underscores.

## Usage

```sh
Usage: renvsubst [PARAMETERS] [FLAGS] [FILTERS]
```

## Parameters

| Parameter            | Description                                                                       |
| :------------------- | :-------------------------------------------------------------------------------- |
| `-i` `[INPUT_FILE]`  | Specify the input file. Use `-` to read from `stdin`.                             |
| `-o` `[OUTPUT_FILE]` | Specify the output file. If not provided, the output will be written to `stdout`. |

## Flags

| Parameter            | Description                                             |
| -------------------- | ------------------------------------------------------- |
| `--fail-on-unset`    | Fail if an environment variable is not set.             |
| `--fail-on-empty`    | Fail if an environment variable is empty.               |
| `--strict`           | Alias for `--fail-on-unset` and `--fail-on-empty`.      |
| `--no-replace-unset` | Do not replace variables that are not set.              |
| `--no-replace-empty` | Do not replace variables that are empty.                |
| `--no-replace`       | Alias for`--no-replace-unset` and `--no-replace-empty`. |
| `--no-escape`        | Disable escaping of variables.                          |
| `-h`                 | Show help text.                                         |
| `-v`                 | Show the version of the program.                        |

## Filters

| Parameter                    | Description                                                                                                              |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `--prefix`                   | Only replace variables with the specified prefix.                                                                        |
| `--suffix`                   | Only replace variables with the specified suffix.                                                                        |
| `--variable` [VARIABLE_NAME] | Specify variable to replace. If not provided, all variables will be replaced. Variables can be specified multiple times. |

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable will be replaced, while all others will remain unchanged.

## Escaping

To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($).
For example, to escape VAR_NAME, use `$$VAR_NAME`, `$${VAR_NAME}`, or `$${VAR_NAME:-DEFAULT_VALUE}`. The second dollar sign will be removed, resulting in `$VAR_NAME`, `${VAR_NAME}`, or `${VAR_NAME:-DEFAULT_VALUE}`.

To turn off escaping entirely, use the `--no-escape` flag. Escaping is evaluated after the evaluation of the variable, so you only need to escape text like this:

`$${VAR_NAME}` will be replaced with `${VAR_NAME}`.
`$$VAR_NAME` will be replaced with `$VAR_NAME`.
`$${VAR_NAME:-DEFAULT_VALUE}` will be replaced with `${VAR_NAME:-DEFAULT_VALUE}`.

`I have a pa$$word` will be replaced with `I have a pa$word`. To escape this text, you have multiple options:

Escape the whole variable: `I have a pa$$$$word`.
Use the `--no-escape` flag.
Use the `--no-replace-empty` flag. If there is no environment variable named `word`, the variable will not be replaced.

## Examples

### Preparation

Create a test file:

```sh
cat << EOF > test.txt
This is a "\$FILE_NAME" file.
It has more than "\${AMOUNT}" different variables.
You can also use "\${UNSET_VARIABLE:-default}" values inside variables like "\$\${UNSET_VARIABLE:-default}".
Here are more variable like "\${PREFIXED_VARIABLE_1}" and "\${VARIABLE_1_SUFFIXED}".
Here are more "\$PREFIXED_VARIABLE_2" and "\$VARIABLE_2_SUFFIXED" variables!
EOF
```

Set variables:

```sh
export FILE_NAME=test.txt
export AMOUNT=1
export PREFIXED_VARIABLE_1="variable with a prefix"
export PREFIXED_VARIABLE_2="another variable with a prefix"
export VARIABLE_1_SUFFIXED="variable with a suffix"
export VARIABLE_2_SUFFIXED="another variable with a suffix"
```

### Commands

#### default usage

Replace all variables inside `test.txt` and output the result to `stdout`:

```sh
renvsubst -i test.txt

# output:
This is a "test.txt" file.
It has more than "1" different variables.
You can also use "default" values inside variables like ${UNSET_VARIABLE:-default}.
Here are more variable like "variable with a prefix" and "variable with a suffix".
Here are more "another variable with a prefix" and "another variable with a suffix" variables!
```

#### filter variables

Replace only variable `AMOUNT` and `UNSET_VARIABLE`:

```sh
renvsubst -i test.txt --variable AMOUNT --variable UNSET_VARIABLE

# output:
This is a "$FILE_NAME" file.
It has more than "1" different variables.
You can also use "default" values inside variables like ${UNSET_VARIABLE:-default}.
Here are more variable like "${PREFIXED_VARIABLE_1}" and "${VARIABLE_1_SUFFIXED}".
Here are more "$PREFIXED_VARIABLE_2" and "$VARIABLE_2_SUFFIXED" variables!
```

#### filter with prefix

Replace only variables with the prefix `PREFIXED`

```sh
renvsubst -i test.txt --prefix PREFIXED

# output:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like ${UNSET_VARIABLE:-default}.
Here are more variable like "variable with a prefix" and "${VARIABLE_1_SUFFIXED}".
Here are more "another variable with a prefix" and "$VARIABLE_2_SUFFIXED" variables!
```

#### filter with suffix

```sh
renvsubst -i test.txt --suffix SUFFIXED

# output:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like ${UNSET_VARIABLE:-default}.
Here are more variable like "${PREFIXED_VARIABLE_1}" and "variable with a suffix".
Here are more "$PREFIXED_VARIABLE_2" and "another variable with a suffix" variables!
```

## multiple filter

```sh
renvsubst -i test.txt --prefix PREFIXED --suffix SUFFIXED

# output:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like "${UNSET_VARIABLE:-default}".
Here are more variable like "variable with a prefix" and "variable with a suffix".
Here are more "another variable with a prefix" and "another variable with a suffix" variables!
```

## container

Additionally, there is a minimal renvsubst container. You can find an example kubernetes manifest in the  `deploy` folder.
