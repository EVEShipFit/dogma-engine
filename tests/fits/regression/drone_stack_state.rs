//! Two stacks of the same drone keep their own state: only the active stack
//! counts towards the drones in space, and each stack is its quantity strong.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Vexor "Drone stack state"

Drone Damage Amplifier II

3x Hammerhead II
2x Hammerhead II !off
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
