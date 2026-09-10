use crate::harness::all;

const FIT: &str = r#"
[Stabber, 20240806 - Community Fit by Nora Maldroan]
Tracking Enhancer II
Damage Control II
Gyrostabilizer II
Gyrostabilizer II

50MN Cold-Gas Enduring Microwarpdrive
Large Shield Extender II
Large Shield Extender II
Warp Disruptor II

220mm Vulcan AutoCannon II, Barrage M
220mm Vulcan AutoCannon II, Barrage M
Medium Infectious Scoped Energy Neutralizer
Small Infectious Scoped Energy Neutralizer
220mm Vulcan AutoCannon II, Barrage M
220mm Vulcan AutoCannon II, Barrage M

Medium Ancillary Current Router I
Medium EM Shield Reinforcer I
Medium Polycarbon Engine Housing I



Acolyte II x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
