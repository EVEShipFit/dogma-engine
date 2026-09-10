use crate::harness::all;

const FIT: &str = r#"
[Wreathe, 20240806 - Community Fit]
Expanded Cargohold II
Expanded Cargohold II
Expanded Cargohold II
Inertial Stabilizers II
Inertial Stabilizers II

Large Shield Extender I
Large Shield Extender I
Multispectrum Shield Hardener I
Thermal Shield Amplifier I
EM Shield Amplifier I


Medium Hyperspatial Velocity Optimizer I
Medium Hyperspatial Velocity Optimizer I
Medium Hyperspatial Velocity Optimizer I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
