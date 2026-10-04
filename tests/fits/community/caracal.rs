use crate::harness::all;

const FIT: &str = r#"%esf/1
Caracal "20240721 - Community Fit"

5x Prototype 'Arbalest' Heavy Assault Missile Launcher I :Scourge Heavy Assault Missile

10MN Y-S8 Compact Afterburner
Large C5-L Compact Shield Booster
Large Compact Pb-Acid Cap Battery
Compact EM Shield Amplifier
X5 Enduring Stasis Webifier

3x Ballistic Control System I
IFFA Compact Damage Control

3x Medium Ancillary Current Router I

2x Hornet I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
