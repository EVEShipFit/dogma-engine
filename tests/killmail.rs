//! ESI killmails, the ship that died as it was fitted.

use esf_dogma_engine::{Fit, Slot, State};
use esf_format::esi::{load_esi_fitting, save_esi_fitting};
use esf_format::killmail::{EsiKillmail, load_killmail};

use crate::harness::{info, input, snapshot_json};

const SHIP: i32 = 117923593;
const STRUCTURE: i32 = 117621358;
const CAPSULE: i32 = 138642919;

fn killmail(id: i32) -> Fit {
    let killmail: EsiKillmail =
        serde_json::from_str(&input(&format!("killmail-{id}.json"))).unwrap();
    load_killmail(&info(), &killmail)
}

/* Fits do not compare, so their JSON stands in for them. */
fn json(fit: &Fit) -> serde_json::Value {
    serde_json::to_value(fit).unwrap()
}

#[test]
fn loads_a_ship() {
    snapshot_json("killmail-ship", &killmail(SHIP));
}

#[test]
fn loads_a_structure() {
    snapshot_json("killmail-structure", &killmail(STRUCTURE));
}

#[test]
fn loads_a_capsule() {
    snapshot_json("killmail-capsule", &killmail(CAPSULE));
}

#[test]
fn names_the_fit_after_the_killmail() {
    assert_eq!(killmail(SHIP).name.as_deref(), Some("Killmail 117923593"));
}

#[test]
fn loads_a_charge_in_the_module_of_its_flag() {
    let charges: Vec<_> = killmail(SHIP)
        .items
        .iter()
        .filter(|item| matches!(item.slot, Slot::High(_)))
        .map(|item| item.charge.as_ref().map(|charge| charge.type_id))
        .collect();

    assert_eq!(
        charges,
        [Some(12777), Some(12777), None, Some(12777), Some(12777)]
    );
}

#[test]
fn adds_up_what_was_destroyed_and_what_dropped() {
    let fit = killmail(STRUCTURE);
    let fighters: Vec<_> = fit
        .items
        .iter()
        .filter(|item| item.type_id == 47141)
        .collect();

    assert_eq!(fighters.len(), 1);
    assert_eq!(fighters[0].slot, Slot::FighterBay);
    assert_eq!(fighters[0].quantity, 8);
    assert_eq!(fighters[0].state, State::Offline);
}

#[test]
fn puts_an_implant_in_its_slot() {
    let slots: Vec<_> = killmail(CAPSULE)
        .items
        .iter()
        .map(|item| item.slot)
        .collect();

    assert_eq!(
        slots,
        [Slot::Implant(7), Slot::Implant(9), Slot::Implant(10)]
    );
}

/* A fit has no place for a fuel bay, nor for what is inside a container. */
#[test]
fn leaves_out_what_a_fit_has_no_place_for() {
    let killmail: EsiKillmail = serde_json::from_str(
        r#"{
            "killmail_id": 1,
            "killmail_time": "2026-09-23T16:11:55Z",
            "victim": {
                "ship_type_id": 587,
                "items": [
                    {"flag": 133, "item_type_id": 16274, "quantity_dropped": 10, "singleton": 0},
                    {
                        "flag": 5, "item_type_id": 3467, "quantity_destroyed": 1, "singleton": 0,
                        "items": [{"flag": 5, "item_type_id": 2048, "quantity_dropped": 1, "singleton": 0}]
                    }
                ]
            }
        }"#,
    )
    .unwrap();

    let fit = load_killmail(&info(), &killmail);
    assert_eq!(fit.items.len(), 1);
    assert_eq!(fit.items[0].type_id, 3467);
}

#[test]
fn round_trips_through_an_esi_fitting() {
    for id in [SHIP, STRUCTURE] {
        let fit = killmail(id);
        let again = load_esi_fitting(&info(), &save_esi_fitting(&info(), &fit));
        assert_eq!(json(&again), json(&fit), "killmail {id}");
    }
}

/* ESI fittings have no implants, so they go in the cargo. */
#[test]
fn saves_an_implant_as_cargo_in_an_esi_fitting() {
    let fitting = save_esi_fitting(&info(), &killmail(CAPSULE));
    let flags: Vec<_> = fitting
        .items
        .iter()
        .map(|item| item.flag.as_str())
        .collect();

    assert_eq!(flags, ["Cargo", "Cargo", "Cargo"]);
}
