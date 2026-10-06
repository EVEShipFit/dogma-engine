//! A subsystem is always online, whatever state the fit asks for.

use crate::harness::all;

const FIT: &str = r#"%esf/1
Loki "Subsystem online"

Loki Core - Augmented Nuclear Reactor
Loki Defensive - Covert Reconfiguration !off
Loki Offensive - Launcher Efficiency Configuration
Loki Propulsion - Wake Limiter
"#;

regression! {
    skills_5 = FIT, skills: all(5);
}
