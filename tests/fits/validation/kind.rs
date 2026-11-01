//! A mode or skill of a type that is not one.

use crate::harness::all;

const SHIP: &str = r#"%esf/1
Tornado "Ship"
"#;

validation! {
    charge_as_mode = SHIP, skills: all(5), edit: |fit| fit.ship.mode = Some(194);
    charge_as_skill = SHIP, skills: all(5), edit: |fit| _ = fit.character.skills.insert(194, 5);
}
