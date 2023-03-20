# renvsubst

A command line utility to substitute (bash-like) variables in the format of `$VAR_NAME`, `${VAR_NAME}` or `${VAR_NAME:-DEFAULT_VALUE}` with their corresponding values from the environment or the default value if provided.
A valid variable name is a string that starts with a letter or an underscore, followed by any number of letters, numbers, or underscores.

The primary objective of `renvsubst` is to serve as a lightweight and high-performance utility for use in containers.

## Usage

```sh
Usage: renvsubst [FLAGS] [FILTERS] [INPUT] [OUTPUT] | [-h | --help | --version]
```

## Flags

When the same flag is provided multiple times, renvsubst will throw an error.

| Parameter            | Description                                                                    |
| -------------------- | ------------------------------------------------------------------------------ |
| `--fail-on-unset`    | Fails if an environment variable is not set.                                   |
| `--fail-on-empty`    | Fails if an environment variable is empty.                                     |
| `--fail`             | Alias for `--fail-on-unset` and `--fail-on-empty`.                             |
| `--no-replace-unset` | Does not replace variables that are not set in the environment.                |
| `--no-replace-empty` | Does not replace variables that are empty.                                     |
| `--no-replace`       | Alias for`--no-replace-unset` and `--no-replace-empty`.                        |
| `--no-escape`        | Disable escaping of variables.                                                 |
| `--unbuffer-lines`   | Do not buffer lines before printing. Saves memory, but may impact performance. |

## Filters

Every filter can be specified multiple times!

| Parameter                          | Description                                                                   |
| ---------------------------------- | ----------------------------------------------------------------------------- |
| `-p`, `--prefix`[=PREFIX]          | Only replace variables with the specified prefix.                             |
| `-s`, `--suffix`[=SUFFIX]          | Only replace variables with the specified suffix.                             |
| `-v`, `--variable`[=VARIABLE_NAME] | Specify variable to replace. If not provided, all variables will be replaced. |

The variables will be substituted according to the specified prefix, suffix, or variable name. If none of these options are provided, all variables will be substituted. When one or more options are specified, only variables that match the given prefix, suffix, or variable name will be replaced, while all others will remain unchanged.

If multiple identical prefixes, suffixes or variables are provided, only one copy of each will be used.

## Input

| Parameter              | Description                                                                                                                 |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `-i`, `--input`[=FILE] | Path to the input file. If omitted, renvsubst will read from `stdin`. To use `stdin` explicitly, use `-` as the input file. |

## Output

| Parameter               | Description                                                                                                                    |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `-o`, `--output`[=FILE] | Path to the output file. If omitted, renvsubst will write to `stdout`. To use `stdout` explicitly, use `-` as the output file. |

## General

| Parameter        | Description                      |
| ---------------- | -------------------------------- |
| `-h` \| `--help` | Show help text.                  |
| `--version`      | Show the version of the program. |

## Escaping

To retain a variable's original value and prevent it from being substituted by an environment variable, add a second dollar sign ($).
For example, to escape VAR_NAME, use `$$VAR_NAME`, `$${VAR_NAME}`, or `$${VAR_NAME:-DEFAULT_VALUE}`. The second dollar sign will be removed, resulting in `$VAR_NAME`, `${VAR_NAME}`, or `${VAR_NAME:-DEFAULT_VALUE}`.

To turn off escaping entirely, use the `--no-escape` flag.

`$${VAR_NAME}` will be replaced with `${VAR_NAME}`.
`$$VAR_NAME` will be replaced with `$VAR_NAME`.
`$${VAR_NAME:-DEFAULT_VALUE}` will be replaced with `${VAR_NAME:-DEFAULT_VALUE}`.

`I have a pa$$word` will be replaced with `I have a pa$word`. To escape this text, you have multiple options:

Escape the whole variable: `I have a pa$$$$word`.
Use the `--no-escape` flag.
Use the `--no-replace-empty` flag. If there is no environment variable named `word`, the variable will not be replaced.

## Examples

### Preparation

Create a test variable:

INPUT="""This is a "\$FILE_NAME" file.
It has more than "\${AMOUNT}" different variables.
You can also use "\${UNSET_VARIABLE:-default}" values inside variables like "\${UNSET_VARIABLE:-default}".
Here are more variable like "\${PREFIXED_VARIABLE_1}" and "\${VARIABLE_1_SUFFIXED}".
Here are more "\$PREFIXED_VARIABLE_2" and "\$VARIABLE_2_SUFFIXED" variables!
Here are other prefixed "\$prefixed_VARIABLE_3" and suffixed "\$VARIABLE_3_suffixed" variables!
Or you can escape Text with two dollar signs (\$\$) like fi\$\$h => fi\$h.
"""

Create a test file:

