//! Readers for the flatbuffers produced by `sde-patched`.
//!
//! The data is split in two. `sde.dat` holds everything needed to calculate a
//! fit; `names.dat` only exists to turn a name from an EFT-fit back into a
//! type, and can be left out by anyone who never imports one.

#[rustfmt::skip]
#[allow(warnings, clippy::all, clippy::pedantic)]
mod eve_generated;

#[rustfmt::skip]
#[allow(warnings, clippy::all, clippy::pedantic)]
mod names_generated;

mod data;
mod info;
mod names;

pub use data::Sde;
pub use info::{InfoNameSde, InfoSde};
pub use names::Names;

pub use eve_generated::eve;
