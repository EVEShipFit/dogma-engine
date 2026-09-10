use crate::harness::all;

const FIT: &str = r#"
[Omen, 20240721 - Community Fit]
800mm Crystalline Carbonide Restrained Plates
Damage Control II
Heat Sink II
Heat Sink II
Medium Ancillary Armor Repairer
Multispectrum Energized Membrane II

50MN Quad LiF Restrained Microwarpdrive
Warp Disruptor II
Medium F-RX Compact Capacitor Booster, Navy Cap Booster 800

Focused Medium Pulse Laser II, Conflagration M
Focused Medium Pulse Laser II, Conflagration M
Focused Medium Pulse Laser II, Conflagration M
Focused Medium Pulse Laser II, Conflagration M
Focused Medium Pulse Laser II, Conflagration M

Medium Energy Locus Coordinator II
Medium Energy Locus Coordinator II
Medium Polycarbon Engine Housing I



Warrior II x2
Hammerhead II x3
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
