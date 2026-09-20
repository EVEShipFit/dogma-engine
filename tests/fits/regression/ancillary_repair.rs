//! The ancillary repairers and boosters hold fueledArmorRepair and
//! fueledShieldBoosting, a second name for the effect their normal
//! counterparts use, so the repair and boost rates missed them and stayed at 0.
//!
//! The loaded repairer is pinned at its unboosted rate:
//! chargedArmorDamageMultiplier is not applied yet.

use crate::harness::all;

const ARMOR_LOADED: &str = r#"
[Vexor, Ancillary armor repairer loaded]
Medium Ancillary Armor Repairer, Nanite Repair Paste
"#;

const ARMOR_EMPTY: &str = r#"
[Vexor, Ancillary armor repairer empty]
Medium Ancillary Armor Repairer
"#;

const SHIELD_LOADED: &str = r#"
[Caracal, Ancillary shield booster loaded]

Medium Ancillary Shield Booster, Cap Booster 100
"#;

/* Without a charge it falls back on the capacitor, and still boosts. */
const SHIELD_EMPTY: &str = r#"
[Caracal, Ancillary shield booster empty]

Medium Ancillary Shield Booster
"#;

/* The fit from the report, which showed 0 for both layers at once. */
const BOTH: &str = r#"
[Vexor, Ancillary both layers]
Medium Ancillary Armor Repairer, Nanite Repair Paste

Medium Ancillary Shield Booster, Cap Booster 100
"#;

regression! {
    armor_loaded_skills_0 = ARMOR_LOADED, skills: all(0);
    armor_loaded_skills_5 = ARMOR_LOADED, skills: all(5);
    armor_empty_skills_5 = ARMOR_EMPTY, skills: all(5);
    shield_loaded_skills_0 = SHIELD_LOADED, skills: all(0);
    shield_loaded_skills_5 = SHIELD_LOADED, skills: all(5);
    shield_empty_skills_5 = SHIELD_EMPTY, skills: all(5);
    both_skills_5 = BOTH, skills: all(5);
}
