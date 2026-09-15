//! A few ships scale a bonus with the pilot's security status. The SDE effects
//! do the work; the engine only supplies the status.

use crate::harness::all;

/* Repair amount: +10% per point of status. */
const ENFORCER: &str = r#"
[Enforcer, Security status]
Medium Armor Repairer II

Medium Shield Booster II

Heavy Assault Missile Launcher II, Scourge Heavy Assault Missile
"#;

/* Damage: +7.5% per point of negative status, on both guns and missiles. */
const SIDEWINDER: &str = r#"
[Sidewinder, Security status]



125mm Gatling AutoCannon II, EMP S
Rocket Launcher II, Scourge Rocket
"#;

regression! {
    enforcer_status_0 = ENFORCER, skills: all(5);
    enforcer_status_5 = ENFORCER, skills: all(5), edit: |fit| fit.character.security_status = 5.0;
    enforcer_status_minus_10 = ENFORCER, skills: all(5), edit: |fit| fit.character.security_status = -10.0;
    sidewinder_status_0 = SIDEWINDER, skills: all(5);
    sidewinder_status_5 = SIDEWINDER, skills: all(5), edit: |fit| fit.character.security_status = 5.0;
    sidewinder_status_minus_10 = SIDEWINDER, skills: all(5), edit: |fit| fit.character.security_status = -10.0;
}
