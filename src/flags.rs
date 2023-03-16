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
/// - `unbuffered_lines`: Do not buffer lines. This will print each line as soon as it is processed in chunks of 4096 bytes.
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
    unbuffered_lines: Option<bool>,
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
/// - `UnbufferedLines`: Do not buffer lines. This will print each line as soon as it is processed in chunks of 4096 bytes.
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
    UnbufferedLines,
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
    pub fn set(&mut self, flag: Flag, value: bool) -> Result<(), ParseArgsError> {
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
                        "--no-replace-unset".to_string(),
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
            Flag::UnbufferedLines => {
                if self.unbuffered_lines.is_some() {
                    return Err(ParseArgsError::DuplicateValue(
                        "--unbuffered-lines".to_string(),
                    ));
                }
                self.unbuffered_lines = Some(value);
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
    pub fn get(&self, flag: Flag) -> Option<bool> {
        match flag {
            Flag::FailOnUnset => return self.fail_on_unset,
            Flag::FailOnEmpty => return self.fail_on_empty,
            Flag::Fail => return self.fail,
            Flag::NoReplaceUnset => return self.no_replace_unset,
            Flag::NoReplaceEmpty => return self.no_replace_empty,
            Flag::NoReplace => return self.no_replace,
            Flag::NoEscape => return self.no_escape,
            Flag::UnbufferedLines => return self.unbuffered_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_success() {
        let mut flags = Flags::default();

        assert!(flags.set(Flag::NoEscape, true).is_ok());
        assert_eq!(flags.get(Flag::NoEscape), Some(true));

        assert!(flags.set(Flag::NoReplaceUnset, true).is_ok());
        assert_eq!(flags.get(Flag::NoReplaceUnset), Some(true));

        assert!(flags.set(Flag::NoReplaceEmpty, true).is_ok());
        assert_eq!(flags.get(Flag::NoReplaceEmpty), Some(true));
    }

    #[test]
    fn test_set_duplicate_value() {
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoEscape, true),
            Err(ParseArgsError::DuplicateValue("--no-escape".to_string()))
        );
    }

    #[test]
    fn test_set_conflicting_flags() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );

        assert_eq!(
            flags.set(Flag::FailOnEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_set_no_replace_and_related_flags() {
        let mut flags = Flags::default();

        flags.set(Flag::NoReplace, true).unwrap();
        assert_eq!(
            flags.set(Flag::NoReplaceUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_get_default() {
        let flags = Flags::default();

        assert_eq!(flags.get(Flag::NoEscape), None);
        assert_eq!(flags.get(Flag::NoReplaceUnset), None);
        assert_eq!(flags.get(Flag::NoReplaceEmpty), None);
        assert_eq!(flags.get(Flag::Fail), None);
        assert_eq!(flags.get(Flag::FailOnUnset), None);
        assert_eq!(flags.get(Flag::FailOnEmpty), None);
        assert_eq!(flags.get(Flag::NoReplace), None);
    }

    #[test]
    fn test_fail_fail_on_unset() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_unset_fail() {
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();

        assert_eq!(
            flags.set(Flag::Fail, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail".to_string(),
                "--fail-on-unset".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_empty_fail() {
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::Fail, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail".to_string(),
                "--fail-on-empty".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_fail_on_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_unset_already_set() {
        let mut flags = Flags::default();
        flags.set(Flag::FailOnUnset, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, true),
            Err(ParseArgsError::DuplicateValue(
                "--fail-on-unset".to_string()
            ))
        );
    }

    #[test]
    fn test_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--fail-on-empty".to_string(),
            ))
        );
    }
    #[test]
    fn test_fail_on_empty_already_set() {
        let mut flags = Flags::default();
        flags.set(Flag::FailOnEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, true),
            Err(ParseArgsError::DuplicateValue(
                "--fail-on-empty".to_string()
            ))
        );
    }

    #[test]
    fn test_flags_fail_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }
    #[test]
    fn test_flags_fail_fail_on_unset_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );
    }
    #[test]
    fn test_no_replace_empty_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--no-replace-empty".to_string(),
            ))
        );
    }
    #[test]
    fn test_no_replace_empty_fail_on_unset_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_no_replace_unset() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceUnset, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_nore_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_unset_duplicate() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceUnset, true),
            Err(ParseArgsError::DuplicateValue(
                "--no-replace-unset".to_string()
            ))
        );
    }

    #[test]
    fn test_no_replace_empty_duplicate() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, true),
            Err(ParseArgsError::DuplicateValue(
                "--no-replace-empty".to_string()
            ))
        );
    }
    #[test]
    fn test_no_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_empty_no_replace() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceEmpty, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplace, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace".to_string(),
                "--no-replace-empty".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_unset_no_replace() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplaceUnset, true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplace, true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace".to_string(),
                "--no-replace-unset".to_string(),
            ))
        );
    }

    #[test]
    fn test_unbuffered_lines() {
        let mut flags = Flags::default();
        flags.set(Flag::UnbufferedLines, true).unwrap();

        assert_eq!(
            flags.set(Flag::UnbufferedLines, true),
            Err(ParseArgsError::DuplicateValue(
                "--unbuffered-lines".to_string()
            ))
        );
    }
}
