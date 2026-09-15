//! Validate that structure attributes are applied correctly throughout.

use crate::harness::all;

const DEFENSE: &str = r#"
[Astrahus, Structure stats defense]
Standup Layered Armor Plating I
Standup Ballistic Control System I

Standup Cap Battery I
Standup Variable Spectrum ECM I

Standup Heavy Energy Neutralizer I

Standup M-Set Missile Precision I

Standup Cloning Center I
"#;

/* The launchers hold a charge and reload; the neutralizer holds none. A
 * guided bomb launcher and market hub need at least a Fortizar. */
const ARMED: &str = r#"
[Fortizar, Structure stats armed]


Standup Multirole Missile Launcher I, Standup Cruise Missile
Standup Guided Bomb Launcher I, Standup Light Guided Bomb
Standup Heavy Energy Neutralizer I


Standup Market Hub I
"#;

regression! {
    defense_skills_5 = DEFENSE, skills: all(5);
    armed_skills_5 = ARMED, skills: all(5);
}
