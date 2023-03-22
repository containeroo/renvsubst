use crate::errors::ParseArgsError;
use std::collections::HashMap;

/// `Flags` is a container that holds command line flags and their values.
/// It stores the flags in a `HashMap`, where the keys are of `Flag` and the values are of `FlagItem`.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use your_crate_name::{Flags, Flag, FlagItem};
///
/// let mut flags = Flags::default();
/// flags.set(Flag::FailOnUnset, "--fail-on-unset", true).unwrap();
///
/// let flag_value = flags.get(Flag::FailOnUnset).value.unwrap_or(false);
/// assert_eq!(flag_value, true);
/// ```
#[derive(Debug, Default)]
pub struct Flags {
    flags: HashMap<Flag, FlagItem>,
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
pub enum Flag {
    FailOnUnset,
    FailOnEmpty,
    Fail,
    NoReplaceUnset,
    NoReplaceEmpty,
    NoReplace,
    NoEscape,
    UnbufferedLines,
    Color,
}

impl Flags {
    /// Sets a flag with the given `flag_type`, `flag`, and `value` in the `Flags` struct.
    /// This method also checks for conflicting and duplicate flags before updating the `HashMap`.
    ///
    /// # Arguments
    ///
    /// * `flag_type`: The type of the flag, which is an enum `Flag`.
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
    /// use your_crate_name::{Flags, Flag};
    ///
    /// let mut flags = Flags::default();
    /// let flag_result = flags.set(Flag::FailOnUnset, "--fail-on-unset", true);
    ///
    /// assert!(flag_result.is_ok());
    /// ```
    pub fn set(&mut self, flag_type: Flag, flag: &str, value: bool) -> Result<(), ParseArgsError> {
        let conflicting_options = match flag_type {
            Flag::FailOnUnset => vec![Flag::Fail, Flag::NoReplaceUnset],
            Flag::FailOnEmpty => vec![Flag::Fail, Flag::NoReplaceEmpty],
            Flag::Fail => vec![
                Flag::FailOnUnset,
                Flag::FailOnEmpty,
                Flag::NoReplace,
                Flag::NoReplaceUnset,
                Flag::NoReplaceEmpty,
            ],
            Flag::NoReplaceUnset => vec![Flag::FailOnUnset, Flag::NoReplace],
            Flag::NoReplaceEmpty => vec![Flag::FailOnEmpty, Flag::NoReplace],
            Flag::NoReplace => {
                vec![
                    Flag::NoReplaceUnset,
                    Flag::NoReplaceEmpty,
                    Flag::Fail,
                    Flag::FailOnUnset,
                    Flag::FailOnEmpty,
                ]
            }
            _ => vec![],
        };

        // Check for conflicting flags
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

        // Check for duplicate flags
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
    /// * `flag_option`: The type of the flag, which is an enum `Flag`.
    ///
    /// # Returns
    ///
    /// An `Option<&FlagItem>` containing a reference to the `FlagItem` if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate_name::{Flags, Flag};
    ///
    /// let mut flags = Flags::default();
    /// flags.set(Flag::FailOnUnset, "--fail-on-unset", true).unwrap();
    ///
    /// let flag_item = flags.get(Flag::FailOnUnset);
    /// assert!(flag_item.is_some());
    /// ```
    pub fn get(&self, flag_option: Flag) -> Option<&FlagItem> {
        self.flags.get(&flag_option)
    }

    /// Update the value of a specific flag in the `Flags` struct.
    ///
    /// This method takes a `Flag` and a new boolean value as arguments and updates
    /// the `value` field of the corresponding `FlagItem` in the `flags` `HashMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_module::{Flags, Flag};
    ///
    /// let mut flags = Flags::default();
    /// flags.update_flag_value(Flag::Color, false);
    ///
    /// assert_eq!(flags.get(Flag::Color).unwrap().value, Some(false));
    /// ```
    ///
    /// # Arguments
    ///
    /// * `flag` - A `Flag` enum variant to identify the flag to update.
    /// * `new_value` - The new boolean value to set for the specified flag.
    pub fn update(&mut self, flag: Flag, new_value: bool) {
        if let Some(flag_item) = self.flags.get_mut(&flag) {
            flag_item.value = Some(new_value);
        }
    }

    /// Returns `true` if the specified `flag` is set in the `flags` `HashMap`, and its value is `true`.
    /// Returns `false` otherwise (i.e., if the flag is not set, or its value is `false`).
    ///
    /// # Arguments
    ///
    /// * `flag` - The `Flag` enum value to check if set and true in the `flags` `HashMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `parsed_args` is an instance of a struct containing a `Flags` instance.
    /// let flag_set = parsed_args.flags.is_flag_set(Flag::Fail);
    /// assert_eq!(flag_set, true);
    /// ```
    pub fn is_flag_set(&self, flag: Flag) -> bool {
        self.flags
            .get(&flag)
            .map_or(false, |f| f.value.unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_success() {
        let mut flags = Flags::default();

        assert!(flags.set(Flag::NoEscape, "--no-escape", true).is_ok());
        assert_eq!(flags.get(Flag::NoEscape).and_then(|f| f.value), Some(true));
        assert!(flags
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .is_ok());
        assert_eq!(
            flags.get(Flag::NoReplaceUnset).and_then(|f| f.value),
            Some(true)
        );

        assert!(flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .is_ok());
        assert_eq!(
            flags.get(Flag::NoReplaceEmpty).and_then(|f| f.value),
            Some(true)
        );
    }

    #[test]
    fn test_set_duplicate_value() {
        let mut flags = Flags::default();
        flags.set(Flag::NoEscape, "--no-escape", true).unwrap();

        assert_eq!(
            flags.set(Flag::NoEscape, "--no-escape", true),
            Err(ParseArgsError::DuplicateFlag("--no-escape".to_string()))
        );
    }

    #[test]
    fn test_set_conflicting_flags() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-unset".to_string(),
                "--fail".to_string(),
            ))
        );

