use esf_data::{InfoNameSde, InfoSde};
use esf_dogma_engine::{Calculation, Fit, Options, Projection};
use esf_format::eft;

use super::dump::{dump, dump_violations};
use super::skills::Skills;
use super::{NAMES, SDE};

const SNAPSHOTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/snapshots");

pub fn snapshot(module_path: &str, name: &str, eft_fit: &str, skills: Skills, edit: fn(&mut Fit)) {
    assert_valid(eft_fit, edit);

    insta::with_settings!({snapshot_path => SNAPSHOTS, prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(case(module_path, name), calculate_fit(eft_fit, skills, edit));
    });
}

/// A regression fit has to be one EVE would let you fly, so that no case
/// quietly rests on a fit that could not exist. Skills belong to the character
/// rather than to the fit, so this judges it with everything trained; a case
/// is free to calculate the same fit with fewer.
fn assert_valid(eft_fit: &str, edit: fn(&mut Fit)) {
    let (fit, calculation) = calculate(eft_fit, super::all(5), edit, &Options::default());

    let info = InfoSde::new(&SDE);
    let violations = esf_dogma_engine::validate(&info, &fit, &calculation);
    assert!(
        violations.is_empty(),
        "this fit breaks a rule, so EVE would not let you fly it:\n{}",
        dump_violations(&info, &fit, &violations)
    );
}

/// Like [`snapshot`], but stores the rules the fit breaks instead of its
/// attributes.
pub fn snapshot_violations(
    module_path: &str,
    name: &str,
    eft_fit: &str,
    skills: Skills,
    edit: fn(&mut Fit),
) {
    insta::with_settings!({snapshot_path => SNAPSHOTS, prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!(case(module_path, name), validate_fit(eft_fit, skills, edit));
    });
}

/* Everything below `fits`, so `regression::fits::community::gila` names the
 * snapshot `community-gila-<case>`. */
fn case(module_path: &str, name: &str) -> String {
    module_path
        .split("::")
        .skip_while(|segment| *segment != "fits")
        .skip(1)
        .chain([name])
        .collect::<Vec<&str>>()
        .join("-")
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

    let info = InfoSde::new(&SDE);
    let violations = esf_dogma_engine::validate(&info, &fit, &calculation);
    dump(&info, &fit, &calculation, &violations)
}

fn validate_fit(eft_fit: &str, skills: Skills, edit: fn(&mut Fit)) -> String {
    let (fit, calculation) = calculate(eft_fit, skills, edit, &Options::default());

    let info = InfoSde::new(&SDE);
    let violations = esf_dogma_engine::validate(&info, &fit, &calculation);
    dump_violations(&info, &fit, &violations)
}
