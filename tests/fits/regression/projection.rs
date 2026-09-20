//! One fit aimed at another. The engine never links them up itself: a fit
//! reports what it hands out, and the caller puts that in the fit it reaches.

use crate::harness::all;

const WEB: &str = r#"
[Celestis, Web]
Stasis Webifier II
"#;

/* A burst hands over buffs rather than effects, and only once it has a
 * charge naming which. */
const BURSTS: &str = r#"
[Claymore, Bursts]
Shield Command Burst II, Shield Harmonizing Charge
Skirmish Command Burst II, Rapid Deployment Charge
"#;

const BURSTS_WITHOUT_CHARGES: &str = r#"
[Claymore, Bursts]
Shield Command Burst II
Skirmish Command Burst II
"#;

regression! {
    /* What a fit hands out is what lands on another. An effect carries the
     * attributes the receiver reads; a burst carries buffs instead, and
     * without a charge to name one it carries nothing. */
    web_boat = WEB, skills: all(5);
    bursts_boat = BURSTS, skills: all(5);
    bursts_boat_without_charges = BURSTS_WITHOUT_CHARGES, skills: all(5);
}
