//! Validate various of bonuses different configuration give to structures.

use esf_dogma_engine::Security;

use crate::harness::all;

/* Powered (because of Cloning Center) should give more hp / cap. */
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

/* Rigs should apply bonuses to modules, more so outside high-sec. */
const RIGGED: &str = r#"
[Astrahus, Structure bonus rigged]


Standup Multirole Missile Launcher I, Standup Cruise Missile

Standup M-Set Missile Precision I
Standup M-Set Missile Projection I
Standup M-Set Enhanced Targeting System I
"#;

regression! {
    powered_skills_0 = POWERED, skills: all(0);
    powered_skills_5 = POWERED, skills: all(5);
    unpowered_skills_5 = UNPOWERED, skills: all(5);
    rigged_skills_5 = RIGGED, skills: all(5);
    rigged_low_sec = RIGGED, skills: all(5), edit: |fit| fit.environment.security = Security::LowSec;
    rigged_null_sec = RIGGED, skills: all(5), edit: |fit| fit.environment.security = Security::NullSec;
    rigged_wormhole = RIGGED, skills: all(5), edit: |fit| fit.environment.security = Security::Wormhole;
}
