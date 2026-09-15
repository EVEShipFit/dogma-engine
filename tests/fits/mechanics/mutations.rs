//! A mutated item takes the attributes and effects of its base, then those of
//! the type it mutated into, then the rolled values.

use esf_format::eft::Error;

use crate::harness::{all, load};

const FIT: &str = r#"
[Tristan, Mutations]
Warp Scrambler II [1]

Hobgoblin II x1 [2]
Hobgoblin II x2

[1] Warp Scrambler II
  Unstable Warp Scrambler Mutaplasmid
  capacitorNeed 7.5, cpu 30, maxRange 10500
[2] Hobgoblin II
  Exigent Light Drone Firepower Mutaplasmid
  armorHP 110, damageMultiplier 2.2, falloff 2100, hp 250, maxRange 2200, maxVelocity 3500, shieldCapacity 62, trackingSpeed 2.3
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}

#[test]
fn missing_roll() {
    let eft = FIT.replace("capacitorNeed 7.5, ", "");
    assert_eq!(
        load(&eft).unwrap_err(),
        Error::MissingRoll {
            mutation: "[1] Warp Scrambler II".to_string(),
            attribute_id: 6,
        }
    );
}
