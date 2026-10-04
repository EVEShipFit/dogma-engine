//! DNA, the fits EVE links to in chat.

use esf_data::{Info, InfoName};
use esf_dogma_engine::{Fit, Slot};
use esf_format::dna::{Error, load_dna};

use crate::harness::{info, info_name, snapshot_json};

fn id(name: &str) -> i32 {
    info_name().type_name_to_id(name).unwrap()
}

/// `<ship>:<type>;<quantity>:...::`, by name; a name ending in `_` is in the cargo.
fn dna(ship: &str, items: &[(&str, u32)]) -> String {
    let mut dna = id(ship).to_string();
    for (name, quantity) in items {
        let (name, cargo) = match name.strip_suffix('_') {
            Some(name) => (name, "_"),
            None => (*name, ""),
        };
        dna += &format!(":{}{cargo};{quantity}", id(name));
    }
    dna + "::"
}

/// Each item of the fit, as name, slot and quantity.
fn items(fit: &Fit) -> Vec<(String, Slot, u32)> {
    fit.items
        .iter()
        .map(|item| {
            let name = info().get_type(item.type_id).unwrap().name().to_string();
            (name, item.slot, item.quantity)
        })
        .collect()
}

fn item(name: &str, slot: Slot, quantity: u32) -> (String, Slot, u32) {
    (name.to_string(), slot, quantity)
}

#[test]
fn loads_a_fit() {
    let dna = dna(
        "Rifter",
        &[
            ("200mm AutoCannon II", 3),
            ("1MN Afterburner II", 1),
            ("Warp Scrambler II", 1),
            ("Damage Control II", 1),
            ("Gyrostabilizer II", 2),
            ("Small Projectile Burst Aerator I", 1),
            ("Small Auxiliary Nano Pump I", 2),
            ("Warrior II", 3),
            ("Republic Fleet EMP S", 400),
            ("Nanite Repair Paste", 10),
            ("Ocular Filter - Basic", 1),
            ("Small Armor Repairer II_", 1),
        ],
    );
    snapshot_json("dna-rifter", &load_dna(&info(), &dna).unwrap());
}

#[test]
fn reads_a_chat_link() {
    let dna = dna("Rifter", &[("200mm AutoCannon II", 1)]);
    let link = format!("fitting:{dna}");

    assert_eq!(
        items(&load_dna(&info(), &link).unwrap()),
        items(&load_dna(&info(), &dna).unwrap())
    );
}

#[test]
fn leaves_out_the_last_item_of_a_link_cut_short() {
    let dna = dna("Rifter", &[("Damage Control II", 1), ("Warrior II", 3)]);
    let cut = &dna[..dna.len() - 4];

    assert_eq!(
        items(&load_dna(&info(), cut).unwrap()),
        [item("Damage Control II", Slot::Low(0), 1)]
    );
}

#[test]
fn puts_a_module_marked_for_cargo_in_the_cargo() {
    let dna = dna(
        "Rifter",
        &[("Damage Control II", 1), ("Damage Control II_", 2)],
    );

    assert_eq!(
        items(&load_dna(&info(), &dna).unwrap()),
        [
            item("Damage Control II", Slot::Low(0), 1),
            item("Damage Control II", Slot::Cargo, 2),
        ]
    );
}

#[test]
fn fills_a_rack_up_to_eight_slots() {
    let dna = dna("Rifter", &[("Damage Control II", 10)]);
    let fit = load_dna(&info(), &dna).unwrap();

    assert_eq!(fit.items.len(), 8);
    assert_eq!(fit.items[7].slot, Slot::Low(7));
}

#[test]
fn skips_an_unknown_type() {
    let dna = format!("{}:999999999;1::", id("Rifter"));
    assert!(load_dna(&info(), &dna).unwrap().items.is_empty());
}

#[test]
fn needs_a_ship() {
    let dna = format!("{};1::", id("Damage Control II"));
    assert_eq!(load_dna(&info(), &dna).unwrap_err(), Error::NoShip);
}

#[test]
fn needs_numbers() {
    assert_eq!(
        load_dna(&info(), "587:x;1::").unwrap_err(),
        Error::InvalidNumber("x".to_string())
    );
}
