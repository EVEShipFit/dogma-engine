//! A mode applies its bonuses to the ship, and via the character to the
//! modules and charges that need a skill.

use crate::harness::all;

const CONFESSOR: &str = r#"%esf/1
Confessor "Tactical modes"

Small Focused Beam Laser II

1MN Afterburner II

Heat Sink II
"#;

const JACKDAW: &str = r#"%esf/1
Jackdaw "Tactical modes"

Light Missile Launcher II :Scourge Light Missile
"#;

const CONFESSOR_DEFENSE: &str = r#"%esf/1
Confessor "Tactical modes" /Defense

Small Focused Beam Laser II

1MN Afterburner II

Heat Sink II
"#;

const CONFESSOR_SHARPSHOOTER: &str = r#"%esf/1
Confessor "Tactical modes" /Sharpshooter

Small Focused Beam Laser II

1MN Afterburner II

Heat Sink II
"#;

const CONFESSOR_PROPULSION: &str = r#"%esf/1
Confessor "Tactical modes" /Propulsion

Small Focused Beam Laser II

1MN Afterburner II

Heat Sink II
"#;

const JACKDAW_SHARPSHOOTER: &str = r#"%esf/1
Jackdaw "Tactical modes" /Sharpshooter

Light Missile Launcher II :Scourge Light Missile
"#;

regression! {
    none_skills_0 = CONFESSOR, skills: all(0);
    none_skills_5 = CONFESSOR, skills: all(5);
    defense_skills_0 = CONFESSOR_DEFENSE, skills: all(0);
    defense_skills_5 = CONFESSOR_DEFENSE, skills: all(5);
    sharpshooter_skills_0 = CONFESSOR_SHARPSHOOTER, skills: all(0);
    sharpshooter_skills_5 = CONFESSOR_SHARPSHOOTER, skills: all(5);
    propulsion_skills_0 = CONFESSOR_PROPULSION, skills: all(0);
    propulsion_skills_5 = CONFESSOR_PROPULSION, skills: all(5);
    missile_none_skills_5 = JACKDAW, skills: all(5);
    missile_sharpshooter_skills_5 = JACKDAW_SHARPSHOOTER, skills: all(5);
}
