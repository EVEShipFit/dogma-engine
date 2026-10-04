use crate::harness::all;

const FIT: &str = r#"%esf/1
Wreathe "20240806 - Community Fit"

2x Large Shield Extender I
Multispectrum Shield Hardener I
Thermal Shield Amplifier I
EM Shield Amplifier I

3x Expanded Cargohold II
2x Inertial Stabilizers II

3x Medium Hyperspatial Velocity Optimizer I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
