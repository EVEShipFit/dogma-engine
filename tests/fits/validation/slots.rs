//! Where an item may sit, and how many may sit there.

use crate::harness::all;
use esf_dogma_engine::Slot;

const HIGH_SLOTS: &str = r#"%esf/1
Rifter "High slots"

4x 200mm AutoCannon II
"#;

const ONE_MODULE: &str = r#"%esf/1
Rifter "One module"

Damage Control II
"#;

const IN_CARGO: &str = r#"%esf/1
Rifter "In cargo"

Damage Control II @cargo
"#;

const TWO_MODULES: &str = r#"%esf/1
Rifter "Two modules"

Damage Control II
Gyrostabilizer II
"#;

validation! {
    too_many_high_slots = HIGH_SLOTS, skills: all(5);
    wrong_rack = ONE_MODULE, skills: all(5), edit: |fit| fit.items[0].slot = Slot::High(0);
    slot_taken = TWO_MODULES, skills: all(5), edit: |fit| fit.items[1].slot = fit.items[0].slot;
    module_in_cargo = IN_CARGO, skills: all(5);
}
