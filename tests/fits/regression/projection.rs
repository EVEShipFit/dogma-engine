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

/* Logistics repairs the ship it is aimed at, which is a rate on that ship
 * rather than an attribute of its own. */
const REMOTE_ARMOR: &str = r#"
[Oneiros, Remote Armor]
Large Remote Armor Repairer II
"#;

const REMOTE_SHIELD: &str = r#"
[Basilisk, Remote Shield]
Large Remote Shield Booster II
"#;

const REMOTE_HULL: &str = r#"
[Osprey, Remote Hull]
Large Remote Hull Repairer II
"#;

const REMOTE_ARMOR_DRONES: &str = r#"
[Vexor, Repair Drones]

Heavy Armor Maintenance Bot I x5
"#;

/* Capacitor warfare: one gives capacitor away, the other two take it. */
const REMOTE_CAP: &str = r#"
[Basilisk, Remote Cap]
Large Remote Capacitor Transmitter II
"#;

const NEUTRALIZER: &str = r#"
[Curse, Neutralizer]
Heavy Energy Neutralizer II
"#;

const NOSFERATU: &str = r#"
[Curse, Nosferatu]
Heavy Energy Nosferatu II
"#;

/* A Marauder in bastion shrugs off remote assistance, and a dreadnought in
 * siege shrugs off dampening and weapon disruption. */
const BASTION: &str = r#"
[Paladin, Bastion]
Bastion Module I
"#;

const SIEGE: &str = r#"
[Revelation, Siege]
Siege Module II
Dual Giga Pulse Laser II, Conflagration L
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
    /* Command bursts reach a fleet mate as buffs, and an empty burst says
     * nothing at all. */
    bursts = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(BURSTS, all(5))); };
    bursts_without_charges = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(BURSTS_WITHOUT_CHARGES, all(5))); };
    /* Bastion drops remote assistance to a twentieth, and siege takes
     * seventy percent off dampening and weapon disruption. */
    bastion_alone = BASTION, skills: all(5);
    siege_alone = SIEGE, skills: all(5);
    sensor_booster_on_bastion = BASTION, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(SENSOR_BOOSTER, all(5))); };
    dampener_on_siege = SIEGE, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(DAMPENER, all(5))); };
    tracking_disruptor_on_siege = SIEGE, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(TRACKING_DISRUPTOR, all(5))); };
    /* Logistics adds to the repair rate of the ship it is aimed at, and
     * bastion leaves almost none of it. */
    remote_armor_rep = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_ARMOR, all(5))); };
    remote_shield_boost = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_SHIELD, all(5))); };
    remote_hull_rep = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_HULL, all(5))); };
    remote_armor_rep_drones = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_ARMOR_DRONES, all(5))); };
    remote_armor_rep_on_bastion = BASTION, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_ARMOR, all(5))); };
    /* Capacitor moved onto or off the fit lands on its peak load, which is
     * what decides how long the capacitor lasts. */
    remote_cap = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(REMOTE_CAP, all(5))); };
    neutralizer = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(NEUTRALIZER, all(5))); };
    nosferatu = TARGET, skills: all(5), edit: |fit| { fit.incoming.extend(outgoing(NOSFERATU, all(5))); };
    /* Everything at once, to lock how they pile up. */
    all_of_it = TARGET, skills: all(5), edit: |fit| {
        fit.incoming.extend(outgoing(WEB, all(5)));
        fit.incoming.extend(outgoing(PAINTER, all(5)));
        fit.incoming.extend(outgoing(DAMPENER, all(5)));
        fit.incoming.extend(outgoing(TRACKING_DISRUPTOR, all(5)));
        fit.incoming.extend(outgoing(GUIDANCE_DISRUPTOR, all(5)));
    };
}
