//! Calculates the dogma attributes of an EVE Online ship fit: the ship, its
//! items and the character.
//!
//! Describe the fit with [`fit::Fit`] and pass it to [`calculate()`], together
//! with the static data from `esf-data`.
//!
//! ```no_run
//! use esf_data::sde::{InfoSde, Sde};
//! use esf_dogma::fit::Fit;
//! use esf_dogma::{Options, calculate};
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

mod calculate;
pub mod fit;

pub use calculate::{
    AttributeValue, Calculation, EffectOperator, ItemResult, Options, Source, SourceRef, calculate,
};
