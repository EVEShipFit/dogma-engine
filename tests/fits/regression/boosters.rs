//! A booster always gives its bonus, but only the side effects the fit rolled.
//! EFT has no way to say which, so the case sets them on the loaded fit.

use std::collections::BTreeSet;

use crate::harness::all;

const FIT: &str = r#"
[Rifter, Boosters]
Medium Shield Booster II

200mm AutoCannon II

Standard Blue Pill Booster
"#;

const SIDE_EFFECTS: [i32; 4] = [2737, 2739, 2745, 2749];

regression! {
    bonus_skills_0 = FIT, skills: all(0);
    side_effects_skills_0 = FIT, skills: all(0), edit: roll;
    side_effects_skills_5 = FIT, skills: all(5), edit: roll;
}

fn roll(fit: &mut esf_dogma_engine::Fit) {
    fit.items[2].booster_side_effects = BTreeSet::from(SIDE_EFFECTS);
}
