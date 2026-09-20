//! A capacitor booster injects what its charge holds, so it takes load off the
//! capacitor instead of putting it on, and the depletion simulation credits it.
//!
//! An ancillary shield booster takes the same charge as fuel, and must not
//! hand the ship any capacitor for it.

use crate::harness::all;

const LOADED: &str = r#"
[Vexor, Capacitor booster loaded]

Medium Capacitor Booster II, Cap Booster 400
"#;

const EMPTY: &str = r#"
[Vexor, Capacitor booster empty]

Medium Capacitor Booster II
"#;

/* Runs dry in 40s without the booster, and holds with it. */
const DRAINED: &str = r#"
[Vexor, Capacitor booster keeps up]
Medium Armor Repairer II
Medium Armor Repairer II

5MN Microwarpdrive II
Medium Capacitor Booster II, Cap Booster 400
"#;

const FUEL_ONLY: &str = r#"
[Caracal, Cap booster charge as fuel]

Medium Ancillary Shield Booster, Cap Booster 100
"#;

regression! {
    loaded_skills_5 = LOADED, skills: all(5);
    empty_skills_5 = EMPTY, skills: all(5);
    drained_skills_5 = DRAINED, skills: all(5);
    fuel_only_skills_5 = FUEL_ONLY, skills: all(5);
}
