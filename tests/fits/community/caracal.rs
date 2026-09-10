use crate::harness::all;

const FIT: &str = r#"
[Caracal, 20240721 - Community Fit]
Ballistic Control System I
Ballistic Control System I
Ballistic Control System I
IFFA Compact Damage Control

10MN Y-S8 Compact Afterburner
Large C5-L Compact Shield Booster
Large Compact Pb-Acid Cap Battery
Compact EM Shield Amplifier
X5 Enduring Stasis Webifier

Prototype 'Arbalest' Heavy Assault Missile Launcher I, Scourge Heavy Assault Missile
Prototype 'Arbalest' Heavy Assault Missile Launcher I, Scourge Heavy Assault Missile
Prototype 'Arbalest' Heavy Assault Missile Launcher I, Scourge Heavy Assault Missile
Prototype 'Arbalest' Heavy Assault Missile Launcher I, Scourge Heavy Assault Missile
Prototype 'Arbalest' Heavy Assault Missile Launcher I, Scourge Heavy Assault Missile

Medium Ancillary Current Router I
Medium Ancillary Current Router I
Medium Ancillary Current Router I



Hornet I x2
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
