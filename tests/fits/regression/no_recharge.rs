//! A hull without rechargeRate or shieldRechargeRate gets no capacitor peak or
//! passive shield recharge, instead of NaN.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Moros Interbus Edition "No recharge"
"#;

regression! {
    skills_5 = FIT, skills: all(5);
}
