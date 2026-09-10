use crate::harness::all;

const FIT: &str = r#"
[Orca, 20240806 - Community Fit by Limal]
Reinforced Bulkheads II
Damage Control II

Drone Navigation Computer II
Drone Navigation Computer II
Multispectrum Shield Hardener II
Pith X-Type Thermal Shield Hardener
Pith X-Type Kinetic Shield Hardener

Large Industrial Core II
Large Asteroid Ore Compressor I
Mining Foreman Burst II, Mining Laser Field Enhancement Charge
Shield Command Burst II, Shield Extension Charge
Mining Foreman Burst II, Mining Laser Optimization Charge
Small Tractor Beam II

Large Drone Mining Augmentor II
Large Drone Mining Augmentor II
Large Drone Mining Augmentor I



'Augmented' Mining Drone x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
