//! Only a module is calculated in a slot of the ship, and only an implant or
//! booster in a slot of the character; anything else sits there like cargo.

use esf_dogma_engine::{Fit, Options, Slot};

use crate::harness::{all, calculate};

const FIT: &str = r#"%esf/1
Rifter "Ship in a slot"

Torrent @cargo
"#;

fn assert_not_calculated(edit: fn(&mut Fit)) {
    let (_, calculation) = calculate(FIT, all(5), edit, &Options::default());

    assert!(calculation.items[0].attributes.is_empty());
}

#[test]
fn in_medium_slot() {
    assert_not_calculated(|fit| fit.items[0].slot = Slot::Medium(0));
}

#[test]
fn in_implant_slot() {
    assert_not_calculated(|fit| fit.items[0].slot = Slot::Implant(1));
}

#[test]
fn in_booster_slot() {
    assert_not_calculated(|fit| fit.items[0].slot = Slot::Booster(1));
}
