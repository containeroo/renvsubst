use crate::errors::ParseArgsError;
use crate::utils::START_PARAMETERS;
use std::collections::HashSet;

/// `Filters` is a struct that holds optional sets of string prefixes, suffixes, and variables
/// for filtering environment variable replacements. Each field contains an `Option<HashSet<String>>`,
/// allowing for the possibility of an empty or uninitialized set.
///
/// The `Debug` and `Default` traits are derived for the `Filters` struct, enabling easy debugging
/// and the creation of default instances.
#[derive(Debug, Default)]
pub struct Filters {
    /// A set of string prefixes.
    pub prefixes: Option<HashSet<String>>,
    /// A set of string suffixes.
    pub suffixes: Option<HashSet<String>>,
    /// A set of string variables.
    pub variables: Option<HashSet<String>>,
}

/// `Filter` is an enumeration representing the different types of filters
/// available for filtering environment variable replacements.
///
/// The available filter types are:
/// * `Prefix`: Filters based on string prefixes.
/// * `Suffix`: Filters based on string suffixes.
/// * `Variable`: Filters based on the complete variable name.
///
/// The enum derives the following traits: `Debug`, `PartialEq`, `Eq`, `Hash`, `Copy`, and `Clone`.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Filter {
    Prefix,
    Suffix,
    Variable,
}

impl Filters {
    /// Adds a filter to the `Filters` struct based on a command-line argument and its value.
    ///
    /// # Arguments
    ///
    /// * `filter`: A `Filter` enum variant specifying the type of filter to add.
    /// * `arg`: A string slice containing the command-line argument to parse.
    /// * `value`: An optional string slice containing the value of the argument.
    /// * `iter`: A mutable reference to an iterator over the remaining command-line arguments.
    ///
    /// # Returns
    ///
    /// A `Result<(), ParseArgsError>` indicating whether the operation was successful, or an
    /// error if there was an issue with parsing the arguments.
    ///
    /// # Errors
    ///
    /// This function can return the following errors:
    ///
    /// * `ParseArgsError::MissingValue(arg)` - When a value is missing for the specified argument.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = Filters::new();
    /// filters.add(Filter::Prefix, "--prefix", Some("prefix_"), &mut ["-v".to_string()].iter())
    ///     .unwrap();
    /// assert_eq!(filters.prefixes.unwrap().contains("prefix_"), true);
    /// ```
    ///
    /// # Notes
    ///
    /// This function is used to add filters to the `Filters` struct based on command-line arguments
    /// and their values. The `filter` parameter specifies the type of filter to add, and the `arg`
    /// and `value` parameters provide the command-line argument and its value, respectively.
    ///
    /// If `value` is `None`, the function will attempt to get the value from the next argument in
    /// the command-line arguments iterator `iter`. If the value is not found, the function will
    /// return a `ParseArgsError::MissingValue` error.
    ///
    /// If the value of the argument is in the `START_PARAMETERS` list, the function will return a
    /// `ParseArgsError::MissingValue` error, since the value is expected to be a filter value and
    /// not another parameter. Otherwise, the function will add the value to the appropriate filter
    /// in the `Filters` struct, based on the value of the `filter` parameter.
    pub fn add(
        &mut self,
        filter: Filter,
        arg: &str,
        value: Option<&str>,
        iter: &mut std::slice::Iter<String>,
    ) -> Result<(), ParseArgsError> {
        let flag_arg: String = value.map_or_else(
            // if no value is provided... (was not --prefix=prefix_)
            || {
                // if not, get the next argument as the value
                iter.next()
                    .map(std::string::ToString::to_string) // convert the value to a string
                    // return an error if the value is missing
                    .ok_or_else(|| ParseArgsError::MissingValue(arg.to_string()))
            },
            |s| Ok(s.to_string()), // return the value if it exists
        )?;

        if START_PARAMETERS.contains(&flag_arg.as_str()) {
            return Err(ParseArgsError::MissingValue(arg.to_string()));
        }

        match filter {
            Filter::Prefix => {
                self.prefixes
                    .get_or_insert_with(HashSet::new)
                    .insert(flag_arg);
            }
            Filter::Suffix => {
                self.suffixes
                    .get_or_insert_with(HashSet::new)
                    .insert(flag_arg);
            }
            Filter::Variable => {
                self.variables
                    .get_or_insert_with(HashSet::new)
                    .insert(flag_arg);
            }
        }

        return Ok(());
    }

