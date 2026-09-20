//! What the character has to have trained, for the hull, the modules and the
//! charges alike.

use crate::harness::{all, none};

const FIT: &str = r#"
[Rifter, Skills]
200mm AutoCannon II, Barrage S
"#;

validation! {
    untrained = FIT, skills: none();
    trained = FIT, skills: all(5);
}
