//! Calculates the dogma attributes of an EVE Online ship fit: the ship, its
//! items and the character.
//!
//! Describe the fit with [`Fit`] and pass it to [`calculate()`], together
//! with the static data from `esf-data`.
//!
//! ```no_run
//! use esf_data::{InfoSde, Sde};
//! use esf_dogma_engine::{Fit, Options, calculate};
//!
//! let bytes = std::fs::read("sde.dat").unwrap();
//! let sde = Sde::new(&bytes).unwrap();
//!
//! let fit: Fit = serde_json::from_str(
//!     r#"{
//!         "ship": {"type_id": 587},
//!         "items": [{"type_id": 2873, "slot": {"type": "high", "index": 0}, "state": "active"}]
//!     }"#,
//! )
//! .unwrap();
//!
//! let calculation = calculate(&InfoSde::new(&sde), &fit, &Options::default());
//! ```

#![warn(missing_docs)]

mod calculate;
mod fit;

pub use calculate::{
    AttributeValue, Calculation, EffectOperator, ItemResult, Options, Source, SourceRef, calculate,
};
pub use fit::{Character, Charge, Fit, FitItem, Mutation, Ship, Slot, State};
