use crate::harness::all;

const FIT: &str = r#"%esf/1
Vexor "20240721 - Community Fit"

4x Heavy Electron Blaster II :Caldari Navy Antimatter Charge M

Large Compact Pb-Acid Cap Battery
Stasis Webifier II
Omnidirectional Tracking Link II :Tracking Speed Script
10MN Y-S8 Compact Afterburner

2x Drone Damage Amplifier II
Medium Armor Repairer II
Multispectrum Energized Membrane II
Damage Control II

2x Medium Auxiliary Nano Pump I
Medium Explosive Armor Reinforcer I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
