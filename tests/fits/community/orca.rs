use crate::harness::all;

const FIT: &str = r#"%esf/1
Orca "20240806 - Community Fit by Limal"

Large Industrial Core II
Large Asteroid Ore Compressor I
Mining Foreman Burst II :Mining Laser Field Enhancement Charge
Shield Command Burst II :Shield Extension Charge
Mining Foreman Burst II :Mining Laser Optimization Charge
Small Tractor Beam II

2x Drone Navigation Computer II
Multispectrum Shield Hardener II
Pith X-Type Thermal Shield Hardener
Pith X-Type Kinetic Shield Hardener

Reinforced Bulkheads II
Damage Control II

2x Large Drone Mining Augmentor II
Large Drone Mining Augmentor I

5x 'Augmented' Mining Drone
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
