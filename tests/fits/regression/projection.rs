//! One fit aimed at another. The engine never links them up itself: a fit
//! reports what it hands out, and the caller puts that in the fit it reaches.

use crate::harness::{all, outgoing};

/* Turrets, missiles and a propulsion module, so every kind of ewar has
 * something to bite on. */
const TARGET: &str = r#"
[Rupture, Target]
Damage Control II
Gyrostabilizer II

10MN Afterburner II
Large Shield Extender II

425mm AutoCannon II, Republic Fleet EMP M
Heavy Missile Launcher II, Scourge Fury Heavy Missile
"#;

const WEB: &str = r#"
[Celestis, Web]
Stasis Webifier II
"#;

const TWO_WEBS: &str = r#"
[Celestis, Webs]
Stasis Webifier II
Stasis Webifier II
"#;

const PAINTER: &str = r#"
[Celestis, Painter]
Target Painter II
"#;

const DAMPENER: &str = r#"
[Celestis, Dampener]
Remote Sensor Dampener II
"#;

const TRACKING_DISRUPTOR: &str = r#"
[Celestis, Tracking Disruptor]
Tracking Disruptor II
"#;

const GUIDANCE_DISRUPTOR: &str = r#"
[Celestis, Guidance Disruptor]
Guidance Disruptor II
"#;

const SENSOR_BOOSTER: &str = r#"
[Osprey, Sensor Booster]
Remote Sensor Booster II
"#;

const WEB_DRONES: &str = r#"
[Vexor, Web Drones]

'Aergia' Hobgoblin SW-300 x5
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
    /* The baseline the cases below are read against. */
    target_alone = TARGET, skills: all(5);
    web = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(WEB, all(5))); };
    painter = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(PAINTER, all(5))); };
    dampener = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(DAMPENER, all(5))); };
    tracking_disruptor = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(TRACKING_DISRUPTOR, all(5))); };
    guidance_disruptor = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(GUIDANCE_DISRUPTOR, all(5))); };
    sensor_booster = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(SENSOR_BOOSTER, all(5))); };
    /* A drone stack aims once per drone, so five of them stack five times. */
    web_drones = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(WEB_DRONES, all(5))); };
    /* Two webs from one ship, and from two, both stack penalised. */
    two_webs_one_ship = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(TWO_WEBS, all(5))); };
    two_webs_two_ships = TARGET, skills: all(5), edit: |fit| {
        fit.incoming.extend(outgoing(WEB, all(5)));
        fit.incoming.extend(outgoing(WEB, all(5)));
    };
    /* Everything at once, to lock how they pile up. */
    all_of_it = TARGET, skills: all(5), edit: |fit| {
        fit.incoming.extend(outgoing(WEB, all(5)));
        fit.incoming.extend(outgoing(PAINTER, all(5)));
        fit.incoming.extend(outgoing(DAMPENER, all(5)));
        fit.incoming.extend(outgoing(TRACKING_DISRUPTOR, all(5)));
        fit.incoming.extend(outgoing(GUIDANCE_DISRUPTOR, all(5)));
    };
}
