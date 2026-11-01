//! What an EFT export holds, and what it drops.
//!
//! A fit that came from an EFT as EVE writes it has to come back out as the
//! same text.

use esf_dogma_engine::{Slot, State};
use esf_format::eft::Error;

use crate::harness::{load_eft, save_eft};

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
    let fit = load_eft(eft).unwrap();
    assert_eq!(save_eft(&fit).unwrap(), eft);
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
    let mut fit = load_eft("[Rifter, My Rifter]\n200mm AutoCannon II").unwrap();
    fit.name = None;

    assert_eq!(
        save_eft(&fit).unwrap(),
        "[Rifter, Rifter]\n200mm AutoCannon II\n"
    );
}

/* A rig can never be more than online, so it is written without a state, even
 * though the calculation lowers it to one. */
#[test]
fn only_writes_a_state_a_module_can_reach() {
    let eft = "[Rifter, States]\n\n\n\nSmall Projectile Ambit Extension I\n";
    let mut fit = load_eft(eft).unwrap();

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
        assert_eq!(
            save_eft(&fit).unwrap(),
            format!("[Rifter, States]\n{expected}")
        );
    }
}

/* EVE skips a line with any state but "/offline". */
#[test]
fn writes_no_state_but_offline() {
    let eft = "[Rifter, States]\n1MN Afterburner II\n";
    let mut fit = load_eft(eft).unwrap();

    for state in [State::Online, State::Overload] {
        fit.items[0].state = state;
        assert_eq!(save_eft(&fit).unwrap(), eft);
    }
}

/* EVE writes no line for an empty subsystem slot. */
#[test]
fn writes_no_empty_subsystem_slot() {
    let mut fit = load_eft(EVERYTHING).unwrap();
    fit.items.retain(|item| item.slot != Slot::Subsystem(0));

    assert_eq!(
        save_eft(&fit).unwrap(),
        EVERYTHING.replace("Loki Defensive - Augmented Durability\n", "")
    );
}

const MUTATIONS: &str = "\
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

#[test]
fn reads_a_stack_between_modules() {
    let fit = load_eft("[Tristan, Stacks]\nDamage Control II\nHobgoblin II x1\n").unwrap();

    assert_eq!(fit.items[0].slot, Slot::Low(0));
    assert_eq!(fit.items[1].slot, Slot::DroneBay);
    assert_eq!(fit.items[1].quantity, 1);
}

/* EVE cannot import a mutation, so it is written as the type it mutated. */
#[test]
fn writes_a_mutated_item_as_its_base_type() {
    let fit = load_eft(MUTATIONS).unwrap();

    assert_eq!(
        save_eft(&fit).unwrap(),
        "[Tristan, Mutations]\nWarp Scrambler II\n\nHobgoblin II x1\nHobgoblin II x2\n"
    );
}

#[test]
fn rejects_a_mutation_missing_a_roll() {
    let eft = MUTATIONS.replace("capacitorNeed 7.5, ", "");
    assert_eq!(
        load_eft(&eft).unwrap_err(),
        Error::MissingRoll {
            mutation: "[1] Warp Scrambler II".to_string(),
            attribute_id: 6,
        }
    );
}

/* EFT has no tube, so a launched squadron reads back as one in the bay. */
#[test]
fn writes_a_fighter_in_a_tube_as_one_in_the_bay() {
    let mut fit = load_eft("[Thanatos, Fighters]\n\nTemplar II x6\n").unwrap();
    fit.items[0].slot = Slot::FighterTube(0);

    let again = load_eft(&save_eft(&fit).unwrap()).unwrap();
    assert_eq!(again.items[0].slot, Slot::FighterBay);
    assert_eq!(again.items[0].quantity, 6);
}

#[test]
fn explains_each_error() {
    let cases = [
        (Error::Empty, "empty EFT"),
        (Error::InvalidHeader, "invalid EFT header"),
        (
            Error::InvalidEmptySlot("[Empty Odd slot]".to_string()),
            "invalid empty slot [Empty Odd slot]",
        ),
        (Error::UnknownType("Odd".to_string()), "unknown type Odd"),
        (Error::UnknownTypeId(1), "unknown type id 1"),
        (
            Error::NoSlot("Odd".to_string()),
            "Odd does not fit in any slot",
        ),
        (Error::UnknownMutation(3), "unknown mutation [3]"),
        (
            Error::InvalidMutation("[1]".to_string()),
            "invalid mutation [1]",
        ),
        (
            Error::UnknownAttribute("odd".to_string()),
            "unknown attribute odd",
        ),
        (
            Error::MissingRoll {
                mutation: "[1] Warp Scrambler II".to_string(),
                attribute_id: 20,
            },
            "[1] Warp Scrambler II has no roll for attribute 20",
        ),
        (
            Error::NotMutable {
                item: "Rifter".to_string(),
                mutaplasmid: "Odd Mutaplasmid".to_string(),
            },
            "Odd Mutaplasmid does not apply to Rifter",
        ),
    ];

    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}
