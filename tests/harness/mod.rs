//! Machinery shared by the regression tests.
//!
//! Not every test uses every helper, hence the allows.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::LazyLock;

use esf_dogma_engine::sde;

mod case;
mod skills;
mod statistics;

pub use case::snapshot;
pub use skills::{Skills, all, none};

/// Override with ESF_SDE and ESF_NAMES when the flatbuffers live elsewhere.
fn read(variable: &str, default: &str) -> Vec<u8> {
    let filename = std::env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default));
    std::fs::read(&filename)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", filename.display(), error))
}

static SDE_BYTES: LazyLock<Vec<u8>> =
    LazyLock::new(|| read("ESF_SDE", "node_modules/@eveshipfit/sde/dist/sde.dat"));
static NAMES_BYTES: LazyLock<Vec<u8>> =
    LazyLock::new(|| read("ESF_NAMES", "node_modules/@eveshipfit/sde/dist/names.dat"));

static SDE: LazyLock<sde::Sde<'static>> = LazyLock::new(|| sde::Sde::new(&SDE_BYTES).unwrap());
static NAMES: LazyLock<sde::Names<'static>> =
    LazyLock::new(|| sde::Names::new(&NAMES_BYTES).unwrap());

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
