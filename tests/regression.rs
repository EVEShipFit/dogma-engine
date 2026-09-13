//! Behaviour lock for the engine.
//!
//! Every case calculates a fit and compares the attributes against a stored
//! snapshot.
//!
//! Run `cargo insta review` to inspect and accept changed snapshots.

#![cfg(all(feature = "eft", feature = "rust"))]

#[macro_use]
mod harness;

mod fits;
