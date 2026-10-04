//! What the ship only has so much of.

use crate::harness::all;

const FITTING: &str = r#"%esf/1
Rifter "Fitting"

3x Warp Disruptor II

1600mm Steel Plates II
"#;

const CALIBRATION: &str = r#"%esf/1
Rifter "Calibration"

3x Small Projectile Collision Accelerator I
"#;

const MANY_DRONES: &str = r#"%esf/1
Vexor "Many drones"

13x Hammerhead II
"#;

const FEW_DRONES: &str = r#"%esf/1
Vexor "Few drones"

5x Hobgoblin II
"#;

const CARGO: &str = r#"%esf/1
Rifter "Cargo"

100000x EMP S
"#;

validation! {
    cpu_and_powergrid = FITTING, skills: all(5);
    calibration = CALIBRATION, skills: all(5);
    drone_bay = MANY_DRONES, skills: all(5);
    launched_drones = FEW_DRONES, skills: all(5).with("Drones", 0);
    cargo_bay = CARGO, skills: all(5);
}
