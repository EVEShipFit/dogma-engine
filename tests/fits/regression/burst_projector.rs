//! Burst projectors keep their cycle time in an attribute of their own; without
//! it, capacitor peak load divides by zero.

use crate::harness::all;

const ECM: &str = r#"
[Nyx, ECM Jammer Burst Projector]

ECM Jammer Burst Projector
"#;

const SENSOR_DAMPENING: &str = r#"
[Nyx, Sensor Dampening Burst Projector]

Sensor Dampening Burst Projector
"#;

const TARGET_ILLUMINATION: &str = r#"
[Nyx, Target Illumination Burst Projector]

Target Illumination Burst Projector
"#;

const WEAPON_DISRUPTION: &str = r#"
[Nyx, Weapon Disruption Burst Projector]

Weapon Disruption Burst Projector
"#;

regression! {
    ecm = ECM, skills: all(5);
    sensor_dampening = SENSOR_DAMPENING, skills: all(5);
    target_illumination = TARGET_ILLUMINATION, skills: all(5);
    weapon_disruption = WEAPON_DISRUPTION, skills: all(5);
}
