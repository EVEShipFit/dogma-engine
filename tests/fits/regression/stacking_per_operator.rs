//! Stacking penalty chains are per operator: the Damage Control (PreMul)
//! is not penalized by the Energized Membranes (PostPercent).

use crate::harness::all;

const FIT: &str = r#"
[Rifter, Stacking per operator]

Damage Control II
EM Energized Membrane II
EM Energized Membrane II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
