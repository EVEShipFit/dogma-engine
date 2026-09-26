use esf_data::InfoSde;
use serde::Serialize;

use super::SDE;

const INPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/formats");
const SNAPSHOTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/snapshots");

pub fn info() -> InfoSde<'static> {
    InfoSde::new(&SDE)
}

/// A file in `formats/`, as it came from where the format lives.
pub fn input(file: &str) -> String {
    let filename = format!("{INPUT}/{file}");
    std::fs::read_to_string(&filename)
        .unwrap_or_else(|error| panic!("cannot read {filename}: {error}"))
}

/// Stores `value` as JSON, as a format loads or saves it.
pub fn snapshot_json(name: &str, value: &impl Serialize) {
    let json = serde_json::to_string_pretty(value).unwrap();
    insta::with_settings!({snapshot_path => SNAPSHOTS, prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(format!("formats-{name}"), json);
    });
}
