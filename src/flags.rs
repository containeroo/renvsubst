use crate::errors::ParseArgsError;

/// The `Flags` struct represents the various command-line flags that can be used to modify
/// the behavior of `renvsubst`. Each flag is represented by an `Option<bool>` value, which
/// is `None` by default. When a flag is set to `Some(true)`, it indicates that the flag is
/// enabled. When a flag is set to `Some(false)`, it indicates that the flag is disabled.
///
/// The available flags are:
///
/// - `fail_on_unset`: Fails if an environment variable is not set.
///
/// - `fail_on_empty`: Fails if an environment variable is empty.
///
/// - `no_replace_unset`: Does not replace variables that are not set in the environment.
///
/// - `no_replace_empty`: Does not replace variables that are set but empty in the environment.
///
/// - `no_escape`: Disables escaping of variables with two dollar signs ($$).
///
#[derive(Debug, Default)]
pub struct Flags {
    fail_on_unset: Option<bool>,
    fail_on_empty: Option<bool>,
    fail: Option<bool>,
    no_replace_unset: Option<bool>,
    no_replace_empty: Option<bool>,
    no_replace: Option<bool>,
    no_escape: Option<bool>,
}

/// The `Flag` enum represents the various command-line flags that can be used to modify
/// the behavior of `renvsubst`. Each flag corresponds to a specific behavior that can be
/// enabled or disabled using a command-line flag.
///
/// The available flags are:
///
/// - `FailOnUnset`: Fails if an environment variable is not set.
///
/// - `FailOnEmpty`: Fails if an environment variable is empty.
///
/// - `NoReplaceUnset`: Does not replace variables that are not set in the environment.
///
/// - `NoReplaceEmpty`: Does not replace variables that are set but empty in the environment.
///
/// - `NoEscape`: Disables escaping of variables with two dollar signs ($$).
///
/// This enum implements the `Copy` and `Clone` traits, allowing it to be easily copied and
/// cloned as needed. It also implements the `Debug`, `PartialEq`, `Eq`, and `Hash` traits for
/// easy debugging and comparison.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Flag {
    FailOnUnset,
    FailOnEmpty,
    Fail,
    NoReplaceUnset,
    NoReplaceEmpty,
    NoReplace,
    NoEscape,
}

impl Flags {
    /// Set a flag to a boolean value.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to set.
    /// * `value` - The value to set the flag to.
    ///
    /// # Errors
    ///
    /// Returns an error if the flag is already set or if there are conflicting flags.
    pub fn set_flag(&mut self, flag: Flag, value: bool) -> Result<(), ParseArgsError> {
        match flag {
            Flag::FailOnUnset => {
                if self.fail == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail-on-unset".to_string(),
                        "--fail".to_string(),
                    ));
                }
                if self.no_replace_unset == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail-on-unset".to_string(),
                        "--no-replace-unset".to_string(),
                    ));
                }
                if self.fail_on_unset.is_some() {
                    return Err(ParseArgsError::DuplicateValue(
                        "--fail-on-unset".to_string(),
                    ));
                }

                self.fail_on_unset = Some(value);
                return Ok(());
            }
            Flag::FailOnEmpty => {
                if self.fail == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail-on-empty".to_string(),
                        "--fail".to_string(),
                    ));
                }
                if self.no_replace_empty == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail-on-empty".to_string(),
                        "--no-replace-empty".to_string(),
                    ));
                }
                if self.fail_on_empty.is_some() {
                    return Err(ParseArgsError::DuplicateValue(
                        "--fail-on-empty".to_string(),
                    ));
                }
                self.fail_on_empty = Some(value);
                return Ok(());
            }
            Flag::Fail => {
                if self.fail_on_unset == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail".to_string(),
                        "--fail-on-unset".to_string(),
                    ));
                }
                if self.fail_on_empty == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--fail".to_string(),
                        "--fail-on-empty".to_string(),
                    ));
                }
                self.fail = Some(value);
                self.fail_on_unset = Some(value);
                self.fail_on_empty = Some(value);

                return Ok(());
            }
            Flag::NoReplaceUnset => {
                if self.no_replace == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace-unset".to_string(),
                        "--no-replace".to_string(),
                    ));
                }
                if self.fail_on_unset == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace-unset".to_string(),
                        "--fail-on-unset".to_string(),
                    ));
                }
                if self.no_replace_unset.is_some() {
                    return Err(ParseArgsError::DuplicateValue(
                        "--no-replace-unset".to_string(),
                    ));
                }
                self.no_replace_unset = Some(value);

                return Ok(());
            }
            Flag::NoReplaceEmpty => {
                if self.no_replace == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace-empty".to_string(),
                        "--no-replace".to_string(),
                    ));
                }
                if self.fail_on_empty == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace-empty".to_string(),
                        "--fail-on-empty".to_string(),
                    ));
                }
                if self.no_replace_empty.is_some() {
                    return Err(ParseArgsError::DuplicateValue(
                        "--no-replace-empty".to_string(),
                    ));
                }
                self.no_replace_empty = Some(value);

                return Ok(());
            }
            Flag::NoReplace => {
                if self.no_replace_unset == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace".to_string(),
                        "--no_replace-unset".to_string(),
                    ));
                }
                if self.no_replace_empty == Some(true) {
                    return Err(ParseArgsError::ConflictingFlags(
                        "--no-replace".to_string(),
                        "--no-replace-empty".to_string(),
                    ));
                }
                self.no_replace = Some(value);
                self.fail_on_unset = Some(value);
                self.fail_on_empty = Some(value);

                return Ok(());
            }
            Flag::NoEscape => {
                if self.no_escape.is_some() {
                    return Err(ParseArgsError::DuplicateValue("--no-escape".to_string()));
                }
                self.no_escape = Some(value);
                Ok(())
            }
        }
    }

    /// Get the value of a flag.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to get the value of.
    ///
    /// # Returns
    ///
    /// Returns the value of the flag, or None if the flag has not been set.
    pub fn get_flag(&self, flag: Flag) -> Option<bool> {
        match flag {
            Flag::FailOnUnset => return self.fail_on_unset,
            Flag::FailOnEmpty => return self.fail_on_empty,
            Flag::Fail => return self.fail,
            Flag::NoReplaceUnset => return self.no_replace_unset,
            Flag::NoReplaceEmpty => return self.no_replace_empty,
            Flag::NoReplace => return self.no_replace,
            Flag::NoEscape => return self.no_escape,
        }
    }
}
