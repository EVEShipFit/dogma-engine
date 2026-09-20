//! An attribute can name two others as its floor and its cap. Polarized
//! weapons assign a resonance of 100, and only the cap brings it back to 1,
//! leaving every resistance at 0%.

use crate::harness::all;

const POLARIZED: &str = r#"
[Armageddon, Polarized]
Polarized Mega Pulse Laser
Polarized Mega Pulse Laser
Polarized Mega Pulse Laser
Polarized Mega Pulse Laser
Polarized Mega Pulse Laser
"#;

/* One gun zeroes the resistances the same as a full rack. */
const POLARIZED_ONE: &str = r#"
[Armageddon, Polarized single]
Polarized Mega Pulse Laser
"#;

/* Resistance modules win none of it back: the assign runs last, and the cap
 * it lands on is the same either way. */
const POLARIZED_TANKED: &str = r#"
[Armageddon, Polarized tanked]
Polarized Mega Pulse Laser

Multispectrum Shield Hardener II

Damage Control II
Multispectrum Energized Membrane II
"#;

regression! {
    polarized_rack = POLARIZED, skills: all(5);
    polarized_single = POLARIZED_ONE, skills: all(5);
    polarized_tanked = POLARIZED_TANKED, skills: all(5);
}
