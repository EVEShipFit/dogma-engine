//! What the character has to have trained, for the hull, the modules and the
//! charges alike.

use crate::harness::{all, none};

const FIT: &str = r#"
[Rifter, Skills]
200mm AutoCannon II, Barrage S
"#;

/* The specialization is trained, but nothing it rests on is. */
const PREREQUISITES: &str = r#"
[Rifter, Prerequisites]
200mm AutoCannon II
"#;

validation! {
    untrained = FIT, skills: none();
    trained = FIT, skills: all(5);
    prerequisites = PREREQUISITES, skills: none().with("Small Autocannon Specialization", 5);
}
