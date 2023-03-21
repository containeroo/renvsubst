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
    fail_on_unset: FlagItem,
    fail_on_empty: FlagItem,
    fail: FlagItem,
    no_replace_unset: FlagItem,
    no_replace_empty: FlagItem,
    no_replace: FlagItem,
    no_escape: FlagItem,
    unbuffered_lines: FlagItem,
}

/// A `FlagItem` represents a command line flag and its associated value.
///
/// Each flag has a name and an optional boolean value. The `flag` field
/// stores the name of the flag as it appears on the command line, while the `value`
/// field stores the boolean value that is associated with the flag.
#[derive(Debug, Default, Clone)]
pub struct FlagItem {
    /// The name of the flag as passed on the command line (e.g., "--fail", "-f").
    pub flag: String,

    /// The boolean value associated with the flag.
    ///
    /// This field is set to `None` if the flag does not have an associated value.
    pub value: Option<bool>,
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
pub enum FlagType {
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
    /// Sets the value of the given `flag_type` in the `Flags` struct.
    ///
    /// This function checks for conflicts and duplicates before setting the flag's value.
    /// If a conflict or duplicate is found, an appropriate error is returned.
    ///
    /// # Arguments
    ///
    /// * `flag_type` - The flag type to set.
    /// * `flag` - The flag's name.
    /// * `value` - The value to set for the flag.
    ///
    /// # Returns
    ///
    /// * A `Result` indicating the success or failure of the operation.
    pub fn set(
        &mut self,
        flag_type: FlagType,
        flag: &str,
        value: bool,
    ) -> Result<(), ParseArgsError> {
        match flag_type {
            // Handle each flag type separately to check for conflicts and duplicates
            FlagType::FailOnUnset => {
                // Check for conflicts with other flags
                if let Some(true) = self.fail.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail.flag.clone(),
                    ));
                }
                if let Some(true) = self.no_replace_unset.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace_unset.flag.clone(),
                    ));
                }

                // Check for duplicates
                if let Some(true) = self.fail_on_unset.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.fail_on_unset = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::FailOnEmpty => {
                // Check for conflicts with other flags
                if let Some(true) = self.fail.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail.flag.clone(),
                    ));
                }
                if let Some(true) = self.no_replace_empty.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace_empty.flag.clone(),
                    ));
                }

                // Check for duplicates
                if let Some(true) = self.fail_on_empty.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.fail_on_empty = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::Fail => {
                // Check for duplicates
                if let Some(true) = self.fail.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Check for conflicts with other flags
                if let Some(true) = self.fail_on_unset.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail_on_unset.flag.clone(),
                    ));
                }
                if let Some(true) = self.fail_on_empty.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail_on_empty.flag.clone(),
                    ));
                }

                // Set the flag value
                self.fail = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };
                self.fail_on_unset = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };
                self.fail_on_empty = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::NoReplaceUnset => {
                // Check for conflicts with other flags
                if let Some(true) = self.no_replace.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace.flag.clone(),
                    ));
                }
                if let Some(true) = self.fail_on_unset.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail_on_unset.flag.clone(),
                    ));
                }

                // Check for duplicates
                if let Some(true) = self.no_replace_unset.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.no_replace_unset = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::NoReplaceEmpty => {
                // Check for conflicts with other flags
                if let Some(true) = self.no_replace.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace.flag.clone(),
                    ));
                }
                if let Some(true) = self.fail_on_empty.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.fail_on_empty.flag.clone(),
                    ));
                }

                // Check for duplicates
                if let Some(true) = self.no_replace_empty.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.no_replace_empty = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::NoReplace => {
                // Check for duplicates
                if let Some(true) = self.no_replace.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Check for conflicts with other flags
                if let Some(true) = self.no_replace_unset.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace_unset.flag.clone(),
                    ));
                }
                if let Some(true) = self.no_replace_empty.value.as_ref() {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        self.no_replace_empty.flag.clone(),
                    ));
                }

                // Set the flag value
                self.no_replace = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                self.no_replace_unset = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                self.fail_on_empty = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::NoEscape => {
                // Check for duplicates
                if let Some(true) = self.no_escape.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.no_escape = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
            }
            FlagType::UnbufferedLines => {
                // Check for duplicates
                if let Some(true) = self.unbuffered_lines.value.as_ref() {
                    return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
                }

                // Set the flag value
                self.unbuffered_lines = FlagItem {
                    flag: flag.to_string(),
                    value: Some(value),
                };

                return Ok(());
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
    pub fn get(&self, flag: FlagType) -> &FlagItem {
        match flag {
            FlagType::FailOnUnset => return &self.fail_on_unset,
            FlagType::FailOnEmpty => return &self.fail_on_empty,
            FlagType::Fail => return &self.fail,
            FlagType::NoReplaceUnset => return &self.no_replace_unset,
            FlagType::NoReplaceEmpty => return &self.no_replace_empty,
            FlagType::NoReplace => return &self.no_replace,
            FlagType::NoEscape => return &self.no_escape,
            FlagType::UnbufferedLines => return &self.unbuffered_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_success() {
        let mut flags = Flags::default();

        assert!(flags.set(FlagType::NoEscape, "--no-escape", true).is_ok());
        assert_eq!(flags.get(FlagType::NoEscape).value, Some(true));

        assert!(flags
            .set(FlagType::NoReplaceUnset, "--no-replace-unset", true)
            .is_ok());
        assert_eq!(flags.get(FlagType::NoReplaceUnset).value, Some(true));

        assert!(flags
            .set(FlagType::NoReplaceEmpty, "--no-replace-empty", true)
            .is_ok());
        assert_eq!(flags.get(FlagType::NoReplaceEmpty).value, Some(true));
    }

    #[test]
    fn test_set_duplicate_value() {
        let mut flags = Flags::default();
        flags.set(FlagType::NoEscape, "--no-escape", true).unwrap();

        assert_eq!(
            flags.set(FlagType::NoEscape, "--no-escape", true),
            Err(ParseArgsError::DuplicateFlag("--no-escape".to_string()))
        );
    }

    #[test]
    fn test_set_conflicting_flags() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );

        assert_eq!(
            flags.set(FlagType::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_set_no_replace_and_related_flags() {
        let mut flags = Flags::default();

        flags
            .set(FlagType::NoReplace, "--no-replace", true)
            .unwrap();
        assert_eq!(
            flags.set(FlagType::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );

        assert_eq!(
            flags.set(FlagType::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_get_default() {
        let flags = Flags::default();

        assert_eq!(flags.get(FlagType::NoEscape).value, None);
        assert_eq!(flags.get(FlagType::NoReplaceUnset).value, None);
        assert_eq!(flags.get(FlagType::NoReplaceEmpty).value, None);
        assert_eq!(flags.get(FlagType::Fail).value, None);
        assert_eq!(flags.get(FlagType::FailOnUnset).value, None);
        assert_eq!(flags.get(FlagType::FailOnEmpty).value, None);
        assert_eq!(flags.get(FlagType::NoReplace).value, None);
    }

    #[test]
    fn test_fail_duplicate_long_long() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::Fail, "--fail", true),
            Err(ParseArgsError::DuplicateFlag("--fail".to_string(),))
        );
    }

    #[test]
    fn test_fail_duplicate_long_short() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::Fail, "-f", true),
            Err(ParseArgsError::DuplicateFlag("-f".to_string(),))
        );
    }

    #[test]
    fn test_fail_duplicate_short_long() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "-f", true).unwrap();

        assert_eq!(
            flags.set(FlagType::Fail, "--fail", true),
            Err(ParseArgsError::DuplicateFlag("--fail".to_string(),))
        );
    }

    #[test]
    fn test_fail_fail_on_unset() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_unset_fail() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::FailOnUnset, "--fail-on-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::Fail, "--fail", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail".to_string(),
                "--fail-on-unset".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_empty_fail() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::Fail, "--fail", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail".to_string(),
                "--fail-on-empty".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_fail_on_empty() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_on_unset_already_set() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::FailOnUnset, "--fail-on-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::DuplicateFlag("--fail-on-unset".to_string()))
        );
    }

    #[test]
    fn test_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--fail-on-empty".to_string(),
            ))
        );
    }
    #[test]
    fn test_fail_on_empty_already_set() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::DuplicateFlag("--fail-on-empty".to_string()))
        );
    }

    #[test]
    fn test_flags_fail_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }
    #[test]
    fn test_flags_fail_fail_on_unset_conflict() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );
    }
    #[test]
    fn test_no_replace_empty_fail_on_empty_conflict_long_short() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnEmpty, "-e", true),
            Err(ParseArgsError::ConflictingFlags(
                "-e".to_string(),
                "--no-replace-empty".to_string(),
            ))
        );
    }
    #[test]
    fn test_no_replace_empty_fail_on_unset_conflict_short_short() {
        let mut flags = Flags::default();
        flags.set(FlagType::Fail, "-f", true).unwrap();

        assert_eq!(
            flags.set(FlagType::FailOnUnset, "-u", true),
            Err(ParseArgsError::ConflictingFlags(
                "-u".to_string(),
                "-f".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_no_replace_unset() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplace, "--no-replace", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_nore_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplace, "--no-replace", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_unset_duplicate() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplaceUnset, "--no-replace-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::DuplicateFlag(
                "--no-replace-unset".to_string()
            ))
        );
    }

    #[test]
    fn test_no_replace_empty_duplicate() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::DuplicateFlag(
                "--no-replace-empty".to_string()
            ))
        );
    }
    #[test]
    fn test_no_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplace, "--no-replace", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_empty_no_replace() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplace, "--no-replace", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace".to_string(),
                "--no-replace-empty".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_unset_no_replace() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplaceUnset, "--no-replace-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplace, "--no-replace", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace".to_string(),
                "--no-replace-unset".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_duplicate() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::NoReplace, "--no-replace", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::NoReplace, "--no-replace", true),
            Err(ParseArgsError::DuplicateFlag("--no-replace".to_string()))
        );
    }

    #[test]
    fn test_unbuffered_lines() {
        let mut flags = Flags::default();
        flags
            .set(FlagType::UnbufferedLines, "--unbuffer-lines", true)
            .unwrap();

        assert_eq!(
            flags.set(FlagType::UnbufferedLines, "--unbuffer-lines", true),
            Err(ParseArgsError::DuplicateFlag(
                "--unbuffer-lines".to_string()
            ))
        );
    }
}
