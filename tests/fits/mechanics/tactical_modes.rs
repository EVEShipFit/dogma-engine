//! A mode applies its bonuses to the ship, and via the character to the
//! modules and charges that need a skill. EFT has no line for it, so the case
//! sets it on the loaded fit.

use crate::harness::all;

const CONFESSOR: &str = r#"
[Confessor, Tactical modes]
Heat Sink II

1MN Afterburner II

Small Focused Beam Laser II
"#;

const JACKDAW: &str = r#"
[Jackdaw, Tactical modes]



Light Missile Launcher II, Scourge Light Missile
"#;

const CONFESSOR_DEFENSE: i32 = 34319;
const CONFESSOR_SHARPSHOOTER: i32 = 34321;
const CONFESSOR_PROPULSION: i32 = 34323;
const JACKDAW_SHARPSHOOTER: i32 = 35678;

regression! {
    none_skills_0 = CONFESSOR, skills: all(0);
    none_skills_5 = CONFESSOR, skills: all(5);
    defense_skills_0 = CONFESSOR, skills: all(0), edit: |fit| fit.ship.mode = Some(CONFESSOR_DEFENSE);
    defense_skills_5 = CONFESSOR, skills: all(5), edit: |fit| fit.ship.mode = Some(CONFESSOR_DEFENSE);
    sharpshooter_skills_0 = CONFESSOR, skills: all(0), edit: |fit| fit.ship.mode = Some(CONFESSOR_SHARPSHOOTER);
    sharpshooter_skills_5 = CONFESSOR, skills: all(5), edit: |fit| fit.ship.mode = Some(CONFESSOR_SHARPSHOOTER);
    propulsion_skills_0 = CONFESSOR, skills: all(0), edit: |fit| fit.ship.mode = Some(CONFESSOR_PROPULSION);
    propulsion_skills_5 = CONFESSOR, skills: all(5), edit: |fit| fit.ship.mode = Some(CONFESSOR_PROPULSION);
    missile_none_skills_5 = JACKDAW, skills: all(5);
    missile_sharpshooter_skills_5 = JACKDAW, skills: all(5), edit: |fit| fit.ship.mode = Some(JACKDAW_SHARPSHOOTER);
}
