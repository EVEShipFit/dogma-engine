use esf_data::sde;
use esf_dogma::fit::Fit;
use esf_format::eft;

use super::dump::dump;
use super::skills::Skills;
use super::{NAMES, SDE};

const SNAPSHOTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/snapshots");

pub fn snapshot(module_path: &str, name: &str, eft_fit: &str, skills: Skills, edit: fn(&mut Fit)) {
    /* Everything below `fits`, so `regression::fits::community::gila` names
     * the snapshot `community-gila-<case>`. */
    let case: Vec<&str> = module_path
        .split("::")
        .skip_while(|segment| *segment != "fits")
        .skip(1)
        .chain([name])
        .collect();

    insta::with_settings!({snapshot_path => SNAPSHOTS, prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(case.join("-"), calculate_fit(eft_fit, skills, edit));
    });
}

/* EFT cannot express everything a fit can, so a case may edit the loaded fit. */
fn calculate_fit(eft_fit: &str, skills: Skills, edit: fn(&mut Fit)) -> String {
    let info_name = sde::InfoNameSde::new(&SDE, Some(&NAMES)).unwrap();
    let mut fit = eft::load_eft(&info_name, eft_fit.trim()).unwrap();
    fit.character.skills = skills.levels;
    edit(&mut fit);

    let info = sde::InfoSde::new(&SDE);
    let calculation = esf_dogma::calculate(&info, &fit);

    dump(&info, &fit, &calculation)
}
