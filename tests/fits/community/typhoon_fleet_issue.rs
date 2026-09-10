use crate::harness::all;

const FIT: &str = r#"
[Typhoon Fleet Issue, 20240720 - Community Fit by Melamori]
Ballistic Control System II
Ballistic Control System II
Ballistic Control System II
Drone Damage Amplifier II
Drone Damage Amplifier II
Ballistic Control System II
Missile Guidance Enhancer II

Large Shield Booster II
Missile Guidance Computer II, Missile Precision Script
Missile Guidance Computer II, Missile Precision Script
Eutectic Compact Cap Recharger
500MN Quad LiF Restrained Microwarpdrive

Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Rapid Heavy Missile Launcher II, Caldari Navy Scourge Heavy Missile
Auto Targeting System I
Drone Link Augmentor II

Large EM Shield Reinforcer II
Large Capacitor Control Circuit I
Large Capacitor Control Circuit I



Ogre II x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
