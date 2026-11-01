//! A module with a cycle time of 0 drains the capacitor at once.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Tristan "Zero cycle time"

Warp Scrambler II +Unstable Warp Scrambler {capacitorNeed 7.5, duration 0}
"#;

regression! {
    skills_0 = FIT, skills: all(0);
}
