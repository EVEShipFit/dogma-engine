//! The states EVE gives a fit it imports.

use esf_data::Info;
use esf_dogma_engine::{Slot, State};
use esf_format::post_load;

use crate::harness::{Skills, all, info, load, none};

/// Each item of the fit after `post_load`, as name, quantity and state.
fn settle(eft: &str, skills: Skills) -> Vec<(String, u32, State)> {
    let mut fit = load(eft).unwrap();
    fit.character.skills = skills.levels;

    let info = info();
    post_load(&info, &mut fit);

    fit.items
        .iter()
        .map(|item| {
            let name = info.get_type(item.type_id).unwrap().name().to_string();
            (name, item.quantity, item.state)
        })
        .collect()
}

fn item(name: &str, quantity: u32, state: State) -> (String, u32, State) {
    (name.to_string(), quantity, state)
}

#[test]
fn keeps_a_fit_eve_allows() {
    let eft = "[Tristan, Fine]\n\n1MN Afterburner II\n\n\nHobgoblin II x5";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("1MN Afterburner II", 1, State::Active),
            item("Hobgoblin II", 5, State::Active),
        ]
    );
}

#[test]
fn onlines_a_cloak() {
    let eft = "[Rifter, Cloak]\n\n\nPrototype Cloaking Device I";
    assert_eq!(
        settle(eft, all(5)),
        [item("Prototype Cloaking Device I", 1, State::Online)]
    );
}

#[test]
fn activates_one_propulsion_module() {
    let eft = "[Rifter, Prop]\n\n5MN Microwarpdrive II\n1MN Afterburner II";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("5MN Microwarpdrive II", 1, State::Active),
            item("1MN Afterburner II", 1, State::Online),
        ]
    );
}

#[test]
fn launches_no_more_drones_than_allowed() {
    let eft = "[Vexor, Drones]\n\nHammerhead II x8";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("Hammerhead II", 5, State::Active),
            item("Hammerhead II", 3, State::Offline),
        ]
    );
}

#[test]
fn launches_the_most_damaging_drones_first() {
    let eft = "[Vexor, Drones]\n\nHobgoblin II x5\nHammerhead II x5";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("Hobgoblin II", 5, State::Offline),
            item("Hammerhead II", 5, State::Active),
        ]
    );
}

#[test]
fn fills_the_bandwidth_left_with_smaller_drones() {
    let eft = "[Vexor, Drones]\n\nOgre II x2\nHammerhead II x5\nHobgoblin II x5";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("Ogre II", 2, State::Active),
            item("Hammerhead II", 2, State::Active),
            item("Hammerhead II", 3, State::Offline),
            item("Hobgoblin II", 1, State::Active),
            item("Hobgoblin II", 4, State::Offline),
        ]
    );
}

#[test]
fn launches_no_drones_without_skills() {
    let eft = "[Vexor, Drones]\n\nHobgoblin II x5";
    assert_eq!(
        settle(eft, none()),
        [item("Hobgoblin II", 5, State::Offline)]
    );
}

#[test]
fn leaves_the_bay_drones_in_their_slot() {
    let mut fit = load("[Vexor, Drones]\n\nHammerhead II x8").unwrap();
    fit.character.skills = all(5).levels;
    post_load(&info(), &mut fit);

    assert!(fit.items.iter().all(|item| item.slot == Slot::DroneBay));
}

#[test]
fn onlines_one_of_a_group_with_an_online_limit() {
    let eft =
        "[Venture, Survey]\n\nBasic Mining Survey Chipset\nML-3 Compact Mining Survey Chipset";
    assert_eq!(
        settle(eft, all(5)),
        [
            item("Basic Mining Survey Chipset", 1, State::Active),
            item("ML-3 Compact Mining Survey Chipset", 1, State::Offline),
        ]
    );
}

#[test]
fn starts_in_the_first_mode() {
    let mut fit = load("[Confessor, Mode]").unwrap();
    let info = info();
    post_load(&info, &mut fit);

    let mode = fit.ship.mode.and_then(|mode| info.get_type(mode));
    assert_eq!(mode.map(|mode| mode.name()), Some("Confessor Defense Mode"));
}

#[test]
fn keeps_a_mode_already_set() {
    let mut fit = load("[Confessor, Mode]").unwrap();
    fit.ship.mode = Some(34321);
    post_load(&info(), &mut fit);

    assert_eq!(fit.ship.mode, Some(34321));
}

#[test]
fn fills_the_fighter_tubes() {
    let mut fit = load("[Thanatos, Fighters]\n\nFirbolg II x30\nDromi II x30").unwrap();
    fit.character.skills = all(5).levels;
    let info = info();
    post_load(&info, &mut fit);

    let items: Vec<_> = fit
        .items
        .iter()
        .map(|item| {
            let name = info.get_type(item.type_id).unwrap().name();
            (name, item.slot, item.quantity, item.state)
        })
        .collect();
    assert_eq!(
        items,
        [
            ("Firbolg II", Slot::FighterBay, 12, State::Offline),
            ("Dromi II", Slot::FighterBay, 27, State::Offline),
            ("Firbolg II", Slot::FighterTube(0), 6, State::Active),
            ("Firbolg II", Slot::FighterTube(1), 6, State::Active),
            ("Firbolg II", Slot::FighterTube(2), 6, State::Active),
            ("Dromi II", Slot::FighterTube(3), 3, State::Active),
        ]
    );
}
