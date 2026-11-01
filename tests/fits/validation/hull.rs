//! A hull that is not a ship, structure or container.

use crate::harness::all;

const CHARGE: &str = r#"%esf/1
Tornado "Charge"

800mm Repeating Cannon II
"#;

const MODULE: &str = r#"%esf/1
Tornado "Module"

5x Hammerhead II
"#;

const CONTAINER: &str = r#"%esf/1
Tornado "Container"

10x Hammerhead II @cargo
"#;

const SHIP: &str = r#"%esf/1
Tornado "Ship"
"#;

validation! {
    charge = CHARGE, skills: all(5), edit: |fit| fit.ship.type_id = 194;
    module = MODULE, skills: all(5), edit: |fit| fit.ship.type_id = 563;
    unknown = SHIP, skills: all(5), edit: |fit| fit.ship.type_id = -1;
    container = CONTAINER, skills: all(5), edit: |fit| fit.ship.type_id = 11489;
}
