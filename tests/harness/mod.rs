//! Machinery shared by the regression tests.
//!
//! Not every test uses every helper, hence the allows.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;
use std::sync::LazyLock;

use esf_data::{InfoName, InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::Projection;

mod case;
mod dump;
mod skills;

pub use case::{calculate, load, outgoing, snapshot};
pub use skills::{Skills, all, none};

/// What a beacon hands out, by name.
pub fn beacon(name: &str) -> Projection {
    let type_id = InfoNameSde::new(&SDE, Some(&NAMES))
        .unwrap()
        .type_name_to_id(name)
        .unwrap_or_else(|| panic!("no such beacon: {name}"));

    esf_dogma_engine::beacon(&InfoSde::new(&SDE), type_id)
}

/// Override with ESF_SDE and ESF_NAMES when the flatbuffers live elsewhere.
fn read(variable: &str, default: &str) -> Vec<u8> {
    let filename = std::env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default));
    std::fs::read(&filename)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", filename.display(), error))
}

static SDE_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    read(
        "ESF_SDE",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/sde.dat"
        ),
    )
});
static NAMES_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    read(
        "ESF_NAMES",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/names.dat"
        ),
    )
});

static SDE: LazyLock<Sde<'static>> = LazyLock::new(|| Sde::new(&SDE_BYTES).unwrap());
static NAMES: LazyLock<Names<'static>> = LazyLock::new(|| Names::new(&NAMES_BYTES).unwrap());

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
