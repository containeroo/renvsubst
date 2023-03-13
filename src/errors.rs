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
            Self::UnknownFlag(flag) => return write!(f, "Unknown flag: {}", flag),
            Self::MissingValue(flag) => return write!(f, "Flag '{}' requires a value!", flag),
            Self::ConflictingFlags(flag1, flag2) => {
                return write!(f, "Flags {} and {} cannot be used together!", flag1, flag2)
            }
            Self::DuplicateValue(flag) => {
                return write!(f, "Flag '{}' cannot be specified more than once!", flag)
            }
        }
    }
}

impl std::error::Error for ParseArgsError {}
