//! EVE's inventory flags: where an item is, by name (ESI fittings).

use esf_dogma_engine::Slot;

/// Where an item is, as far as its flag says. Implants and boosters get their
/// slot from their type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Place {
    Slot(Slot),
    Implant,
    Booster,
}

/* A run of flags, one per position in the rack. */
struct Rack {
    name: &'static str,
    count: u8,
    slot: fn(u8) -> Slot,
}

const RACKS: [Rack; 7] = [
    Rack {
        name: "LoSlot",
        count: 8,
        slot: Slot::Low,
    },
    Rack {
        name: "MedSlot",
        count: 8,
        slot: Slot::Medium,
    },
    Rack {
        name: "HiSlot",
        count: 8,
        slot: Slot::High,
    },
    Rack {
        name: "RigSlot",
        count: 8,
        slot: Slot::Rig,
    },
    Rack {
        name: "SubSystemSlot",
        count: 8,
        slot: Slot::Subsystem,
    },
    Rack {
        name: "FighterTube",
        count: 5,
        slot: Slot::FighterTube,
    },
    Rack {
        name: "ServiceSlot",
        count: 8,
        slot: Slot::Service,
    },
];

const BAYS: [(&str, Place); 5] = [
    ("Cargo", Place::Slot(Slot::Cargo)),
    ("DroneBay", Place::Slot(Slot::DroneBay)),
    ("Booster", Place::Booster),
    ("Implant", Place::Implant),
    ("FighterBay", Place::Slot(Slot::FighterBay)),
];

/// By name, like `HiSlot0` or `DroneBay`.
pub(crate) fn place_of_flag_name(name: &str) -> Option<Place> {
    let rack = name.find(|c: char| c.is_ascii_digit()).and_then(|split| {
        let rack = RACKS.iter().find(|rack| rack.name == &name[..split])?;
        let index: u8 = name[split..].parse().ok()?;
        (index < rack.count).then(|| (rack.slot)(index))
    });
    if let Some(slot) = rack {
        return Some(Place::Slot(slot));
    }

    BAYS.iter()
        .find(|(bay, _)| *bay == name)
        .map(|(_, place)| *place)
}

/// The name of the flag a slot is, like `HiSlot0`. Implants and boosters are
/// named by their bay, as the flag does not hold the slot.
pub(crate) fn flag_name(slot: Slot) -> String {
    let place = match slot {
        Slot::Implant(_) => Place::Implant,
        Slot::Booster(_) => Place::Booster,
        slot => Place::Slot(slot),
    };

    if let Some(index) = rack_index(slot) {
        /* Which rack a slot is in, by building the same slot from each. */
        if let Some(rack) = RACKS.iter().find(|rack| (rack.slot)(index) == slot) {
            return format!("{}{index}", rack.name);
        }
    }

    BAYS.iter()
        .find(|(_, bay)| *bay == place)
        .map(|(name, _)| name.to_string())
        .expect("every slot is a rack or a bay")
}

fn rack_index(slot: Slot) -> Option<u8> {
    match slot {
        Slot::High(index)
        | Slot::Medium(index)
        | Slot::Low(index)
        | Slot::Rig(index)
        | Slot::Subsystem(index)
        | Slot::Service(index)
        | Slot::FighterTube(index) => Some(index),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_name() {
        assert_eq!(
            place_of_flag_name("HiSlot0"),
            Some(Place::Slot(Slot::High(0)))
        );
        assert_eq!(
            place_of_flag_name("SubSystemSlot3"),
            Some(Place::Slot(Slot::Subsystem(3)))
        );
        assert_eq!(place_of_flag_name("Cargo"), Some(Place::Slot(Slot::Cargo)));
        assert_eq!(place_of_flag_name("HiSlot8"), None);
        assert_eq!(place_of_flag_name("HiSlot"), None);
        assert_eq!(place_of_flag_name("Cargo0"), None);
        assert_eq!(place_of_flag_name("FuelBay"), None);
    }

    #[test]
    fn names_round_trip() {
        for slot in [
            Slot::Low(3),
            Slot::Medium(0),
            Slot::High(7),
            Slot::Rig(2),
            Slot::Subsystem(1),
            Slot::Service(4),
            Slot::FighterTube(2),
            Slot::FighterBay,
            Slot::DroneBay,
            Slot::Cargo,
        ] {
            assert_eq!(
                place_of_flag_name(&flag_name(slot)),
                Some(Place::Slot(slot))
            );
        }
        assert_eq!(flag_name(Slot::Implant(7)), "Implant");
        assert_eq!(flag_name(Slot::Booster(2)), "Booster");
    }
}
