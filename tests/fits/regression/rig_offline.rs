//! An offline rig gives no bonus and no drawback, but still uses calibration.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Rifter "Rig offline"

Small Core Defense Field Extender I
Small Core Defense Field Extender I !off
"#;

regression! {
    skills_5 = FIT, skills: all(5);
}
