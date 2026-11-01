//! Only a fighter is calculated in a fighter tube; anything else sits there
//! like cargo.

use esf_dogma_engine::{Options, Slot};

use crate::harness::{all, calculate};

const FIT: &str = r#"%esf/1
Thanatos "Laser in a tube"

Dual Heavy Beam Laser I
"#;

#[test]
fn not_calculated() {
    let edit = |fit: &mut esf_dogma_engine::Fit| fit.items[0].slot = Slot::FighterTube(0);
    let (_, calculation) = calculate(FIT, all(5), edit, &Options::default());

    assert!(calculation.items[0].attributes.is_empty());
}