```sh
cat << EOF > input.txt
This is a "\$FILE_NAME" file.
It has more than "\${AMOUNT}" different variables.
You can also use "\${UNSET_VARIABLE:-default}" values inside variables like "\${UNSET_VARIABLE:-default}".
Here are more variable like "\${PREFIXED_VARIABLE_1}" and "\${VARIABLE_1_SUFFIXED}".
Here are more "\$PREFIXED_VARIABLE_2" and "\$VARIABLE_2_SUFFIXED" variables!
Here are other prefixed "\$prefixed_VARIABLE_3" and suffixed "\$VARIABLE_3_suffixed" variables!
Or you can escape Text with two dollar signs (\$\$) like fi\$\$h => fi\$h.
EOF
```

Set variables:

```sh
export FILE_NAME=input.txt
export AMOUNT=1
export PREFIXED_VARIABLE_1="variable with a prefix"
export PREFIXED_VARIABLE_2="another variable with a prefix"
export prefixed_VARIABLE_3="small letters prefix"
export VARIABLE_1_SUFFIXED="variable with a suffix"
export VARIABLE_2_SUFFIXED="another variable with a suffix"
export VARIABLE_3_suffixed="small letters suffix"
```

### Commands

#### default usage

Replace all variables inside `input.txt` and output the result to `output.txt`:

```sh
renvsubst --input input.txt --output output.txt

# output.txt:
This is a "test.txt" file.
It has more than "1" different variables.
You can also use "default" values inside variables like "default".
Here are more variable like "variable with a prefix" and "variable with a suffix".
Here are more "another variable with a prefix" and "another variable with a suffix" variables!
Here are other prefixed "small letters prefix" and suffixed "small letters suffix" variables!
Or you can escape Text with two dollar signs ($$) like fi$h => fi.
```

#### filter variables

Replace only variable `AMOUNT` and `UNSET_VARIABLE` inside `input.txt` and output to `stdout`:

```sh
renvsubst -v=AMOUNT --variable UNSET_VARIABLE < input.txt

# stdout:
This is a "$FILE_NAME" file.
It has more than "1" different variables.
You can also use "default" values inside variables like "default".
Here are more variable like "${PREFIXED_VARIABLE_1}" and "${VARIABLE_1_SUFFIXED}".
Here are more "$PREFIXED_VARIABLE_2" and "$VARIABLE_2_SUFFIXED" variables!
Here are other prefixed "$prefixed_VARIABLE_3" and suffixed "$VARIABLE_3_suffixed" variables!
Or you can escape Text with two dollar signs ($$) like fi$h => fi$h.
```

#### filter with prefix

Replace only variables with the prefix `PREFIXED` from the variable `INPUT` and write the output to the file `output.txt`:

```sh
renvsubst --prefix PREFIXED --input - <<< $INPUT > output.txt

# output.txt:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like "${UNSET_VARIABLE:-default}".
Here are more variable like "variable with a prefix" and "${VARIABLE_1_SUFFIXED}".
Here are more "another variable with a prefix" and "$VARIABLE_2_SUFFIXED" variables!
Here are other prefixed "$prefixed_VARIABLE_3" and suffixed "$VARIABLE_3_suffixed" variables!
Or you can escape Text with two dollar signs ($$) like fi$h => fi$h.
```

#### filter with suffix

Replace only variables with the suffix `SUFFIXED` inside `input.txt` and write the output to the file `output.txt`:

```sh
renvsubst --suffix=SUFFIXED -i - < input.txt > output.txt

# output.txt:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like "${UNSET_VARIABLE:-default}".
Here are more variable like "${PREFIXED_VARIABLE_1}" and "variable with a suffix".
Here are more "$PREFIXED_VARIABLE_2" and "another variable with a suffix" variables!
Here are other prefixed "$prefixed_VARIABLE_3" and suffixed "$VARIABLE_3_suffixed" variables!
Or you can escape Text with two dollar signs ($$) like fi$h => fi$h.
```

#### multiple filter

Replace only variables with the prefixes `PREFIXED` or `prefixed` or suffix `SUFFIXED` from the variable `INPUT` and output to `stdout`:

```sh
renvsubst --prefix PREFIXED --prefix=prefixed --suffix=SUFFIXED <<< $INPUT

# stdout:
This is a "$FILE_NAME" file.
It has more than "${AMOUNT}" different variables.
You can also use "${UNSET_VARIABLE:-default}" values inside variables like "$${UNSET_VARIABLE:-default}".
Here are more variable like "variable with a prefix" and "variable with a suffix".
Here are more "another variable with a prefix" and "another variable with a suffix" variables!
Here are other prefixed "" and suffixed "$VARIABLE_3_suffixed" variables!
```

## container

Furthermore, there is a `renvsubst` container available in a minimal form. In the `deploy` directory, you can find Kubernetes manifests as examples. Please note that as the container uses `scratch` as the "base image," it lacks a shell within the container. Consequently, input/output redirection will __NOT__ work at all. Instead, it is necessary to use the `-i|--input` and `-o|--output` options to pass data to `renvsubst`. Please refrain from using the `<` and `>` symbols to redirect input/output, as illustrated in the "bad" example. Instead, use the "good" example, which employs the `--input` and `--output` options to pass data.

__bad:__

```sh
renvsubst < input.txt > output.txt
```

__good:__

```sh
renvsubst --input input.txt --output output.txt
```
