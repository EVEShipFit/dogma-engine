use crate::harness::all;

const FIT: &str = r#"%esf/1
Omen "20240721 - Community Fit"

5x Focused Anode Pulse Particle Stream I :Imperial Navy Multifrequency M

10MN Y-S8 Compact Afterburner
Large Compact Pb-Acid Cap Battery
X5 Enduring Stasis Webifier

3x Extruded Compact Heat Sink
2x Compact Multispectrum Energized Membrane
Medium ACM Compact Armor Repairer

Medium EM Armor Reinforcer I
2x Medium Auxiliary Nano Pump I

5x Acolyte I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
