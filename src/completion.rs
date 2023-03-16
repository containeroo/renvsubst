use crate::errors::ParseArgsError;

/// Struct to hold the completion type for the program.
#[derive(Debug, Default, PartialEq)]
pub struct Completion {
    pub completion: Option<String>,
}

impl Completion {
    /// Sets the completion type.
    ///
    /// # Arguments
    ///
    /// * `completion_type` - A string slice that represents the completion type to be set.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the completion type is valid and has been set successfully.
    /// * `Err(ParseArgsError)` if the completion type is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// let mut completion = Completion::default();
    /// completion.set("bash");
    /// ```
    pub fn set(&mut self, completion_type: &str) -> Result<(), ParseArgsError> {
        match completion_type {
            "bash" => {
                self.completion = Some(BASH_COMPLETION.to_string());
                Ok(())
            }
            "zsh" => {
                self.completion = Some(ZSH_COMPLETION.to_string());
                Ok(())
            }
            _ => Err(ParseArgsError::InvalidCompletionType(
                completion_type.to_string(),
            )),
        }
    }

    /// Unwraps the completion type and returns it as a string slice.
    ///
    /// # Returns
    ///
    /// * A string slice that represents the completion type.
    ///
    /// # Panics
    ///
    /// Panics if the completion type is None.
    ///
    /// # Example
    ///
    /// ```
    /// let completion = Completion { completion: Some("bash".to_string()) };
    /// let completion_type = completion.unwrap();
    /// ```
    pub fn unwrap(&self) -> &str {
        self.completion.as_ref().unwrap()
    }
}

const BASH_COMPLETION: &str = "#!/bin/bash

# Define the completion function
_parse_completion() {
    # Get the current word being completed
    local current_word=\"${COMP_WORDS[COMP_CWORD]}\"

    # Check if the current word is a flag that requires a value
    local flags_with_values=(
        \"--prefix\"
        \"-p\"
        \"--suffix\"
        \"-s\"
        \"--variable\"
        \"-v\"
    )
    for flag in \"${flags_with_values[@]}\"; do
        if [[ \"$flag\" == \"$current_word\"* ]]; then
            # Complete with files and directories
            COMPREPLY=( $(compgen -f -- \"$current_word\") )
            return 0
        fi
    done

    # Check if the current word is a flag
    local flags=(
        \"-h\"
        \"--help\"
        \"--version\"
        \"--fail-on-unset\"
        \"--fail-on-empty\"
        \"--fail\"
        \"--no-replace-unset\"
        \"--no-replace-empty\"
        \"--no-replace\"
        \"--no-escape\"
        \"-p\"
        \"--prefix\"
        \"-s\"
        \"--suffix\"
        \"-v\"
        \"--variable\"
    )
    for flag in \"${flags[@]}\"; do
        if [[ \"$flag\" == \"$current_word\"* ]]; then
            # Complete with the flag
            COMPREPLY=( \"$flag\" )
            return 0
        fi
    done

    # Otherwise, no completion
    COMPREPLY=()
    return 0
}

# Register the completion function
complete -F _parse_completion parse
";

const ZSH_COMPLETION: &str = "#compdef parse

# Define the completion function
_parse_completion() {
    # Get the current word being completed
    local current_word=${words[-1]}

    # Check if the current word is a flag that requires a value
    local flags_with_values=(
        \"--prefix\"
        \"-p\"
        \"--suffix\"
        \"-s\"
        \"--variable\"
        \"-v\"
    )
    if [[ \"${flags_with_values[@]}\" =~ \"${current_word}\" ]]; then
        # Complete with files and directories
        _files
        return
    fi

    # Check if the current word is a flag
    local flags=(
        \"-h\"
        \"--help\"
        \"--version\"
        \"--fail-on-unset\"
        \"--fail-on-empty\"
        \"--fail\"
        \"--no-replace-unset\"
        \"--no-replace-empty\"
        \"--no-replace\"
        \"--no-escape\"
        \"-p\"
        \"--prefix\"
        \"-s\"
        \"--suffix\"
        \"-v\"
        \"--variable\"
    )
    if [[ \"${flags[@]}\" =~ \"${current_word}\" ]]; then
        # Complete with the flag
        _describe -t options 'flag' $flags
        return
    fi

    # Otherwise, no completion
    return
}

# Register the completion function
compdef _parse_completion parse
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bash_completion() {
        let mut completion = Completion::default();
        completion.set("bash").unwrap();
        assert_eq!(completion.completion, Some(BASH_COMPLETION.to_string()));
    }

    #[test]
    fn test_set_zsh_completion() {
        let mut completion = Completion::default();
        completion.set("zsh").unwrap();
        assert_eq!(completion.completion, Some(ZSH_COMPLETION.to_string()));
    }

    #[test]
    fn test_set_invalid_completion_type() {
        let mut completion = Completion::default();
        let result = completion.set("invalid");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::InvalidCompletionType("invalid".to_string())
        );
    }

    #[test]
    fn test_unwrap_completion() {
        let mut completion = Completion::default();
        completion.set("bash").unwrap();
        assert_eq!(completion.unwrap(), BASH_COMPLETION);
    }
}
