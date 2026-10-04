//! A fit says where each squadron is and how big it is; nothing is launched
//! on import. A squadron in a tube uses its default abilities unless the fit
//! picks others, and takes one tube however many fighters it holds.

use std::collections::BTreeSet;

use crate::harness::all;

const FIT: &str = r#"%esf/1
Thanatos "Fighters"

Fighter Support Unit II

3x Dromi II @bay
6x Templar II @bay
"#;

const LAUNCHED: &str = r#"%esf/1
Thanatos "Fighters"

Fighter Support Unit II

3x Dromi II
6x Templar II
"#;

const LAUNCHED_OFFLINE: &str = r#"%esf/1
Thanatos "Fighters"

Fighter Support Unit II

3x Dromi II
6x Templar II !off
"#;

const ATTACK: i32 = 6465;
const MISSILES: i32 = 6431;

regression! {
    bay_skills_5 = FIT, skills: all(5);
    launched_skills_0 = LAUNCHED, skills: all(0);
    launched_skills_5 = LAUNCHED, skills: all(5);
    missiles_skills_5 = LAUNCHED, skills: all(5), edit: |fit| {
        fit.items[2].fighter_abilities = Some(BTreeSet::from([ATTACK, MISSILES]));
    };
    offline_skills_5 = LAUNCHED_OFFLINE, skills: all(5);
}
