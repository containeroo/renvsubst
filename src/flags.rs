use crate::errors::ParseArgsError;
use std::collections::HashMap;

/// A collection of command-line flags, represented as a mapping of `Flag` enums to their corresponding `FlagItem`.
///
/// This struct keeps track of the flags that have been set during argument parsing, and ensures that conflicting
/// or duplicate flags are not allowed. Provides methods for setting and retrieving flags.
#[derive(Debug, Default)]
pub struct Flags {
    flags: HashMap<Flag, FlagItem>,
}

/// A command-line flag item, containing both the flag name and its associated value.
///
/// This struct is used to represent individual flag instances in the `Flags` collection, holding
/// information about each flag's name and boolean value. The `value` field is optional, set to `None`
/// if the flag doesn't have an associated value.
#[derive(Debug, Default, Clone)]
pub struct FlagItem {
    /// The name of the flag as passed on the command line (e.g., "--fail", "-f").
    pub flag: String,

    /// The boolean value associated with the flag.
    ///
    /// This field is set to `None` if the flag does not have an associated value.
    pub value: Option<bool>,
}

/// An enumeration of possible command-line flags supported by the application.
///
/// This enum lists all the supported flags that can be used to configure the application's behavior.
/// It's used in conjunction with the `Flags` struct to store and manage the flags and their
/// associated values (if any). The `#[non_exhaustive]` attribute indicates that this enumeration
/// may be extended with new flags in the future without breaking existing code.
///
/// # Variants
///
/// * `FailOnUnset`: Enables the fail-on-unset behavior.
/// * `FailOnEmpty`: Enables the fail-on-empty behavior.
/// * `Fail`: Enables the fail behavior.
/// * `NoReplaceUnset`: Disables replacing unset variables.
/// * `NoReplaceEmpty`: Disables replacing empty variables.
/// * `NoReplace`: Disables replacing both unset and empty variables.
/// * `NoEscape`: Disables escape character interpretation.
/// * `UnbufferedLines`: Enables unbuffered lines mode.
/// * `Color`: Enables colored output.
///
/// The enum derives the following traits: `Debug`, `PartialEq`, `Eq`, `Hash`, `Copy`, and `Clone`.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[non_exhaustive]
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
    /// Sets the value of a flag, ensuring no conflicts or duplicates with other flags.
    ///
    /// * `flag_type`: The type of flag to set (e.g. `Flag::FailOnUnset`).
    /// * `flag`: The argument name (e.g. "--fail-on-unset").
    /// * `value`: The boolean value to set for the flag.
    ///
    /// Returns a `Result<(), ParseArgsError>` indicating success or the specific error that occurred.
    ///
    /// # Errors
    ///
    /// * `ParseArgsError::ConflictingFlags`: When attempting to set a flag that conflicts with a previously set flag.
    /// * `ParseArgsError::DuplicateFlag`: When attempting to set a flag that was already set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut flags = Flags::default();
    ///
    /// flags.set(Flag::FailOnUnset, "--fail-on-unset", true).unwrap();
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

    /// Retrieves the `FlagItem` associated with the given `Flag` variant, if it is set.
    ///
    /// This method looks up the `FlagItem` associated with the provided `flag_variant` in the
    /// `Flags` struct's internal `flags` `HashMap`. If the flag is set, it returns a reference to
    /// the corresponding `FlagItem`; otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `flag_variant`: A `Flag` variant representing the desired command-line flag.
    ///
    /// # Returns
    ///
    /// An `Option<&FlagItem>` that contains a reference to the `FlagItem` if the flag is set,
    /// or `None` if the flag is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut flags = Flags::default();
    /// flags.set(Flag::Fail, "--fail", true).unwrap();
    ///
    /// let fail_flag = flags.get(Flag::Fail);
    /// assert!(fail_flag.is_some());
    /// assert_eq!(flags.get(Flag::Fail).and_then(|f| f.value), None);
    /// ```
    #[must_use]
    pub fn get(&self, flag_variant: Flag) -> Option<&FlagItem> {
        self.flags.get(&flag_variant)
    }

    /// Updates the value of an existing flag in the `Flags` struct.
    ///
    /// This method sets the `value` field of the `FlagItem` associated with the given `Flag`
    /// variant to the specified `new_value`. If the `Flag` is not set in the `flags` `HashMap`,
    /// the method does nothing.
    ///
    /// # Arguments
    ///
    /// * `flag`: A `Flag` variant representing the command-line flag to update.
    /// * `new_value`: A `bool` value to update the `FlagItem`'s `value` field with.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut flags = Flags::default();
    /// flags.set(Flag::Fail, "--fail", true).unwrap();
    ///
    /// flags.update(Flag::Fail, false);
    /// assert_eq!(flags.get(Flag::Fail).and_then(|f| f.value), None);
    /// ```
    pub fn update(&mut self, flag: Flag, new_value: bool) {
        if let Some(flag_item) = self.flags.get_mut(&flag) {
            flag_item.value = Some(new_value);
        }
    }

    /// Returns a boolean indicating whether the specified flag is set.
    ///
    /// # Arguments
    ///
    /// * `flag`: A `Flag` enum value representing the flag to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the specified flag is set, otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::{Flags, Flag};
    ///
    /// let mut flags = Flags::default();
    /// flags.set(Flag::FailOnEmpty, "--fail-on-empty", true);
    ///
    /// assert_eq!(flags.is_flag_set(Flag::FailOnEmpty), true);
    /// assert_eq!(flags.is_flag_set(Flag::NoReplace), false);
    /// ```
    #[must_use]
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