    /// Determines if the given `var_name` matches any of the filters defined in the `Filters` struct.
    ///
    /// * `var_name`: The variable name to check against the filters.
    ///
    /// Returns an `Option<bool>` which is:
    /// * `None` if no filters are set.
    /// * `Some(true)` if the `var_name` matches any of the filters (prefixes, suffixes, or variables).
    /// * `Some(false)` if the `var_name` does not match any of the filters.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut filters = Filters::default();
    /// filters.add(Filter::Prefix, "--prefix", Some("TEST_"), &mut [].iter());
    /// filters.add(Filter::Suffix, "--suffix", Some("_CONFIG"), &mut [].iter());
    /// filters.add(Filter::Variable, "--variable", Some("SPECIAL_VAR"), &mut [].iter());
    ///
    /// assert_eq!(filters.matches("TEST_VAR"), Some(true));
    /// assert_eq!(filters.matches("VAR_CONFIG"), Some(true));
    /// assert_eq!(filters.matches("SPECIAL_VAR"), Some(true));
    /// assert_eq!(filters.matches("OTHER_VAR"), Some(false));
    /// ```
    #[must_use]
    pub fn matches(&self, var_name: &str) -> Option<bool> {
        // return None if no filters are set
        if !(self.prefixes.is_some() || self.suffixes.is_some() || self.variables.is_some()) {
            return None;
        }

        // check if the variable name matches the filters

        // Check if there's a prefix list in the `self.prefixes` field
        let match_prefix: bool = self
            .prefixes
            .as_ref() // Convert the Option<&Vec<String>> to an Option<&[String]>
            .map_or(false, |p| {
                // If there is a prefix list, iterate over it and check if any prefix
                // is found at the start of `var_name`. If any is found, return `true`.
                p.iter().any(|item| var_name.starts_with(item))
            }); // If there is no prefix list, return `false`.
                // Check if there's a suffix list in the `self.suffixes` field

        let match_suffix: bool = self
            .suffixes
            .as_ref() // Convert the Option<&Vec<String>> to an Option<&[String]>
            .map_or(false, |s| {
                // If there is a suffix list, iterate over it and check if any suffix
                // is found at the end of `var_name`. If any is found, return `true`.
                s.iter().any(|item| var_name.ends_with(item))
            }); // If there is no suffix list, return `false`.

        // Check if there's a variable list in the `self.variables` field
        let match_variable: bool = self
            .variables
            .as_ref() // Convert the Option<&HashSet<String>> to an Option<&HashSet<String>>
            .map_or(false, |v| {
                // If there is a variable list, check if it contains `var_name`.
                // If `var_name` is found, return `true`.
                v.contains(&var_name.to_string())
            }); // If there is no variable list, return `false`.

        return Some(match_prefix || match_suffix || match_variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_add_prefix() {
        let mut filters = Filters::default();
        assert!(filters.prefixes.is_none());

        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.prefixes,
            Some(vec!["test".to_string()].into_iter().collect())
        );

        filters
            .add(Filter::Prefix, "-p", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.prefixes,
            Some(
                vec!["test".to_string(), "hello".to_string()]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn test_filters_add_suffix() {
        let mut filters = Filters::default();
        assert!(filters.suffixes.is_none());

        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.suffixes,
            Some(vec!["test".to_string()].into_iter().collect())
        );

        filters
            .add(Filter::Suffix, "-s", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.suffixes,
            Some(
                vec!["test".to_string(), "hello".to_string()]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn test_filters_add_variable() {
        let mut filters = Filters::default();
        assert!(filters.variables.is_none());

        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.variables,
            Some(vec!["test".to_string()].into_iter().collect())
        );

        filters
            .add(Filter::Variable, "-v", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(
            filters.variables,
            Some(
                vec!["test".to_string(), "hello".to_string()]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn test_filters_add_missing_value() {
        let mut filters = Filters::default();
        let result = filters.add(Filter::Prefix, "--prefix", None, &mut [].iter());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--prefix".to_string())
        );
    }

    #[test]
    fn test_filters_add_start_parameter() {
        let mut filters = Filters::default();
        let result = filters.add(Filter::Prefix, "--help", None, &mut [].iter());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--help".to_string())
        );
    }

    #[test]
    fn test_add_start_parameters() {
        let mut filters = Filters::default();
        let args = vec!["--help".to_string(), "value".to_string()];
        let mut iter = args.iter();
        let result = filters.add(Filter::Prefix, "--help", None, &mut iter);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseArgsError::MissingValue("--help".to_string())
        );
    }

    #[test]
    fn test_no_filters() {
        let filters = Filters::default();
        assert_eq!(filters.matches("test_var"), None);
    }

    #[test]
    fn test_prefix_filter() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
        assert_eq!(filters.matches("var_test"), Some(false));
    }

    #[test]
    fn test_suffix_filter() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(false));
        assert_eq!(filters.matches("var_test"), Some(true));
    }

    #[test]
    fn test_variable_filter() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(false));
        assert_eq!(filters.matches("test"), Some(true));
    }

    #[test]
    fn test_multiple_prefix_filters() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Prefix, "-p", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
        assert_eq!(filters.matches("var_test"), Some(false));
        assert_eq!(filters.matches("hello_var"), Some(true));
        assert_eq!(filters.matches("var_hello"), Some(false));
    }

    #[test]
    fn test_multiple_suffix_filters() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "-s", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(false));
        assert_eq!(filters.matches("var_test"), Some(true));
        assert_eq!(filters.matches("hello_var"), Some(false));
        assert_eq!(filters.matches("var_hello"), Some(true));
    }

    #[test]
    fn test_multiple_variable_filters() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "-v", Some("hello"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test"), Some(true));
        assert_eq!(filters.matches("var_test"), Some(false));
        assert_eq!(filters.matches("hello"), Some(true));
        assert_eq!(filters.matches("hello_var"), Some(false));
    }

    #[test]
    fn test_multiple_filters() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
        assert_eq!(filters.matches("var_test"), Some(true));
        assert_eq!(filters.matches("test"), Some(true));
    }

    #[test]
    fn test_multiple_filters_with_no_match() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("hello_var"), Some(false));
    }

    #[test]
    fn test_multiple_filters_with_match() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
    }

    #[test]
    fn test_multiple_filters_with_match_and_no_match() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
        assert_eq!(filters.matches("hello_var"), Some(false));
    }

    #[test]
    fn test_multiple_filters_with_match_and_no_match_and_match() {
        let mut filters = Filters::default();
        filters
            .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
            .unwrap();
        filters
            .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
            .unwrap();
        assert_eq!(filters.matches("test_var"), Some(true));
        assert_eq!(filters.matches("hello_var"), Some(false));
        assert_eq!(filters.matches("test_var"), Some(true));
    }
}
