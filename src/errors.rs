/// An error that occurs while parsing command-line arguments.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseArgsError {
    /// An unknown flag was specified.
    UnknownFlag(String),

    /// A value is missing for a given flag.
    MissingValue(String),

    /// Two or more conflicting flags were specified.
    ConflictingFlags(String, String),

    /// Flag were specified multiple times.
    DuplicateFlag(String),
}

impl std::fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFlag(flag) => return write!(f, "Unknown flag: {flag}"),
            Self::MissingValue(flag) => return write!(f, "Flag '{flag}' requires a value!"),
            Self::ConflictingFlags(flag1, flag2) => {
                return write!(f, "Flags {flag1} and {flag2} cannot be used together!")
            }
            Self::DuplicateFlag(flag) => {
                return write!(f, "Flag '{flag}' cannot be specified more than once!")
            }
        }
    }
}

impl std::error::Error for ParseArgsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_flag_error() {
        let error = ParseArgsError::UnknownFlag(String::from("foo"));
        assert_eq!(format!("{error}"), "Unknown flag: foo");
    }

    #[test]
    fn test_missing_value_error() {
        let error = ParseArgsError::MissingValue(String::from("foo"));
        assert_eq!(format!("{error}"), "Flag 'foo' requires a value!");
    }

    #[test]
    fn test_conflicting_flags_error() {
        let error = ParseArgsError::ConflictingFlags(String::from("foo"), String::from("bar"));
        assert_eq!(
            format!("{error}"),
            "Flags foo and bar cannot be used together!"
        );
    }

    #[test]
    fn test_duplicate_value_error() {
        let error = ParseArgsError::DuplicateFlag(String::from("foo"));
        assert_eq!(
            format!("{error}"),
            "Flag 'foo' cannot be specified more than once!"
        );
    }
}
