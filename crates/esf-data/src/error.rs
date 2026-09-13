use std::fmt;

use flatbuffers::InvalidFlatbuffer;

/// Why the static data could not be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The bytes handed to [`Sde::new`](crate::Sde::new) are not an SDE.
    InvalidSde(InvalidFlatbuffer),
    /// The bytes handed to [`Names::new`](crate::Names::new) are not a
    /// names file.
    InvalidNames(InvalidFlatbuffer),
    /// The SDE and the names file come from different builds.
    BuildMismatch { sde: i32, names: i32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSde(_) => write!(f, "not a valid SDE file"),
            Error::InvalidNames(_) => write!(f, "not a valid names file"),
            Error::BuildMismatch { sde, names } => {
                write!(f, "SDE is build {sde} but the names are build {names}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidSde(error) | Error::InvalidNames(error) => Some(error),
            Error::BuildMismatch { .. } => None,
        }
    }
}
