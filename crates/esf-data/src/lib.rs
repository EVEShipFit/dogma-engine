//! Reads the EVE Online static data (SDE) the dogma engine calculates with.
//!
//! The data comes as flatbuffers built by
//! [sde-patched](https://github.com/EVEShipFit/sde-patched). Load `sde.dat` into
//! [`sde::Sde`], and hand it to the engine through [`sde::InfoSde`].

pub mod info;
pub mod sde;
