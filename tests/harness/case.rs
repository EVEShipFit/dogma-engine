use esf_dogma_engine::calculate;
use esf_dogma_engine::eft;
use esf_dogma_engine::rust;

use super::DATA;
use super::skills::Skills;
use super::statistics::dump;

const SNAPSHOTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots");

pub fn snapshot(module_path: &str, name: &str, eft_fit: &str, skills: Skills) {
    /* Everything below `fits`, so `regression::fits::community::gila` names
     * the snapshot `community-gila-<case>`. */
    let case: Vec<&str> = module_path
        .split("::")
        .skip_while(|segment| *segment != "fits")
        .skip(1)
        .chain([name])
        .collect();

    insta::with_settings!({snapshot_path => SNAPSHOTS, prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(case.join("-"), calculate_fit(eft_fit, skills));
    });
}

fn calculate_fit(eft_fit: &str, skills: Skills) -> String {
    let fit = eft::load_eft(&rust::InfoNameMain::new(&DATA), &eft_fit.trim().to_owned())
        .unwrap()
        .esf_fit;

    let info = rust::InfoMain::new(fit, skills.levels, &DATA);
    let statistics = calculate::calculate(&info);

    dump(&rust::Output::new(&info, &statistics))
}
