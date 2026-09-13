//! A quantity section is recognised by its trailing "x<quantity>" token, not by
//! the first "x" in the line, which here sits inside the type names.

use crate::harness::all;

const FIT: &str = r#"
[Venture, Quantity with x in name]

'Excavator' Mining Drone x2

Meson Exotic Plasma L x100
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
