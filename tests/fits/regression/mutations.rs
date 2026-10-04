//! A mutated item takes the attributes and effects of its base, then those of
//! the type it mutated into, then the rolled values.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Tristan "Mutations"

Warp Scrambler II +Unstable Warp Scrambler {capacitorNeed 7.5, cpu 30, maxRange 10500}

Hobgoblin II +Exigent Light Drone Firepower {armorHP 110, damageMultiplier 2.2, falloff 2100, hp 250, maxRange 2200, maxVelocity 3500, shieldCapacity 62, trackingSpeed 2.3}
2x Hobgoblin II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
