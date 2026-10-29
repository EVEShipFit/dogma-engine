//! Drone damage bonuses reach the Hammerheads, not the 'Argyreos' that
//! requires Drones.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Vexor "Owner bonus not on modules"

Heavy Neutron Blaster II :Void M

Federation Navy 'Argyreos' Magnetic Field Stabilizer
Federation Navy 'Argyreos' Magnetic Field Stabilizer
Drone Damage Amplifier II

5x Hammerhead II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
