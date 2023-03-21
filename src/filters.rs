use crate::errors::ParseArgsError;
use crate::utils::START_PARAMETERS;
use std::collections::HashSet;

/// A struct that contains optional filters for matching strings.
///
/// The `prefixes` field is a set of string prefixes. When matching a string, the `starts_with` method is used to check if the string starts with any of the prefixes in the set. If multiple identical prefixes are added to the set, only one copy of each prefix will be stored.
///
/// The `suffixes` field is a set of string suffixes. When matching a string, the `ends_with` method is used to check if the string ends with any of the suffixes in the set. If multiple identical suffixes are added to the set, only one copy of each suffix will be stored.
///
/// The `variables` field is a set of string variables. When matching a string, the `contains` method is used to check if the string contains any of the variables in the set. If multiple identical variables are added to the set, only one copy of each variable will be stored.
#[derive(Debug, Default)]
pub struct Filters {
    /// A set of string prefixes.
    pub prefixes: Option<HashSet<String>>,
    /// A set of string suffixes.
    pub suffixes: Option<HashSet<String>>,
    /// A set of string variables.
    pub variables: Option<HashSet<String>>,
}

/// An enum that represents the type of filter to be added to the `Filters` struct.
///
/// The `Prefix` variant indicates that the filter is a string prefix. When matching a string, the `starts_with` method is used to check if the string starts with any of the prefixes in the set. If multiple identical prefixes are added to the set, only one copy of each prefix will be stored.
///
/// The `Suffix` variant indicates that the filter is a string suffix. When matching a string, the `ends_with` method is used to check if the string ends with any of the suffixes in the set. If multiple identical suffixes are added to the set, only one copy of each suffix will be stored.
///
/// The `Variable` variant indicates that the filter is a string variable. When matching a string, the `contains` method is used to check if the string contains any of the variables in the set. If multiple identical variables are added to the set, only one copy of each variable will be stored.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Filter {
    Prefix,
    Suffix,
    Variable,
}

impl Filters {
    /// Adds a new filter criterion to the `Filters` struct.
    ///
    /// This function adds a new filter criterion (prefix, suffix, or specific variable name)
    /// to the `Filters` struct. If an invalid filter value is provided, a `ParseArgsError` is returned.
    ///
    /// # Arguments
    ///
    /// * `filter` - A `Filter` enum value representing the type of filter (prefix, suffix, or variable).
    /// * `arg` - A string slice that holds the argument name associated with the filter (e.g., "--prefix").
    /// * `value` - An `Option<&str>` containing the filter value. If `None`, the value will be extracted from the `iter`.
    /// * `iter` - A mutable iterator over a slice of strings, used to extract the filter value if it is not provided in `value`.
    ///
    /// # Errors
    ///
    /// Returns a `ParseArgsError` if the filter value is missing or if it matches a reserved start parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use filters::{Filter, Filters};
    ///
    /// let mut filters = Filters::default();
    /// let mut args: Vec<String> = vec!["--prefix".to_string(), "prefix_".to_string()];
    /// filters.add(Filter::Prefix, "--prefix", None, &mut args.iter()).unwrap();
    ///
    /// assert_eq!(filters.matches("prefix_test_var"), Some(true));
    /// ```
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

    /// Determines if a given variable name matches the specified filter criteria.
    ///
    /// This function checks if the given `var_name` matches any of the filters
    /// set in the `Filters` struct (i.e., prefixes, suffixes, and specific variable names).
    /// If there are no filters set, it returns `None`. Otherwise, it returns `Some(bool)`,
    /// where the boolean value indicates whether the variable name matches any filter.
    ///
    /// # Arguments
    ///
    /// * `var_name` - A string slice that holds the variable name to be tested against the filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use filters::{Filter, Filters};
    ///
    /// let mut filters = Filters::default();
    /// filters
    ///     .add(Filter::Prefix, "--prefix", Some("test"), &mut [].iter())
    ///     .unwrap();
    /// filters
    ///     .add(Filter::Suffix, "--suffix", Some("test"), &mut [].iter())
    ///     .unwrap();
    /// filters
    ///     .add(Filter::Variable, "--variable", Some("test"), &mut [].iter())
    ///     .unwrap();
    /// assert_eq!(filters.matches("test_var"), Some(true));
    /// assert_eq!(filters.matches("var_test"), Some(true));
    /// assert_eq!(filters.matches("test"), Some(true));
    /// ```
    ///
    /// # Returns
    ///
    /// An `Option<bool>` value:
    ///
    /// * `None` if there are no filters set
    /// * `Some(true)` if the given variable name matches any filter
    /// * `Some(false)` if the given variable name does not match any filter
    pub fn matches(&self, var_name: &str) -> Option<bool> {
        // return None if no filters are set
        if !(self.prefixes.is_some() || self.suffixes.is_some() || self.variables.is_some()) {
            return None;
        }

        // check if the variable name matches the filters
        let match_prefix: bool = self
            .prefixes
            .as_ref()
            .map_or(false, |p| p.iter().any(|item| var_name.starts_with(item)));
        let match_suffix: bool = self
            .suffixes
            .as_ref()
            .map_or(false, |s| s.iter().any(|item| var_name.ends_with(item)));
        let match_variable: bool = self
            .variables
            .as_ref()
            .map_or(false, |v| v.contains(&var_name.to_string()));

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
