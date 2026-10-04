//! A module with capacitorNeed but no cycle time stays out of capacitor peak
//! load, instead of dividing by a cycle time of 0.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Rifter "No cycle time"

QA Immunity Module
"#;

regression! {
    skills_5 = FIT, skills: all(5);
}
