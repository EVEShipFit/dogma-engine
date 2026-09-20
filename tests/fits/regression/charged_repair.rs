//! An ancillary armor repairer repairs three times as much while a Nanite
//! Repair Paste is in, and falls back to its plain rate on capacitor alone.
//! The remote ones carry the same multiplier.
//!
//! The paste does not pay for the capacitor the way a cap booster charge does,
//! so the repairer keeps its capacitor need either way.

use crate::harness::all;

const LOADED: &str = r#"
[Vexor, Charged repair loaded]
Medium Ancillary Armor Repairer, Nanite Repair Paste
"#;

const EMPTY: &str = r#"
[Vexor, Charged repair empty]
Medium Ancillary Armor Repairer
"#;

/* A plain repairer holds no multiplier, so paste or not it never trebles. */
const PLAIN: &str = r#"
[Vexor, Charged repair plain repairer]
Medium Armor Repairer II
"#;

const REMOTE: &str = r#"
[Vexor, Charged repair remote]

Medium Ancillary Remote Armor Repairer, Nanite Repair Paste
"#;

regression! {
    loaded_skills_0 = LOADED, skills: all(0);
    loaded_skills_5 = LOADED, skills: all(5);
    empty_skills_5 = EMPTY, skills: all(5);
    plain_skills_5 = PLAIN, skills: all(5);
    remote_skills_5 = REMOTE, skills: all(5);
}
