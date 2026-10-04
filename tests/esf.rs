//! What esf/1 loads and saves, and how it maps onto the engine's fit.
//!
//! The rules of the format itself are tested by the esf/1 test suite.

use esf_dogma_engine::{Slot, State};
use esf_format::esf::{
    EsfFit, decode_base64url, encode_base64url, from_fit, load_esf, main_fit, save_esf,
    save_esf_binary, to_fit,
};

use crate::harness::{info, info_name};

const EVERYTHING: &str = "\
%esf/1
Hecate \"Everything\" /Sharpshooter

2x 150mm Light AutoCannon II :Barrage S
- @high
Small Energy Nosferatu II

5MN Y-T8 Compact Microwarpdrive +Unstable {capacitorNeed 45, cpu 21, power 14, signatureRadiusBonus 500, speedFactor 524}
- @mid
Warp Disruptor II

Damage Control II !off
Gyrostabilizer II !heat

2x Hobgoblin II @bay
3x Warrior II

1000x Barrage S
Damage Control II @cargo

100x Nanite Repair Paste @fuel

Zainou 'Gnome' Shield Management SM-703

Standard Blue Pill Booster
";

fn load(text: &str) -> Vec<EsfFit> {
    load_esf(&info(), text.as_bytes()).unwrap()
}

#[test]
fn round_trips_canonical_text() {
    assert_eq!(save_esf(&info(), &load(EVERYTHING)).unwrap(), EVERYTHING);
}

#[test]
fn round_trips_binary() {
    let fits = load(EVERYTHING);
    let binary = save_esf_binary(&info(), &fits).unwrap();
    let link = encode_base64url(&binary);

    let loaded = load_esf(&info(), &decode_base64url(&link).unwrap()).unwrap();
    assert_eq!(loaded, fits);
}

#[test]
fn names_the_line_that_is_wrong() {
    let error = load_esf(&info(), b"%esf/1\nRifter\nNo Such Module\n").unwrap_err();
    assert_eq!(error.to_string(), "line 3: unknown type \"No Such Module\"");
}

#[test]
fn calculates_what_the_engine_can_hold() {
    let fit = to_fit(&info(), &load(EVERYTHING)[0]).unwrap();

    let slots: Vec<(Slot, u32, State)> = fit
        .items
        .iter()
        .map(|item| (item.slot, item.quantity, item.state))
        .collect();
    assert_eq!(
        slots,
        [
            (Slot::High(0), 1, State::Active),
            (Slot::High(1), 1, State::Active),
            (Slot::High(3), 1, State::Active),
            (Slot::Medium(0), 1, State::Active),
            (Slot::Medium(2), 1, State::Active),
            (Slot::Low(0), 1, State::Offline),
            (Slot::Low(1), 1, State::Overload),
            (Slot::DroneBay, 2, State::Offline),
            (Slot::DroneBay, 3, State::Active),
            (Slot::Cargo, 1000, State::Offline),
            (Slot::Cargo, 1, State::Offline),
            (Slot::Cargo, 100, State::Offline),
            (Slot::Implant(7), 1, State::Online),
            (Slot::Booster(1), 1, State::Online),
        ]
    );
    assert!(fit.ship.mode.is_some());
    assert_eq!(
        fit.items[0].charge.as_ref().map(|charge| charge.type_id),
        Some(12625)
    );

    let mutated = &fit.items[3];
    assert_ne!(
        Some(mutated.type_id),
        mutated.mutation.as_ref().map(|mutation| mutation.base)
    );
    assert_eq!(mutated.mutation.as_ref().unwrap().attributes.len(), 5);
}

/* The engine has no fuel bay, so that comes back as cargo. Nor does it keep
 * the mutaplasmid, so the narrowest one that rolls these values comes back. */
#[test]
fn writes_back_what_the_engine_holds() {
    let fit = to_fit(&info(), &load(EVERYTHING)[0]).unwrap();
    let esf = from_fit(&info(), &fit).unwrap();

    assert_eq!(
        save_esf(&info(), &[esf]).unwrap(),
        EVERYTHING
            .replace(
                "\n100x Nanite Repair Paste @fuel\n",
                "100x Nanite Repair Paste\n"
            )
            .replace("+Unstable", "+Glorified Decayed")
    );
}

#[test]
fn writes_a_fit_loaded_from_eft() {
    let eft = "\
[Rifter, From EFT]
Damage Control II
Gyrostabilizer II /offline

1MN Afterburner II

[Empty High slot]
200mm AutoCannon II, Republic Fleet EMP S

Warrior II x3

Republic Fleet EMP S x1000
";
    let fit = esf_format::eft::load_eft(&info_name(), eft).unwrap();
    let esf = from_fit(&info(), &fit).unwrap();

    assert_eq!(
        save_esf(&info(), &[esf]).unwrap(),
        "\
%esf/1
Rifter \"From EFT\"

- @high
200mm AutoCannon II :Republic Fleet EMP S

1MN Afterburner II

Damage Control II
Gyrostabilizer II !off

3x Warrior II

1000x Republic Fleet EMP S
"
    );
}

#[test]
fn cannot_calculate_a_fit_without_a_ship() {
    let fits = load("%esf/1\n- \"Contract\"\nDamage Control II\n");
    assert!(to_fit(&info(), &fits[0]).is_err());
}

#[test]
fn the_main_fit_is_the_one_not_carried() {
    let fits = load("%esf/1\nHeron \"Scout\"\n\n%esf/1\nRaven\nHeron \"Scout\" @frigate\n");
    assert_eq!(main_fit(&fits).unwrap().hull, fits[1].hull);
}