        assert_eq!(
            flags.set(Flag::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }

    #[test]
    fn test_set_no_replace_and_related_flags() {
        let mut flags = Flags::default();

        flags.set(Flag::NoReplace, "--no-replace", true).unwrap();
        assert_eq!(
            flags.set(Flag::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-empty".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_get_defaults() {
        let flags = Flags::default();

        assert_eq!(flags.get(Flag::NoEscape).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::NoReplaceUnset).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::NoReplaceEmpty).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::Fail).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::FailOnUnset).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::FailOnEmpty).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::NoReplace).and_then(|f| f.value), None);
        assert_eq!(flags.get(Flag::Color).and_then(|f| f.value), None);
    }

    #[test]
    fn test_fail_duplicate_long_long() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::Fail, "--fail", true),
            Err(ParseArgsError::DuplicateFlag("--fail".to_string(),))
        );
    }

    #[test]
    fn test_fail_duplicate_long_short() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::Fail, "-f", true),
            Err(ParseArgsError::DuplicateFlag("-f".to_string(),))
        );
    }

    #[test]
    fn test_fail_duplicate_short_long() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "-f", true).unwrap();

        assert_eq!(
            flags.set(Flag::Fail, "--fail", true),
            Err(ParseArgsError::DuplicateFlag("--fail".to_string(),))
        );
    }

    #[test]
    fn test_fail_fail_on_unset() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, "--fail-on-unset", true),
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
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::Fail, "--fail", true),
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
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::Fail, "--fail", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail".to_string(),
                "--fail-on-empty".to_string(),
            ))
        );
    }

    #[test]
    fn test_fail_fail_on_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, "--fail-on-empty", true),
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
            .set(Flag::FailOnUnset, "--fail-on-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, "--fail-on-unset", true),
            Err(ParseArgsError::DuplicateFlag("--fail-on-unset".to_string()))
        );
    }

    #[test]
    fn test_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, "--no-replace-empty", true),
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
            .set(Flag::FailOnEmpty, "--fail-on-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::DuplicateFlag("--fail-on-empty".to_string()))
        );
    }

    #[test]
    fn test_flags_fail_fail_on_empty_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, "--fail-on-empty", true),
            Err(ParseArgsError::ConflictingFlags(
                "--fail-on-empty".to_string(),
                "--fail".to_string(),
            ))
        );
    }
    #[test]
    fn test_flags_fail_fail_on_unset_conflict() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "--fail", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, "--fail-on-unset", true),
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
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::FailOnEmpty, "-e", true),
            Err(ParseArgsError::ConflictingFlags(
                "-e".to_string(),
                "--no-replace-empty".to_string(),
            ))
        );
    }
    #[test]
    fn test_no_replace_empty_fail_on_unset_conflict_short_short() {
        let mut flags = Flags::default();
        flags.set(Flag::Fail, "-f", true).unwrap();

        assert_eq!(
            flags.set(Flag::FailOnUnset, "-u", true),
            Err(ParseArgsError::ConflictingFlags(
                "-u".to_string(),
                "-f".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_no_replace_unset() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, "--no-replace", true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace-unset".to_string(),
                "--no-replace".to_string(),
            ))
        );
    }

    #[test]
    fn test_nore_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, "--no-replace", true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, "--no-replace-empty", true),
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
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceUnset, "--no-replace-unset", true),
            Err(ParseArgsError::DuplicateFlag(
                "--no-replace-unset".to_string()
            ))
        );
    }

    #[test]
    fn test_no_replace_empty_duplicate() {
        let mut flags = Flags::default();
        flags
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, "--no-replace-empty", true),
            Err(ParseArgsError::DuplicateFlag(
                "--no-replace-empty".to_string()
            ))
        );
    }
    #[test]
    fn test_no_replace_no_replace_empty() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, "--no-replace", true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplaceEmpty, "--no-replace-empty", true),
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
            .set(Flag::NoReplaceEmpty, "--no-replace-empty", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::NoReplace, "--no-replace", true),
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
            .set(Flag::NoReplaceUnset, "--no-replace-unset", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::NoReplace, "--no-replace", true),
            Err(ParseArgsError::ConflictingFlags(
                "--no-replace".to_string(),
                "--no-replace-unset".to_string(),
            ))
        );
    }

    #[test]
    fn test_no_replace_duplicate() {
        let mut flags = Flags::default();
        flags.set(Flag::NoReplace, "--no-replace", true).unwrap();

        assert_eq!(
            flags.set(Flag::NoReplace, "--no-replace", true),
            Err(ParseArgsError::DuplicateFlag("--no-replace".to_string()))
        );
    }

    #[test]
    fn test_unbuffered_lines() {
        let mut flags = Flags::default();
        flags
            .set(Flag::UnbufferedLines, "--unbuffer-lines", true)
            .unwrap();

        assert_eq!(
            flags.set(Flag::UnbufferedLines, "--unbuffer-lines", true),
            Err(ParseArgsError::DuplicateFlag(
                "--unbuffer-lines".to_string()
            ))
        );
    }
}
