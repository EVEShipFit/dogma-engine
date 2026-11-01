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

const SHIP_IN_CARGO: &str = r#"%esf/1
Rifter "Ship in cargo"

Torrent @cargo
"#;

const LASER: &str = r#"%esf/1
Thanatos "Laser"

Dual Heavy Beam Laser I
"#;

validation! {
    too_many_high_slots = HIGH_SLOTS, skills: all(5);
    wrong_rack = ONE_MODULE, skills: all(5), edit: |fit| fit.items[0].slot = Slot::High(0);
    slot_taken = TWO_MODULES, skills: all(5), edit: |fit| fit.items[1].slot = fit.items[0].slot;
    module_in_cargo = IN_CARGO, skills: all(5);
    ship_in_medium_slot = SHIP_IN_CARGO, skills: all(5), edit: |fit| fit.items[0].slot = Slot::Medium(0);
    ship_in_implant_slot = SHIP_IN_CARGO, skills: all(5), edit: |fit| fit.items[0].slot = Slot::Implant(1);
    ship_in_booster_slot = SHIP_IN_CARGO, skills: all(5), edit: |fit| fit.items[0].slot = Slot::Booster(1);
    module_in_fighter_tube = LASER, skills: all(5), edit: |fit| fit.items[0].slot = Slot::FighterTube(0);
    module_in_drone_bay = ONE_MODULE, skills: all(5), edit: |fit| fit.items[0].slot = Slot::DroneBay;
    module_in_fighter_bay = LASER, skills: all(5), edit: |fit| fit.items[0].slot = Slot::FighterBay;
}
