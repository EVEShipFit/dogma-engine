//! Drones are not located in the ship, so ship-location bonuses skip them: no
//! rig, hull or skill bonus to ECM modules reaches the Hornets.

use crate::harness::all;

const FIT: &str = r#"
[Scorpion, Location bonus not on drones]

Large Particle Dispersion Augmentor II

Hornet EC-300 x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
