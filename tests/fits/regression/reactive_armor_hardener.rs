//! The Reactive Armor Hardener shifts resistance towards the damage coming in,
//! every cycle, until it settles. The engine simulates that and hands the
//! module the resistances it ends up holding, but only when asked: EVE itself
//! has no damage to shift against, and shows a plain 15/15/15/15 hardener.

use esf_dogma_engine::{DamageProfile, ReactiveArmor, State};

use crate::harness::all;

const RIFTER: &str = r#"
[Rifter, Reactive Armor Hardener]
Reactive Armor Hardener
Damage Control II
"#;

const THERMAL: DamageProfile = DamageProfile {
    em: 0.0,
    explosive: 0.0,
    kinetic: 0.0,
    thermal: 1.0,
};

regression! {
    /* The default, and what EVE shows. */
    do_not_adapt = RIFTER, skills: all(5);
    /* Even damage never settles on one state; it loops over three. */
    uniform = RIFTER, skills: all(5), edit: |fit| fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
    /* One type only: the other three give everything they have. */
    thermal = RIFTER, skills: all(5), edit: |fit| {
        fit.environment.damage_profile = THERMAL;
        fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
    };
    /* Two types: both end up at the most one type can hold. */
    kinetic_thermal = RIFTER, skills: all(5), edit: |fit| {
        fit.environment.damage_profile = DamageProfile { em: 0.0, explosive: 0.0, kinetic: 79.0, thermal: 21.0 };
        fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
    };
    /* A profile of its own: the module shifts to thermal, while effective
     * hitpoints still answer to the even damage of the fit. */
    own_profile = RIFTER, skills: all(5), edit: |fit| fit.environment.reactive_armor = ReactiveArmor::Profile(THERMAL);
    /* Nothing shifts while it is not running. */
    offline = RIFTER, skills: all(5), edit: |fit| {
        fit.items[0].state = State::Offline;
        fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
    };
    /* Only the cycle time overloads, and with one module that changes nothing. */
    overloaded = RIFTER, skills: all(5), edit: |fit| {
        fit.items[0].state = State::Overload;
        fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
    };
    /* A pilot without the skill still shifts the same; only the cycle differs. */
    no_skills = RIFTER, skills: all(0), edit: |fit| fit.environment.reactive_armor = ReactiveArmor::DamageProfile;
}
