//! A command line utility to substitute (bash-like) variables in the format of `$VAR_NAME`, `${VAR_NAME}` or `${VAR_NAME:-DEFAULT_VALUE}` with their corresponding values from the environment or the default value if provided.
//! A valid variable name is a string that starts with a letter or an underscore, followed by any number of letters, numbers, or underscores.
//!
//! For more information on how to use the `renvsubst` library and command-line utility, refer to the documentation for each module and the associated examples.

pub mod args;
pub mod env_subst;
pub mod errors;
pub mod filters;
pub mod flags;
pub mod help;
pub mod io;
pub mod utils;

pub use crate::args::Args;
pub use crate::env_subst::process_input;
pub use crate::flags::Flag;
pub use crate::io::{open_input, open_output, IO};
pub use crate::utils::print_error;
