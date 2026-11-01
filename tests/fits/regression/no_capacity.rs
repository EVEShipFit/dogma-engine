//! A module with charge groups but no capacity gets no reload time, instead of infinity.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Punisher "No capacity"

Test Site Gatling Pulse Laser :Multifrequency S
"#;

regression! {
    skills_5 = FIT, skills: all(5);
}
