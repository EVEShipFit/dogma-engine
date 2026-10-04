use crate::harness::all;

const FIT: &str = r#"%esf/1
Typhoon Fleet Issue "20240720 - Community Fit by Melamori"

6x Rapid Heavy Missile Launcher II :Caldari Navy Scourge Heavy Missile
Auto Targeting System I
Drone Link Augmentor II

Large Shield Booster II
2x Missile Guidance Computer II :Missile Precision Script
Eutectic Compact Cap Recharger
500MN Quad LiF Restrained Microwarpdrive

3x Ballistic Control System II
2x Drone Damage Amplifier II
Ballistic Control System II
Missile Guidance Enhancer II

Large EM Shield Reinforcer II
2x Large Capacitor Control Circuit I

5x Ogre II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
