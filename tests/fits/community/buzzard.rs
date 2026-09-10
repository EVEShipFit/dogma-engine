use crate::harness::all;

const FIT: &str = r#"
[Buzzard, 20240721 - Community Fit by Kane Carnifex]
Inertial Stabilizers II
Inertial Stabilizers II
Warp Core Stabilizer II

Relic Analyzer II
Cargo Scanner II
Data Analyzer II
Sensor Booster II, Targeting Range Script
5MN Y-T8 Compact Microwarpdrive

Sisters Core Probe Launcher, Sisters Core Scanner Probe
Interdiction Nullifier II
Covert Ops Cloaking Device II

Small Ancillary Current Router II
Small Ancillary Current Router I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
