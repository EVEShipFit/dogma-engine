//! A command burst only hands out buffs while it runs.

use crate::harness::{all, outgoing};

const CLAYMORE: &str = r#"%esf/1
Claymore "Shield Boosts"

Shield Command Burst I :Shield Harmonizing Charge
"#;

#[test]
fn running_burst_hands_out_buffs() {
    assert!(!outgoing(CLAYMORE, all(5)).buffs.is_empty());
}

#[test]
fn burst_that_is_not_running_hands_out_nothing() {
    for state in ["!off", "!on"] {
        let text = CLAYMORE.replace("Charge\n", &format!("Charge {state}\n"));
        assert!(outgoing(&text, all(5)).buffs.is_empty(), "{state}");
    }
}
