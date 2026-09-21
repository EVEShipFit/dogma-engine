//! What an EFT export holds, and what it drops.
//!
//! A fit that came from an EFT has to come back out as the same text; the
//! cases below each add one thing EFT writes in its own way.

use std::collections::BTreeMap;

use esf_dogma_engine::{Fit, Mutation, Slot, State};

use crate::harness::{load, save};

/* Every rack, with a gap in one of them, a charge, a module that is off, and
 * a section for drones, for cargo and for what the character carries. */
const EVERYTHING: &str = "\
[Loki, Everything]
Damage Control II
[Empty Low slot]
Gyrostabilizer II /offline

Large Shield Extender II
5MN Y-T8 Compact Microwarpdrive
[Empty Med slot]
Warp Disruptor II

[Empty High slot]
720mm Howitzer Artillery II, Republic Fleet EMP M

Medium Projectile Ambit Extension II

Loki Defensive - Augmented Durability
Loki Offensive - Projectile Scoping Array
Loki Propulsion - Interdiction Nullifier
Loki Core - Augmented Nuclear Reactor

Warrior II x5

Republic Fleet EMP M x1000
Nanite Repair Paste x50

Ocular Filter - Basic
Standard Blue Pill Booster
";

/* A structure has services, and fighters instead of drones. */
const STRUCTURE: &str = "\
[Astrahus, Services]
Standup Ballistic Control System I

Standup Focused Warp Disruptor I

Standup Guided Bomb Launcher I, Focused Void Bomb

Standup M-Set Structure Target Multiplexing I

Standup Cloning Center I
Standup Market Hub I

Standup Einherji I x9
";

fn round_trip(eft: &str) {
    let fit = load(eft).unwrap();
    assert_eq!(save(&fit).unwrap(), eft);
}

#[test]
fn round_trips_every_section() {
    round_trip(EVERYTHING);
}

#[test]
fn round_trips_a_structure() {
    round_trip(STRUCTURE);
}

#[test]
fn names_the_fit_after_the_ship_when_it_has_no_name() {
    let mut fit = load("[Rifter, My Rifter]\n200mm AutoCannon II").unwrap();
    fit.name = None;

    assert_eq!(
        save(&fit).unwrap(),
        "[Rifter, Rifter]\n200mm AutoCannon II\n"
    );
}

/* A rig can never be more than online, so it is written without a state, even
 * though the calculation lowers it to one. */
#[test]
fn only_writes_a_state_a_module_can_reach() {
    let eft = "[Rifter, States]\n\n\n\nSmall Projectile Ambit Extension I\n";
    let mut fit = load(eft).unwrap();

    for state in [
        State::Offline,
        State::Online,
        State::Active,
        State::Overload,
    ] {
        fit.items[0].state = state;
        let expected = match state {
            State::Offline => "Small Projectile Ambit Extension I /offline\n",
            _ => "Small Projectile Ambit Extension I\n",
        };
        assert_eq!(save(&fit).unwrap(), format!("[Rifter, States]\n{expected}"));
    }
}

#[test]
fn writes_a_state_a_module_is_not_in_by_itself() {
    let eft = "[Rifter, States]\n\n1MN Afterburner II\n";
    let mut fit = load(eft).unwrap();

    fit.items[0].state = State::Online;
    assert_eq!(
        save(&fit).unwrap(),
        "[Rifter, States]\n1MN Afterburner II /online\n"
    );

    fit.items[0].state = State::Overload;
    assert_eq!(
        save(&fit).unwrap(),
        "[Rifter, States]\n1MN Afterburner II /overload\n"
    );
}

/* Which of the mutaplasmids of an item was used cannot be told from the fit,
 * so the export answers one that could have rolled these values. */
#[test]
fn writes_a_mutation_that_rolls_the_same_values() {
    let eft = "\
[Tristan, Mutations]

Warp Scrambler II [1]

Hobgoblin II x1 [2]
Hobgoblin II x2

[1] Warp Scrambler II
  Unstable Warp Scrambler Mutaplasmid
  capacitorNeed 7.5, cpu 30, maxRange 10500
[2] Hobgoblin II
  Exigent Light Drone Firepower Mutaplasmid
  armorHP 110, damageMultiplier 2.2, falloff 2100, hp 250, maxRange 2200, maxVelocity 3500, shieldCapacity 62, trackingSpeed 2.3
";

    let fit = load(eft).unwrap();
    let again = load(&save(&fit).unwrap()).unwrap();

    assert_eq!(rolls(&again), rolls(&fit));
}

/* Mutations do not compare, so the type and what it rolled stand in for them. */
fn rolls(fit: &Fit) -> Vec<String> {
    fit.items
        .iter()
        .map(|item| format!("{} {:?}", item.type_id, item.mutation))
        .collect()
}

/* Nothing names a mutaplasmid that rolls this, so the mutation is dropped and
 * the item written as what it mutated into. */
#[test]
fn drops_a_mutation_without_a_mutaplasmid() {
    let mut fit = load("[Rifter, Mutations]\n\nWarp Scrambler II\n").unwrap();
    fit.items[0].mutation = Some(Mutation {
        base: 587,
        attributes: BTreeMap::new(),
    });

    assert_eq!(
        save(&fit).unwrap(),
        "[Rifter, Mutations]\nWarp Scrambler II\n"
    );
}

/* EFT has no tube, so a launched squadron reads back as one in the bay. */
#[test]
fn writes_a_fighter_in_a_tube_as_one_in_the_bay() {
    let mut fit = load("[Thanatos, Fighters]\n\nTemplar II x6\n").unwrap();
    fit.items[0].slot = Slot::FighterTube(0);

    let again = load(&save(&fit).unwrap()).unwrap();
    assert_eq!(again.items[0].slot, Slot::FighterBay);
    assert_eq!(again.items[0].quantity, 6);
}
