//! Machinery shared by the regression tests.
//!
//! Not every test uses every helper, hence the allows.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::LazyLock;

use esf_dogma_engine::rust;

mod case;
mod skills;
mod statistics;

pub use case::snapshot;
pub use skills::{Skills, all, none};

static DATA: LazyLock<rust::Data> =
    LazyLock::new(|| rust::Data::new(&PathBuf::from("node_modules/@eveshipfit/data/dist/sde")));

macro_rules! regression {
    ($($name:ident = $fit:ident, skills: $skills:expr;)*) => {
        $(
            #[test]
            fn $name() {
                crate::harness::snapshot(module_path!(), stringify!($name), $fit, $skills);
            }
        )*
    };
}
