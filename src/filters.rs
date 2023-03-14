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
    pub prefixes: Option<HashSet<String>>,
    pub suffixes: Option<HashSet<String>>,
    pub variables: Option<HashSet<String>>,
}
