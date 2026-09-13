use esf_dogma_engine::calculate;
use esf_dogma_engine::eft;
use esf_dogma_engine::rust;
use esf_dogma_engine::sde;

use super::skills::Skills;
use super::statistics::{dump, dump_items};
use super::{NAMES, SDE};

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
    let info_name = sde::InfoNameSde::new(&SDE, Some(&NAMES)).unwrap();
    let fit = eft::load_eft(&info_name, eft_fit.trim()).unwrap().esf_fit;

    let info = sde::InfoSde::new(fit, skills.levels, &SDE);
    let statistics = calculate::calculate(&info);

    format!(
        "{}\n{}",
        dump(&rust::Output::new(&info, &statistics)),
        dump_items(&info, &statistics)
    )
}
