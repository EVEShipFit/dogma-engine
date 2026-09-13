//! Imports EVE Online ship fits into an [`esf_dogma_engine::Fit`].
//!
//! Only EFT, the text format EVE copies fits to, is supported; see
//! [`eft::load_eft`]. English names are found in `sde.dat`; add `names.dat` to
//! also match names in the other languages EVE supports.
//!
//! ```no_run
//! use esf_data::{InfoNameSde, Sde};
//! use esf_format::eft::load_eft;
//!
//! let bytes = std::fs::read("sde.dat").unwrap();
//! let sde = Sde::new(&bytes).unwrap();
//! let info = InfoNameSde::new(&sde, None).unwrap();
//!
//! let fit = load_eft(&info, "[Rifter, My Rifter]\n200mm AutoCannon I").unwrap();
//! ```

#![warn(missing_docs)]

pub mod eft;
