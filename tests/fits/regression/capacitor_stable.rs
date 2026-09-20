//! Where the capacitor settles once recharge and drain cancel out.
//!
//! Checked against EVE itself: at these skills the fitting window reports 63.0%
//! excess capacitor recharge rate, so `capacitorPeakDeltaPercentage` has to
//! land there for the level it feeds to mean anything.

use crate::harness::all;

const STABLE: &str = r#"
[Dominix, Capacitor stable]

Cap Recharger II
Cap Recharger II
Cap Recharger II
Cap Recharger II
Cap Recharger II

Heavy Energy Neutralizer I
"#;

regression! {
    stable = STABLE, skills: all(0).with("Capacitor Systems Operation", 3).with("Capacitor Management", 3);
}
