use crate::errors::ParseArgsError;
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

/// List with all the parameters that can be used to start the program.
/// This is used to check if the value of a flag is another flag.
const START_PARAMETERS: &[&str] = &[
    "-h",
    "--help",
    "--version",
    "--fail-on-unset",
    "--fail-on-empty",
    "--fail",
    "--no-replace-unset",
    "--no-replace-empty",
    "--no-escape",
    "--unbuffer-lines",
    "-p",
    "--prefix",
    "-s",
    "--suffix",
    "-v",
    "--variable",
];

impl Filters {
    /// Adds a filter to the `Filters` struct.
    ///
    /// # Arguments
    ///
    /// * `filter` - An enum that represents the type of filter to be added to the `Filters` struct.
    ///
    /// * `arg` - A string slice that represents the filter argument to be added.
    ///
    /// * `value` - An optional string slice that represents the value of the filter argument.
    ///
    /// * `iter` - A mutable slice iterator of strings that represents the remaining arguments to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<(), ParseArgsError>` - A result that returns `Ok(())` if the filter was added successfully, or a `ParseArgsError` if an error occurred.
    pub fn add(
        &mut self,
        filter: Filter,
        arg: &str,
        value: Option<&str>,
        iter: &mut std::slice::Iter<String>,
    ) -> Result<(), ParseArgsError> {
        let flag_arg: String = value
            .map_or_else(  // if no value is provided... (was not --prefix=prefix_)
                || {
                    // if not, get the next argument as the value
                    iter.next()
                        .map(std::string::ToString::to_string) // convert the value to a string
                        .ok_or_else(|| ParseArgsError::MissingValue(arg.to_string())) // return an error if the value is missing
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
}
