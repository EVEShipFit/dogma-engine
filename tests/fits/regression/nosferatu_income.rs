//! A nosferatu brings capacitor in, both in peak delta and in the depletion
//! simulation. Without that income both fits run dry at every skill level.

use crate::harness::all;

const STABLE: &str = r#"%esf/1
Punisher "Nosferatu income stable"

2x Small Energy Nosferatu II

5MN Microwarpdrive II

Small Armor Repairer II
"#;

/* Still unstable with the income, so the simulation has to credit it too. */
const DEPLETES: &str = r#"%esf/1
Punisher "Nosferatu income depletes"

2x Small Energy Nosferatu II

1MN Afterburner II
Warp Disruptor II

Small Armor Repairer II
"#;

regression! {
    stable_skills_0 = STABLE, skills: all(0);
    stable_skills_5 = STABLE, skills: all(5);
    depletes_skills_0 = DEPLETES, skills: all(0);
}
