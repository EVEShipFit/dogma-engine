use crate::harness::all;

const FIT: &str = r#"%esf/1
Rupture "20240806 - Community Fit"

4x 220mm Medium Prototype Automatic Cannon :Republic Fleet EMP M

Large C5-L Compact Shield Booster
Compact Multispectrum Shield Hardener
10MN Y-S8 Compact Afterburner
Large Compact Pb-Acid Cap Battery

3x Counterbalanced Compact Gyrostabilizer
Fourier Compact Tracking Enhancer
IFFA Compact Damage Control

Medium EM Shield Reinforcer I
Medium Projectile Ambit Extension I
Medium Projectile Burst Aerator I

5x Acolyte I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
