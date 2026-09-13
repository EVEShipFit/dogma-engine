//! Every structure bonus in the SDE targets the structure domain, which is the
//! hull. An online service module puts the structure at full power, raising
//! its hitpoints and enabling armor plating; the cap battery raises capacitor.

use crate::harness::all;

const POWERED: &str = r#"
[Astrahus, Structure bonus powered]
Standup Layered Armor Plating I

Standup Cap Battery I



Standup Cloning Center I
"#;

/* Without a service module the plating does nothing. */
const UNPOWERED: &str = r#"
[Astrahus, Structure bonus unpowered]
Standup Layered Armor Plating I

Standup Cap Battery I
"#;

regression! {
    powered_skills_0 = POWERED, skills: all(0);
    powered_skills_5 = POWERED, skills: all(5);
    unpowered_skills_5 = UNPOWERED, skills: all(5);
}
