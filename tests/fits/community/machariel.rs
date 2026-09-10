use crate::harness::all;

const FIT: &str = r#"
[Machariel, 20240721 - Community Fit by HateLesS]
Republic Fleet Gyrostabilizer
Republic Fleet Gyrostabilizer
Republic Fleet Gyrostabilizer
Domination Tracking Enhancer
Domination Tracking Enhancer
Domination Tracking Enhancer

Gist X-Type 500MN Microwarpdrive
Gist X-Type X-Large Shield Booster
Pithum C-Type Multispectrum Shield Hardener
Large Micro Jump Drive
Shadow Serpentis Tracking Computer, Optimal Range Script
Multispectrum Shield Hardener II

800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
800mm Repeating Cannon II, Republic Fleet EMP L
Small Tractor Beam II

Large Hyperspatial Velocity Optimizer II
Large Hyperspatial Velocity Optimizer II
Large Hyperspatial Velocity Optimizer II



Warrior II x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
