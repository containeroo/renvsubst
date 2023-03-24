//! A command line utility to substitute (bash-like) variables in the format of `$VAR_NAME`, `${VAR_NAME}` or `${VAR_NAME:-DEFAULT_VALUE}` with their corresponding values from the environment or the default value if provided.
//! A valid variable name is a string that starts with a letter or an underscore, followed by any number of letters, numbers, or underscores.
//!
//! # Example Usage
//!
//! ```rust
//! use renvsubst::{open_input, open_output, print_error, process_input, Args, Flag, IO};
//!
//! fn main() {
//!   let args = std::env::args()
//!       .skip(1) // skip(1) to skip the program name
//!       .collect::<Vec<String>>();
//!
//!   run(&args).unwrap_or_else(|err| {
//!       print_error(&err);
//!       std::process::exit(1);
//!   });
//! }
//! ```
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
