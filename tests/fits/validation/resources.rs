//! What the ship only has so much of.

use crate::harness::all;

const FITTING: &str = r#"
[Rifter, Fitting]
1600mm Steel Plates II
Warp Disruptor II
Warp Disruptor II
Warp Disruptor II
"#;

const CALIBRATION: &str = r#"
[Rifter, Calibration]
Small Projectile Collision Accelerator I
Small Projectile Collision Accelerator I
Small Projectile Collision Accelerator I
"#;

const MANY_DRONES: &str = r#"
[Vexor, Many drones]

Hammerhead II x13
"#;

const FEW_DRONES: &str = r#"
[Vexor, Few drones]

Hobgoblin II x5
"#;

const CARGO: &str = r#"
[Rifter, Cargo]

EMP S x100000
"#;

validation! {
    cpu_and_powergrid = FITTING, skills: all(5);
    calibration = CALIBRATION, skills: all(5);
    drone_bay = MANY_DRONES, skills: all(5);
    launched_drones = FEW_DRONES, skills: all(5).with("Drones", 0);
    cargo_bay = CARGO, skills: all(5);
}
