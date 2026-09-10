use crate::harness::all;

const FIT: &str = r#"
[Rupture, 20240806 - Community Fit]
Counterbalanced Compact Gyrostabilizer
Counterbalanced Compact Gyrostabilizer
Counterbalanced Compact Gyrostabilizer
Fourier Compact Tracking Enhancer
IFFA Compact Damage Control

Large C5-L Compact Shield Booster
Compact Multispectrum Shield Hardener
10MN Y-S8 Compact Afterburner
Large Compact Pb-Acid Cap Battery

220mm Medium Prototype Automatic Cannon, Republic Fleet EMP M
220mm Medium Prototype Automatic Cannon, Republic Fleet EMP M
220mm Medium Prototype Automatic Cannon, Republic Fleet EMP M
220mm Medium Prototype Automatic Cannon, Republic Fleet EMP M

Medium EM Shield Reinforcer I
Medium Projectile Ambit Extension I
Medium Projectile Burst Aerator I



Acolyte I x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
