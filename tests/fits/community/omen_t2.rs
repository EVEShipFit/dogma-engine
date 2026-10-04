use crate::harness::all;

const FIT: &str = r#"%esf/1
Omen "20240721 - Community Fit"

5x Focused Medium Pulse Laser II :Conflagration M

50MN Quad LiF Restrained Microwarpdrive
Warp Disruptor II
Medium F-RX Compact Capacitor Booster :Navy Cap Booster 800

800mm Crystalline Carbonide Restrained Plates
Damage Control II
2x Heat Sink II
Medium Ancillary Armor Repairer
Multispectrum Energized Membrane II

2x Medium Energy Locus Coordinator II
Medium Polycarbon Engine Housing I

3x Hammerhead II
2x Warrior II
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
