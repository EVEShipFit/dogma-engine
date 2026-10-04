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
