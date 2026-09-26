//! Behaviour lock for the engine.
//!
//! Every case calculates a fit and compares the attributes against a stored
//! snapshot. The formats are locked down the same way, by what they load and
//! save.
//!
//! Run `cargo insta review` to inspect and accept changed snapshots.

#![cfg(test)]

#[macro_use]
mod harness;

mod eft;
mod esi;
mod fits;
mod killmail;
mod link;
