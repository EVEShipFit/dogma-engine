//! Reads the EVE Online static data (SDE) the dogma engine calculates with.
//!
//! The data comes as flatbuffers built by
//! [sde-patched](https://github.com/EVEShipFit/sde-patched). Load `sde.dat` into
//! [`Sde`], and hand it to the engine through [`InfoSde`].

#![warn(missing_docs)]

mod error;
mod info;
mod sde;

pub use error::Error;
pub use info::{Info, InfoExport, InfoName};
pub use sde::{InfoNameSde, InfoSde, Names, Sde, eve};

/// The traits hand out flatbuffers types; implementing them needs this exact
/// version.
pub use flatbuffers;
