//! An owner modifier reaches drones, fighters and charges, not fitted modules.
//! The Federation Navy 'Argyreos' Magnetic Field Stabilizer requires Drones,
//! so drone damage bonuses (Drone Interfacing, the Drone Damage Amplifier and
//! its own drone bonus) must not raise the hybrid damage it hands the blaster;
//! the Hammerheads still get all of them.
//!
//! Pyfa agrees on both damage multipliers: 5.005778 for the blaster and
//! 2.658163 for the Hammerheads with all skills at 0, 9.894233 and 8.223692
//! with all skills at 5.

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
