//! Reads the EVE Online static data (SDE) the dogma engine calculates with.
//!
//! The data comes as flatbuffers built by
//! [sde-patched](https://github.com/EVEShipFit/sde-patched). Load `sde.dat` into
//! [`Sde`], and hand it to the engine through [`InfoSde`].

#![warn(missing_docs)]

mod error;
mod fold;
mod info;
mod sde;

pub use error::Error;
pub use fold::{fold_case, fold_char, sort_by_text};
pub use info::{Info, InfoEsf, InfoExport, InfoName};
pub use sde::{InfoNameSde, InfoSde, MIN_SDE_VERSION, Names, Sde, eve};

/// The traits hand out flatbuffers types; implementing them needs this exact
/// version.
pub use flatbuffers;
