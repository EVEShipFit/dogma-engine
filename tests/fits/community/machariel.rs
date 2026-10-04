use crate::harness::all;

const FIT: &str = r#"%esf/1
Machariel "20240721 - Community Fit by HateLesS"

7x 800mm Repeating Cannon II :Republic Fleet EMP L
Small Tractor Beam II

Gist X-Type 500MN Microwarpdrive
Gist X-Type X-Large Shield Booster
Pithum C-Type Multispectrum Shield Hardener
Large Micro Jump Drive
Shadow Serpentis Tracking Computer :Optimal Range Script
Multispectrum Shield Hardener II

3x Republic Fleet Gyrostabilizer
3x Domination Tracking Enhancer

3x Large Hyperspatial Velocity Optimizer II

5x Warrior II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
