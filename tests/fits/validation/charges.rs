//! What a module takes in it.

use crate::harness::all;

const WRONG_SIZE: &str = r#"
[Rifter, Wrong size]
200mm AutoCannon II, EMP M
"#;

const WRONG_GROUP: &str = r#"
[Rifter, Wrong group]
200mm AutoCannon II, Multifrequency S
"#;

/* A capacitor booster sorts its charges by volume alone, so the size on the
 * charge is no help; only the room in the module is. */
const TOO_LARGE: &str = r#"
[Rifter, Too large]
Small Capacitor Booster II, Cap Booster 800
"#;

validation! {
    charge_size = WRONG_SIZE, skills: all(5);
    charge_group = WRONG_GROUP, skills: all(5);
    charge_capacity = TOO_LARGE, skills: all(5);
}
