use esf_data::{InfoNameSde, InfoSde};
use esf_dogma_engine::{Calculation, Fit, Options, Projection};
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

pub fn load(eft_fit: &str) -> Result<Fit, eft::Error> {
    let info_name = InfoNameSde::new(&SDE, Some(&NAMES)).unwrap();
    eft::load_eft(&info_name, eft_fit.trim())
}

/* EFT cannot express everything a fit can, so a case may edit the loaded fit. */
pub fn calculate(
    eft_fit: &str,
    skills: Skills,
    edit: fn(&mut Fit),
    options: &Options,
) -> (Fit, Calculation) {
    let mut fit = load(eft_fit).unwrap();
    fit.character.skills = skills.levels;
    edit(&mut fit);

    let info = InfoSde::new(&SDE);
    let calculation = esf_dogma_engine::calculate(&info, &fit, options);
    (fit, calculation)
}

/// What another EFT fit hands out, to put in the `incoming` of this one.
pub fn outgoing(eft_fit: &str, skills: Skills) -> Projection {
    let (_, calculation) = calculate(eft_fit, skills, |_| {}, &Options::default());
    calculation.outgoing
}

fn calculate_fit(eft_fit: &str, skills: Skills, edit: fn(&mut Fit)) -> String {
    let (fit, calculation) = calculate(eft_fit, skills, edit, &Options::default());
    dump(&InfoSde::new(&SDE), &fit, &calculation)
}
