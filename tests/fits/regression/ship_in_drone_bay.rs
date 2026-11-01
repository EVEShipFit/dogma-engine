//! Only a drone is calculated in the drone bay; anything else sits there
//! like cargo.

use esf_dogma_engine::{Options, Slot};

use crate::harness::{all, calculate};

const FIT: &str = r#"%esf/1
Rifter "Ship in the drone bay"

Rifter @cargo
"#;

#[test]
fn not_calculated() {
    let edit = |fit: &mut esf_dogma_engine::Fit| fit.items[0].slot = Slot::DroneBay;
    let (_, calculation) = calculate(FIT, all(5), edit, &Options::default());

    assert!(calculation.items[0].attributes.is_empty());
}
