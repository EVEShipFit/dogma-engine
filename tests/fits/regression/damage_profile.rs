//! The damage profile only changes effective hitpoints. The SDE effects do the
//! work; the engine only supplies the profile.

use esf_dogma_engine::DamageProfile;

use crate::harness::all;

const RIFTER: &str = r#"
[Rifter, Damage profile]
Damage Control II
Small Armor Repairer II

Medium Shield Extender II
"#;

regression! {
    rifter_uniform = RIFTER, skills: all(5);
    rifter_em = RIFTER, skills: all(5), edit: |fit| fit.environment.damage_profile = DamageProfile { em: 1.0, explosive: 0.0, kinetic: 0.0, thermal: 0.0 };
    /* Does not add up to one, so it is scaled. */
    rifter_kinetic_thermal = RIFTER, skills: all(5), edit: |fit| fit.environment.damage_profile = DamageProfile { em: 0.0, explosive: 0.0, kinetic: 3.0, thermal: 1.0 };
}
