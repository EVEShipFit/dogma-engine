//! EVEShip.fit links, by the payload they carry once unpacked.

use std::collections::BTreeSet;

use esf_dogma_engine::{Fit, Security, Slot, Spool, State};
use esf_format::link::{Error, load_link, save_link};

use crate::harness::{info_name, input, load, snapshot_json};

/* A mutated module, a drone in space, a squadron in a tube, a booster with a
 * side effect, and a fit that is not in high-sec; all of it a v4 keeps. */
const EVERYTHING: &str = "\
[Thanatos, Everything]

Warp Scrambler II [1]
Heavy Entropic Disintegrator II, Occult L

Templar II x6

Hobgoblin II x5

Standard Blue Pill Booster

[1] Warp Scrambler II
  Unstable Warp Scrambler Mutaplasmid
  capacitorNeed 7.5, cpu 30, maxRange 10500
";

fn everything() -> Fit {
    let mut fit = load(EVERYTHING).unwrap();
    fit.character.skills.insert(3300, 5);
    fit.environment.security = Security::LowSec;
    for item in &mut fit.items {
        match item.slot {
            Slot::FighterBay => item.slot = Slot::FighterTube(0),
            Slot::High(_) if item.charge.is_some() => {
                item.spool = Some(Spool::MultiplierBonus(0.5));
            }
            Slot::Booster(_) => item.booster_side_effects = BTreeSet::from([5]),
            _ => {}
        }
    }
    fit
}

/* Fits do not compare, so their JSON stands in for them. */
fn json(fit: &Fit) -> serde_json::Value {
    serde_json::to_value(fit).unwrap()
}

fn link(version: &str, payload: &str) -> Result<Fit, Error> {
    load_link(&info_name(), version, payload)
}

fn read(name: &str, extension: &str) -> Fit {
    let (_, version) = name.split_once('.').unwrap();
    link(version, &input(&format!("{name}.{extension}"))).unwrap()
}

#[test]
fn loads_a_v1() {
    snapshot_json("link-tornado.v1", &read("tornado.v1", "csv"));
}

#[test]
fn loads_a_v2() {
    snapshot_json("link-loki.v2", &read("loki.v2", "csv"));
}

#[test]
fn loads_a_v3() {
    snapshot_json("link-loki.v3", &read("loki.v3", "csv"));
}

#[test]
fn loads_an_eft() {
    snapshot_json("link-buzzard.eft", &read("buzzard.eft", "txt"));
}

#[test]
fn loads_a_v4() {
    snapshot_json("link-rifter.v4", &read("rifter.v4", "json"));
}

#[test]
fn saves_a_v4() {
    let payload: serde_json::Value = serde_json::from_str(&save_link(&everything())).unwrap();
    snapshot_json("link-everything.v4", &payload);
}

#[test]
fn round_trips_a_v4() {
    let mut fit = everything();
    let again = link("v4", &save_link(&fit)).unwrap();

    fit.character = Default::default();
    assert_eq!(json(&again), json(&fit));
}

/* Whoever opens a link sees the fit with their own skills. */
#[test]
fn leaves_the_character_out_of_a_v4() {
    let payload = save_link(&everything());
    assert!(!payload.contains("character"));

    let with_character =
        r#"{"ship": {"type_id": 587}, "items": [], "character": {"skills": {"3300": 5}}}"#;
    assert!(
        link("v4", with_character)
            .unwrap()
            .character
            .skills
            .is_empty()
    );
}

#[test]
fn loads_the_charge_and_state_of_a_v2() {
    let fit = link("v2", "587,Gun,\n27,2873,1,185,Offline\n28,2873,1,,Overload").unwrap();

    assert_eq!(fit.items[0].slot, Slot::High(0));
    assert_eq!(fit.items[0].charge.as_ref().unwrap().type_id, 185);
    assert_eq!(fit.items[0].state, State::Offline);
    assert!(fit.items[1].charge.is_none());
    assert_eq!(fit.items[1].state, State::Overload);
}

#[test]
fn loads_active_and_passive_drones_of_a_v3() {
    let fit = link("v3", "ship,32872,Drones,\ndrone,2456,5,3\ndrone,2486,0,2").unwrap();
    let drones: Vec<_> = fit
        .items
        .iter()
        .map(|item| (item.type_id, item.quantity, item.state))
        .collect();

    assert_eq!(
        drones,
        [
            (2456, 5, State::Active),
            (2456, 3, State::Offline),
            (2486, 2, State::Offline)
        ]
    );
}

#[test]
fn names_a_type_an_eft_has_by_id() {
    let fit = link("eft", "[Rifter, Ids]\n2873\n\n185 x100").unwrap();

    assert_eq!(fit.items[0].type_id, 2873);
    assert_eq!(fit.items[1].type_id, 185);
    assert_eq!(fit.items[1].quantity, 100);
}

#[test]
fn has_no_name_when_the_link_has_none() {
    assert_eq!(link("v1", "587,,\n").unwrap().name, None);
}

#[test]
fn skips_an_unknown_flag() {
    assert!(
        link("v1", "587,Odd,\n133,16274,10")
            .unwrap()
            .items
            .is_empty()
    );
}

#[test]
fn refuses_what_is_not_a_fit() {
    assert_eq!(
        link("v9", "").unwrap_err(),
        Error::UnknownVersion("v9".to_string())
    );
    assert_eq!(link("killmail", "1/abc").unwrap_err(), Error::Killmail);
    assert_eq!(
        link("v2", "Rifter,Odd,").unwrap_err(),
        Error::InvalidNumber("Rifter".to_string())
    );
    assert_eq!(
        link("v3", "ship,587,Odd,\nmodule,High,0,2873,Active,").unwrap_err(),
        Error::InvalidNumber("0".to_string())
    );
    assert!(matches!(link("eft", "not a fit"), Err(Error::Eft(_))));
    assert!(matches!(link("v4", "[]"), Err(Error::InvalidJson(_))));
    assert!(matches!(link("v4", "{"), Err(Error::InvalidJson(_))));
}
