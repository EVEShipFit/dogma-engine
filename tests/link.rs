//! EVEShip.fit links, by the payload they carry once unpacked.

use esf_dogma_engine::{Fit, Slot, State};
use esf_format::link::{Error, load_link};

use crate::harness::{info_name, input, snapshot_json};

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
}
