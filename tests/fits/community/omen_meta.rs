use crate::harness::all;

const FIT: &str = r#"
[Omen, 20240721 - Community Fit]
Extruded Compact Heat Sink
Extruded Compact Heat Sink
Extruded Compact Heat Sink
Compact Multispectrum Energized Membrane
Compact Multispectrum Energized Membrane
Medium ACM Compact Armor Repairer

10MN Y-S8 Compact Afterburner
Large Compact Pb-Acid Cap Battery
X5 Enduring Stasis Webifier

Focused Anode Pulse Particle Stream I, Imperial Navy Multifrequency M
Focused Anode Pulse Particle Stream I, Imperial Navy Multifrequency M
Focused Anode Pulse Particle Stream I, Imperial Navy Multifrequency M
Focused Anode Pulse Particle Stream I, Imperial Navy Multifrequency M
Focused Anode Pulse Particle Stream I, Imperial Navy Multifrequency M

Medium EM Armor Reinforcer I
Medium Auxiliary Nano Pump I
Medium Auxiliary Nano Pump I



Acolyte I x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
