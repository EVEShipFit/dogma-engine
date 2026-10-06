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
    BuildMismatch {
        /// The build of the SDE.
        sde: i32,
        /// The build of the names file.
        names: i32,
    },
    /// The SDE is older than this dogma-engine needs.
    SdeTooOld {
        /// The major version of the SDE.
        found: i32,
        /// The lowest major version this dogma-engine reads.
        needed: i32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSde(_) => write!(f, "not a valid SDE file"),
            Error::InvalidNames(_) => write!(f, "not a valid names file"),
            Error::BuildMismatch { sde, names } => {
                write!(f, "SDE is build {sde} but the names are build {names}")
            }
            Error::SdeTooOld { found, needed } => {
                write!(
                    f,
                    "SDE is major version {found} but at least {needed} is needed"
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidSde(error) | Error::InvalidNames(error) => Some(error),
            Error::BuildMismatch { .. } | Error::SdeTooOld { .. } => None,
        }
    }
}
