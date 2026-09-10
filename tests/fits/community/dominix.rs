use crate::harness::all;

const FIT: &str = r#"
[Dominix, 20240721 - Community Fit by HateLesS]
Drone Damage Amplifier II
Drone Damage Amplifier II
Drone Damage Amplifier II
Drone Damage Amplifier II
Omnidirectional Tracking Enhancer II
Omnidirectional Tracking Enhancer II
Omnidirectional Tracking Enhancer II

Large Micro Jump Drive
Multispectrum Shield Hardener II
Multispectrum Shield Hardener II
X-Large Shield Booster II
Sensor Booster II

Drone Link Augmentor II
Drone Link Augmentor II
Drone Link Augmentor II
Drone Link Augmentor II

Large Processor Overclocking Unit I
Large Processor Overclocking Unit I
Large Capacitor Control Circuit I



Garde I x5
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
