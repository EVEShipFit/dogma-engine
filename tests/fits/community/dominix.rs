use crate::harness::all;

const FIT: &str = r#"%esf/1
Dominix "20240721 - Community Fit by HateLesS"

4x Drone Link Augmentor II

Large Micro Jump Drive
2x Multispectrum Shield Hardener II
X-Large Shield Booster II
Sensor Booster II

4x Drone Damage Amplifier II
3x Omnidirectional Tracking Enhancer II

2x Large Processor Overclocking Unit I
Large Capacitor Control Circuit I

5x Garde I
"#;

regression! {
    skills_0 = FIT, skills: all(0);
    skills_5 = FIT, skills: all(5);
}
