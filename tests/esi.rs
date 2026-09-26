//! ESI fittings, as a character saves them in game.

use esf_dogma_engine::{Fit, Slot};
use esf_format::esi::{EsiFitting, EsiFittingItem, load_esi_fitting, save_esi_fitting};

use crate::harness::{info, input, load, snapshot_json};

/* Every rack, charges, drones, cargo, and an implant a fitting has no place for. */
const RIFTER: &str = "\
[Rifter, Autocannon brawler]
Damage Control II
Small Armor Repairer II
Gyrostabilizer II
Nanofiber Internal Structure II

1MN Afterburner II
Warp Scrambler II
X5 Enduring Stasis Webifier

200mm AutoCannon II, Republic Fleet EMP S
200mm AutoCannon II, Republic Fleet EMP S
200mm AutoCannon II, Republic Fleet EMP S

Small Projectile Burst Aerator I
Small Auxiliary Nano Pump I
Small Auxiliary Nano Pump I

Warrior II x3

Republic Fleet EMP S x400
Nanite Repair Paste x10

Ocular Filter - Basic
";

fn read(name: &str) -> EsiFitting {
    serde_json::from_str(&input(&format!("{name}.esi.json"))).unwrap()
}

fn by_flag(mut items: Vec<EsiFittingItem>) -> Vec<EsiFittingItem> {
    items.sort_by(|a, b| a.flag.cmp(&b.flag).then(a.type_id.cmp(&b.type_id)));
    items
}

fn empty(name: Option<&str>) -> Fit {
    let mut fit = load("[Rifter, Empty]").unwrap();
    fit.name = name.map(str::to_string);
    fit
}

#[test]
fn loads_a_fitting() {
    snapshot_json("esi-loki", &load_esi_fitting(&info(), &read("loki")));
}

#[test]
fn saves_a_fitting() {
    let fit = load(RIFTER).unwrap();
    snapshot_json("esi-rifter", &save_esi_fitting(&info(), &fit));
}

#[test]
fn round_trips_a_fitting() {
    let fitting = read("loki");
    let again = save_esi_fitting(&info(), &load_esi_fitting(&info(), &fitting));

    assert_eq!(by_flag(again.items), by_flag(fitting.items.clone()));
    assert_eq!(again.name, fitting.name);
    assert_eq!(again.ship_type_id, fitting.ship_type_id);
}

#[test]
fn lists_a_charge_in_the_slot_of_its_module() {
    let fit = load("[Rifter, Gun]\n\n200mm AutoCannon II, EMP S").unwrap();
    let fitting = save_esi_fitting(&info(), &fit);

    let charge = fit.items[0].charge.as_ref().unwrap().type_id;
    assert_eq!(
        fitting.items,
        [
            EsiFittingItem {
                flag: "HiSlot0".to_string(),
                quantity: 1,
                type_id: fit.items[0].type_id,
            },
            EsiFittingItem {
                flag: "HiSlot0".to_string(),
                quantity: 1,
                type_id: charge,
            },
        ]
    );

    let again = load_esi_fitting(&info(), &fitting);
    assert_eq!(again.items.len(), 1);
    assert_eq!(again.items[0].charge.as_ref().unwrap().type_id, charge);
}

/* ESI has no fighter tubes, implants or boosters. */
#[test]
fn puts_what_a_fitting_has_no_place_for_elsewhere() {
    let mut fit = load("[Thanatos, Fighters]\n\nTemplar II x6").unwrap();
    fit.items[0].slot = Slot::FighterTube(0);
    let tube = save_esi_fitting(&info(), &fit);
    assert_eq!(tube.items[0].flag, "FighterBay");

    let implant = save_esi_fitting(&info(), &load(RIFTER).unwrap());
    let ocular = implant.items.last().unwrap();
    assert_eq!(ocular.flag, "Cargo");
}

#[test]
fn skips_an_unknown_flag() {
    let fitting = EsiFitting {
        fitting_id: None,
        name: "Odd".to_string(),
        description: String::new(),
        ship_type_id: 587,
        items: vec![EsiFittingItem {
            flag: "FuelBay".to_string(),
            quantity: 1,
            type_id: 2048,
        }],
    };

    assert!(load_esi_fitting(&info(), &fitting).items.is_empty());
}

#[test]
fn cuts_the_name_to_what_esi_takes() {
    let long = "x".repeat(80);
    assert_eq!(
        save_esi_fitting(&info(), &empty(Some(&long))).name.len(),
        50
    );
}

#[test]
fn names_the_fitting_after_the_ship_when_it_has_no_name() {
    assert_eq!(save_esi_fitting(&info(), &empty(None)).name, "Rifter");
    assert_eq!(save_esi_fitting(&info(), &empty(Some(""))).name, "Rifter");
}
