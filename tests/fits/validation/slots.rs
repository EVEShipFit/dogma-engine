//! Where an item may sit, and how many may sit there.

use crate::harness::all;
use esf_dogma_engine::Slot;

const HIGH_SLOTS: &str = r#"
[Rifter, High slots]
200mm AutoCannon II
200mm AutoCannon II
200mm AutoCannon II
200mm AutoCannon II
"#;

const ONE_MODULE: &str = r#"
[Rifter, One module]
Damage Control II
"#;

const TWO_MODULES: &str = r#"
[Rifter, Two modules]
Damage Control II
Gyrostabilizer II
"#;

validation! {
    too_many_high_slots = HIGH_SLOTS, skills: all(5);
    wrong_rack = ONE_MODULE, skills: all(5), edit: |fit| fit.items[0].slot = Slot::High(0);
    slot_taken = TWO_MODULES, skills: all(5), edit: |fit| fit.items[1].slot = fit.items[0].slot;
}
