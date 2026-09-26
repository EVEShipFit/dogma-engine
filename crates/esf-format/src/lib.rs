//! Imports and exports EVE Online ship fits as an [`esf_dogma_engine::Fit`].
//!
//! - EFT, the text format EVE copies fits to; see [`eft::load_eft`] and
//!   [`eft::save_eft`].
//! - ESI fittings, the fits a character saves in game; see
//!   [`esi::load_esi_fitting`] and [`esi::save_esi_fitting`].
//!
//! Only EFT names its items; the other formats use type ids. English names
//! are found in `sde.dat`; add `names.dat` to also match names in the other
//! languages EVE supports when importing. An export is always English.
//!
//! ```no_run
//! use esf_data::{InfoNameSde, InfoSde, Sde};
//! use esf_format::eft::{load_eft, save_eft};
//!
//! let bytes = std::fs::read("sde.dat").unwrap();
//! let sde = Sde::new(&bytes).unwrap();
//! let info = InfoNameSde::new(&sde, None).unwrap();
//!
//! let fit = load_eft(&info, "[Rifter, My Rifter]\n200mm AutoCannon I").unwrap();
//! let eft = save_eft(&InfoSde::new(&sde), &fit).unwrap();
//! ```

#![warn(missing_docs)]

pub mod eft;
pub mod esi;

mod flags;
mod listed;
