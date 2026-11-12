//! Machinery shared by the regression tests.
//!
//! Not every test uses every helper, hence the allows.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::LazyLock;

use esf_data::{InfoName, InfoNameSde, Names, Sde};
use esf_dogma_engine::Projection;

mod case;
mod dump;
mod formats;
mod skills;

pub use case::{calculate, load, load_eft, outgoing, save_eft, snapshot, snapshot_violations};
pub use formats::{info, info_name, input, snapshot_json};
pub use skills::{Skills, all, none};

/// What a beacon hands out, by name.
pub fn beacon(name: &str) -> Projection {
    let type_id = InfoNameSde::new(&SDE, Some(&NAMES))
        .unwrap()
        .type_name_to_id(name)
        .unwrap_or_else(|| panic!("no such beacon: {name}"));

    esf_dogma_engine::beacon(&*SDE, type_id)
}

/// Override with ESF_SDE and ESF_NAMES when the flatbuffers live elsewhere.
fn read(variable: &str, default: &str) -> Vec<u8> {
    let filename = std::env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default));
    std::fs::read(&filename)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", filename.display(), error))
}

static SDE: LazyLock<Sde> = LazyLock::new(|| {
    Sde::new(read(
        "ESF_SDE",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/sde.dat"
        ),
    ))
    .unwrap()
});
static NAMES: LazyLock<Names> = LazyLock::new(|| {
    Names::new(read(
        "ESF_NAMES",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/names.dat"
        ),
    ))
    .unwrap()
});

macro_rules! validation {
    ($($name:ident = $fit:ident, skills: $skills:expr $(, edit: $edit:expr)?;)*) => {
        $(
            #[test]
            fn $name() {
                #[allow(unused_variables)]
                let edit: fn(&mut esf_dogma_engine::Fit) = |_| {};
                $(let edit: fn(&mut esf_dogma_engine::Fit) = $edit;)?
                crate::harness::snapshot_violations(module_path!(), stringify!($name), $fit, $skills, edit);
            }
        )*
    };
}

macro_rules! regression {
    ($($name:ident = $fit:ident, skills: $skills:expr $(, edit: $edit:expr)?;)*) => {
        $(
            #[test]
            fn $name() {
                #[allow(unused_variables)]
                let edit: fn(&mut esf_dogma_engine::Fit) = |_| {};
                $(let edit: fn(&mut esf_dogma_engine::Fit) = $edit;)?
                crate::harness::snapshot(module_path!(), stringify!($name), $fit, $skills, edit);
            }
        )*
    };
}
