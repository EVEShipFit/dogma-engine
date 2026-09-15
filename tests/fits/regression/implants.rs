//! Implants are in the character, not the ship. A set bonus reaches the other
//! implants of the set via the character; on a structure, nothing applies.

use crate::harness::all;

const PUNISHER: &str = r#"
[Punisher, Implants]
High-grade Amulet Alpha
High-grade Amulet Beta
High-grade Amulet Gamma
High-grade Amulet Delta
High-grade Amulet Epsilon
High-grade Amulet Omega
"#;

const ASTRAHUS: &str = r#"
[Astrahus, Implants]
High-grade Amulet Alpha
High-grade Amulet Beta
High-grade Amulet Gamma
High-grade Amulet Delta
High-grade Amulet Epsilon
High-grade Amulet Omega
"#;

regression! {
    set_skills_0 = PUNISHER, skills: all(0);
    set_skills_5 = PUNISHER, skills: all(5);
    without_omega_skills_5 = PUNISHER, skills: all(5), edit: |fit| {
        fit.items.pop();
    };
    structure_skills_5 = ASTRAHUS, skills: all(5);
}
