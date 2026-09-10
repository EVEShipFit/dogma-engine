use crate::harness::all;

const FIT: &str = r#"
[Gila, 20240806 - Community Fit by HateLesS]
Drone Damage Amplifier II
Drone Damage Amplifier II
Drone Damage Amplifier II

Multispectrum Shield Hardener II
Republic Fleet Large Cap Battery
10MN Monopropellant Enduring Afterburner
Multispectrum Shield Hardener II
Copasetic Compact Shield Boost Amplifier
Pithum C-Type Medium Shield Booster

Upgraded 'Malkuth' Rapid Light Missile Launcher, Caldari Navy Mjolnir Light Missile
Upgraded 'Malkuth' Rapid Light Missile Launcher, Caldari Navy Mjolnir Light Missile
Drone Link Augmentor I
Upgraded 'Malkuth' Rapid Light Missile Launcher, Caldari Navy Mjolnir Light Missile
Upgraded 'Malkuth' Rapid Light Missile Launcher, Caldari Navy Mjolnir Light Missile

Medium Core Defense Operational Solidifier I
Medium Core Defense Operational Solidifier I
Medium Core Defense Operational Solidifier II



Imperial Navy Infiltrator x2
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
