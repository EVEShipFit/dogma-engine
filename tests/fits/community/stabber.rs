use crate::harness::all;

const FIT: &str = r#"%esf/1
Stabber "20240806 - Community Fit by Nora Maldroan"

2x 220mm Vulcan AutoCannon II :Barrage M
Medium Infectious Scoped Energy Neutralizer
Small Infectious Scoped Energy Neutralizer
2x 220mm Vulcan AutoCannon II :Barrage M

50MN Cold-Gas Enduring Microwarpdrive
2x Large Shield Extender II
Warp Disruptor II

Tracking Enhancer II
Damage Control II
2x Gyrostabilizer II

Medium Ancillary Current Router I
Medium EM Shield Reinforcer I
Medium Polycarbon Engine Housing I

5x Acolyte II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
