use crate::harness::all;

const FIT: &str = r#"%esf/1
Gila "20240806 - Community Fit by HateLesS"

2x Upgraded 'Malkuth' Rapid Light Missile Launcher :Caldari Navy Mjolnir Light Missile
Drone Link Augmentor I
2x Upgraded 'Malkuth' Rapid Light Missile Launcher :Caldari Navy Mjolnir Light Missile

Multispectrum Shield Hardener II
Republic Fleet Large Cap Battery
10MN Monopropellant Enduring Afterburner
Multispectrum Shield Hardener II
Copasetic Compact Shield Boost Amplifier
Pithum C-Type Medium Shield Booster

3x Drone Damage Amplifier II

2x Medium Core Defense Operational Solidifier I
Medium Core Defense Operational Solidifier II

2x Imperial Navy Infiltrator
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
