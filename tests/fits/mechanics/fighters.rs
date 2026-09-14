//! A fit says where each squadron is and how big it is; nothing is launched
//! on import. A squadron in a tube uses its default abilities unless the fit
//! picks others, and takes one tube however many fighters it holds.

use std::collections::BTreeSet;

use crate::harness::all;
use esf_dogma_engine::{Slot, State};

const FIT: &str = r#"
[Thanatos, Fighters]
Fighter Support Unit II

Templar II x6

Dromi II x3
"#;

const ATTACK: i32 = 6465;
const MISSILES: i32 = 6431;

regression! {
    bay_skills_5 = FIT, skills: all(5);
    launched_skills_0 = FIT, skills: all(0), edit: launch;
    launched_skills_5 = FIT, skills: all(5), edit: launch;
    missiles_skills_5 = FIT, skills: all(5), edit: |fit| {
        launch(fit);
        fit.items[1].fighter_abilities = Some(BTreeSet::from([ATTACK, MISSILES]));
    };
    offline_skills_5 = FIT, skills: all(5), edit: |fit| {
        launch(fit);
        fit.items[1].state = State::Offline;
    };
}

fn launch(fit: &mut esf_dogma_engine::Fit) {
    fit.items[1].slot = Slot::FighterTube(0);
    fit.items[1].state = State::Active;
    fit.items[2].slot = Slot::FighterTube(1);
    fit.items[2].state = State::Active;
}
