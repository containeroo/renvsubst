use crate::errors::ParseArgsError;
use std::collections::HashMap;

/// `Flags` is a container that holds command line flags and their values.
/// It stores the flags in a HashMap, where the keys are of `FlagType` and the values are of `FlagItem`.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use your_crate_name::{Flags, FlagType, FlagItem};
///
/// let mut flags = Flags::default();
/// flags.set(FlagType::FailOnUnset, "--fail-on-unset", true).unwrap();
///
/// let flag_value = flags.get(FlagType::FailOnUnset).value.unwrap_or(false);
/// assert_eq!(flag_value, true);
/// ```
#[derive(Debug, Default)]
pub struct Flags {
    flags: HashMap<FlagType, FlagItem>,
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
    /// Sets a flag with the given `flag_type`, `flag`, and `value` in the `Flags` struct.
    /// This method also checks for conflicting and duplicate flags before updating the HashMap.
    ///
    /// # Arguments
    ///
    /// * `flag_type`: The type of the flag, which is an enum `FlagType`.
    /// * `flag`: The flag name as a string (e.g., "--fail-on-unset").
    /// * `value`: The boolean value associated with the flag.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if there is a conflict or duplication among the flags.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate_name::{Flags, FlagType};
    ///
    /// let mut flags = Flags::default();
    /// let flag_result = flags.set(FlagType::FailOnUnset, "--fail-on-unset", true);
    ///
    /// assert!(flag_result.is_ok());
    /// ```
    pub fn set(
        &mut self,
        flag_type: FlagType,
        flag: &str,
        value: bool,
    ) -> Result<(), ParseArgsError> {
        let conflicting_options = match flag_type {
            FlagType::FailOnUnset => vec![FlagType::Fail, FlagType::NoReplaceUnset],
            FlagType::FailOnEmpty => vec![FlagType::Fail, FlagType::NoReplaceEmpty],
            FlagType::Fail => vec![FlagType::FailOnUnset, FlagType::FailOnEmpty],
            FlagType::NoReplaceUnset => vec![FlagType::FailOnUnset, FlagType::NoReplace],
            FlagType::NoReplaceEmpty => vec![FlagType::FailOnEmpty, FlagType::NoReplace],
            FlagType::NoReplace => {
                vec![FlagType::NoReplaceUnset, FlagType::NoReplaceEmpty]
            }
            _ => vec![],
        };

        for conflicting_option in &conflicting_options {
            if let Some(conflicting_flag) = self.flags.get(conflicting_option) {
                if let Some(true) = conflicting_flag.value {
                    return Err(ParseArgsError::ConflictingFlags(
                        flag.to_string(),
                        conflicting_flag.flag.clone(),
                    ));
                }
            }
        }
        if let Some(existing_flag) = self.flags.get(&flag_type) {
            if let Some(true) = existing_flag.value {
                return Err(ParseArgsError::DuplicateFlag(flag.to_string()));
            }
        }

        self.flags.insert(
            flag_type,
            FlagItem {
                flag: flag.to_string(),
                value: Some(value),
            },
        );

        return Ok(());
    }

    /// Retrieves a reference to a `FlagItem` from the `Flags` struct based on the specified `flag_option`.
    ///
    /// # Arguments
    ///
    /// * `flag_option`: The type of the flag, which is an enum `FlagType`.
    ///
    /// # Returns
    ///
    /// An `Option<&FlagItem>` containing a reference to the `FlagItem` if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate_name::{Flags, FlagType};
    ///
    /// let mut flags = Flags::default();
    /// flags.set(FlagType::FailOnUnset, "--fail-on-unset", true).unwrap();
    ///
    /// let flag_item = flags.get(FlagType::FailOnUnset);
    /// assert!(flag_item.is_some());
    /// ```
    pub fn get(&self, flag_option: FlagType) -> Option<&FlagItem> {
        self.flags.get(&flag_option)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_success() {
        let mut flags = Flags::default();

        assert!(flags.set(FlagType::NoEscape, "--no-escape", true).is_ok());
        assert!(flags
            .get(FlagType::NoEscape)
            .map_or(false, |f| f.value.unwrap_or(false)));

        assert!(flags
            .set(FlagType::NoReplaceUnset, "--no-replace-unset", true)
            .is_ok());
        assert_eq!(
            flags
                .get(FlagType::NoReplaceUnset)
                .map_or(false, |f| f.value.unwrap_or(false)),
            true
        );

        assert!(flags
            .set(FlagType::NoReplaceEmpty, "--no-replace-empty", true)
            .is_ok());
        assert_eq!(
            flags
                .get(FlagType::NoReplaceEmpty)
                .map_or(false, |f| f.value.unwrap_or(false)),
            true
        );
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

        assert_eq!(
            flags.get(FlagType::NoEscape).map_or(None, |f| f.value),
            None
        );
        assert_eq!(
            flags
                .get(FlagType::NoReplaceUnset)
                .map_or(None, |f| f.value),
            None
        );
        assert_eq!(
            flags
                .get(FlagType::NoReplaceEmpty)
                .map_or(None, |f| f.value),
            None
        );
        assert_eq!(flags.get(FlagType::Fail).map_or(None, |f| f.value), None);
        assert_eq!(
            flags.get(FlagType::FailOnUnset).map_or(None, |f| f.value),
            None
        );
        assert_eq!(
            flags.get(FlagType::FailOnEmpty).map_or(None, |f| f.value),
            None
        );
        assert_eq!(
            flags.get(FlagType::NoReplace).map_or(None, |f| f.value),
            None
        );
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
