//! Two stacks of the same drone keep their own state: only the active stack
//! counts towards the drones in space, and each stack is its quantity strong.

use crate::harness::all;
use esf_dogma::fit::State;

const FIT: &str = r#"
[Vexor, Drone stack state]
Drone Damage Amplifier II

Hammerhead II x3
Hammerhead II x2
"#;

regression! {
    skills_0 = FIT, skills: all(0), edit: |fit| fit.items[2].state = State::Offline;
    skills_5 = FIT, skills: all(5), edit: |fit| fit.items[2].state = State::Offline;
}
