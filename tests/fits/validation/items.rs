//! What the ship itself refuses to carry.

use crate::harness::all;

const RIG_SIZE: &str = r#"%esf/1
Rifter "Rig size"

Medium Ancillary Current Router I
"#;

const CAPITAL: &str = r#"%esf/1
Rifter "Capital"

Capital Shield Booster II
"#;

const RESTRICTED: &str = r#"%esf/1
Rifter "Restricted"

Bomb Launcher II
"#;

const TWO_CONTROLS: &str = r#"%esf/1
Rifter "Two controls"

2x Damage Control II
"#;

/* An afterburner and a microwarpdrive share a group, so the limit on how many
 * of a group run at once is what keeps one of the two off. */
const PROP_MODS: &str = r#"%esf/1
Rifter "Prop mods"

5MN Microwarpdrive II
1MN Afterburner II
"#;

/* A structure takes modules as large as a capital's, and is not a ship. */
const STRUCTURE: &str = r#"%esf/1
Astrahus "Structure"

Standup Heavy Energy Neutralizer I

Standup Cap Battery I

Standup Layered Armor Plating I

Standup M-Set Missile Precision I

Standup Cloning Center I
"#;

const SHIP_MODULE_ON_STRUCTURE: &str = r#"%esf/1
Astrahus "Ship module"

Damage Control II
"#;

const STRUCTURE_MODULE_ON_SHIP: &str = r#"%esf/1
Rifter "Structure module"

Standup Cap Battery I
"#;

validation! {
    rig_size = RIG_SIZE, skills: all(5);
    capital_item = CAPITAL, skills: all(5);
    ship_restricted = RESTRICTED, skills: all(5);
    two_of_one_group_fitted = TWO_CONTROLS, skills: all(5);
    two_prop_mods_active = PROP_MODS, skills: all(5);
    structure = STRUCTURE, skills: all(5);
    ship_module_on_structure = SHIP_MODULE_ON_STRUCTURE, skills: all(5);
    structure_module_on_ship = STRUCTURE_MODULE_ON_SHIP, skills: all(5);
}
