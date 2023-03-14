/// An error that occurs while parsing command-line arguments.
#[derive(Debug, PartialEq)]
pub enum ParseArgsError {
    /// An unknown flag was specified.
    UnknownFlag(String),

    /// A value is missing for a given flag.
    MissingValue(String),

    /// Two or more conflicting flags were specified.
    ConflictingFlags(String, String),

    /// Duplicate values were specified for a given flag.
    DuplicateValue(String),
}

impl std::fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFlag(flag) => return write!(f, "Unknown flag: {flag}"),
            Self::MissingValue(flag) => return write!(f, "Flag '{flag}' requires a value!"),
            Self::ConflictingFlags(flag1, flag2) => {
                return write!(f, "Flags {flag1} and {flag2} cannot be used together!")
            }
            Self::DuplicateValue(flag) => {
                return write!(f, "Flag '{flag}' cannot be specified more than once!")
            }
        }
    }
}

impl std::error::Error for ParseArgsError {}
