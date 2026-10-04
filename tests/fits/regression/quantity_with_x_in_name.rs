//! A quantity section is recognised by its trailing "x<quantity>" token, not by
//! the first "x" in the line, which here sits inside the type names.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Rorqual "Quantity with x in name"

2x 'Excavator' Mining Drone

100x Meson Exotic Plasma L
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
