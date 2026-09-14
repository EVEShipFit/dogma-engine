//! EFT, the text format EVE copies a fit to the clipboard in.

use std::collections::HashMap;
use std::fmt;

use esf_data::InfoName;
use esf_dogma_engine::{Character, Charge, Fit, FitItem, Ship, Slot, State};

/// Why an EFT could not be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The EFT has no lines at all.
    Empty,
    /// The first line is not `[<Ship Type>, <Fit Name>]`.
    InvalidHeader,
    /// A line starts with `[Empty` but names no known slot.
    InvalidEmptySlot(String),
    /// No type has this name.
    UnknownType(String),
    /// The type exists, but is not a module that fits in any slot.
    NoSlot(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty => write!(f, "empty EFT"),
            Error::InvalidHeader => write!(f, "invalid EFT header"),
            Error::InvalidEmptySlot(line) => write!(f, "invalid empty slot {line}"),
            Error::UnknownType(name) => write!(f, "unknown type {name}"),
            Error::NoSlot(name) => write!(f, "module {name} does not fit in any slot"),
        }
    }
}

impl std::error::Error for Error {}

/* The effect that marks which rack a module fits in. */
const EFFECT_LO_POWER: i32 = 11;
const EFFECT_HI_POWER: i32 = 12;
const EFFECT_MED_POWER: i32 = 13;
const EFFECT_RIG_SLOT: i32 = 2663;
const EFFECT_SUBSYSTEM: i32 = 3772;
const EFFECT_SERVICE_SLOT: i32 = 6306;

type Rack = fn(u8) -> Slot;

const RACKS: [(i32, Rack); 6] = [
    (EFFECT_LO_POWER, Slot::Low),
    (EFFECT_HI_POWER, Slot::High),
    (EFFECT_MED_POWER, Slot::Medium),
    (EFFECT_RIG_SLOT, Slot::Rig),
    (EFFECT_SUBSYSTEM, Slot::Subsystem),
    (EFFECT_SERVICE_SLOT, Slot::Service),
];

const CATEGORY_DRONE: i32 = 18;
const CATEGORY_FIGHTER: i32 = 87;

fn section_iter(eft_lines: Vec<&str>) -> impl Iterator<Item = Vec<&str>> {
    let mut section: Vec<&str> = Vec::new();

    let mut eft_lines = eft_lines
        .into_iter()
        .skip(1)
        .fold(Vec::new(), |mut sections, line| {
            if line.is_empty() {
                if !section.is_empty() {
                    sections.push(section.clone());
                    section.clear();
                }
            } else {
                section.push(line);
            }

            sections
        });

    if !section.is_empty() {
        eft_lines.push(section);
    }

    eft_lines.into_iter()
}

fn next_index(rack_indexes: &mut HashMap<i32, u8>, rack: i32) -> u8 {
    let index = rack_indexes.entry(rack).or_insert(0);
    *index += 1;
    *index - 1
}

fn find_slot(
    info: &impl InfoName,
    type_id: i32,
    rack_indexes: &mut HashMap<i32, u8>,
) -> Option<Slot> {
    info.get_dogma_effects(type_id)
        .into_iter()
        .flatten()
        .find_map(|effect| {
            let (rack, slot) = RACKS.iter().find(|(rack, _)| *rack == effect.effect_id())?;
            Some(slot(next_index(rack_indexes, *rack)))
        })
}

fn type_name_to_id(info: &impl InfoName, name: &str) -> Result<i32, Error> {
    info.type_name_to_id(name)
        .ok_or_else(|| Error::UnknownType(name.to_string()))
}

/* Split "<Type Name> x<Quantity>" on its last token, as type names can contain an "x" too. */
fn parse_quantity(line: &str) -> Option<(&str, u32)> {
    let (type_name, quantity) = line.trim().rsplit_once(char::is_whitespace)?;
    let quantity = quantity.strip_prefix('x')?.parse().ok()?;
    Some((type_name.trim(), quantity))
}

/// Load a fit from EFT text. The fit has no skills.
///
/// Modules are active, unless the line ends in `/offline`. As an EVEShip.fit
/// extension, `/online`, `/active` and `/overload` work too. A section where
/// every line ends in `x<quantity>` goes in the drone bay if it holds only
/// drones, in the fighter bay if it holds only fighters, and in the cargo hold
/// otherwise.
pub fn load_eft(info: &impl InfoName, eft: &str) -> Result<Fit, Error> {
    let eft_lines: Vec<&str> = eft.lines().collect();

    /* First line of an EFT always start with "[ship-type,name]". */
    let Some(header) = eft_lines.first() else {
        return Err(Error::Empty);
    };
    if !header.starts_with("[") || !header.ends_with("]") {
        return Err(Error::InvalidHeader);
    }
    let header = header.trim_start_matches("[").trim_end_matches("]");

    let Some((ship_type_name, name)) = header.split_once(",") else {
        return Err(Error::InvalidHeader);
    };

    let mut fit = Fit {
        name: Some(name.to_string()),
        ship: Ship {
            type_id: type_name_to_id(info, ship_type_name)?,
            mode: None,
        },
        items: Vec::new(),
        character: Character::default(),
    };

    /* An EFT has sections, which are seperated by a new line. */
    for section in section_iter(eft_lines) {
        /* A quantity section only if every line ends with "x<quantity>". */
        let quantities: Option<Vec<_>> = section.iter().map(|line| parse_quantity(line)).collect();

        match quantities {
            None => {
                let mut rack_indexes: HashMap<i32, u8> = HashMap::new();

                for line in section {
                    let mut line = line.trim();
                    let mut state = State::Active;

                    if line.starts_with("[Empty") {
                        let rack = match line {
                            "[Empty High slot]" => EFFECT_HI_POWER,
                            "[Empty Med slot]" => EFFECT_MED_POWER,
                            "[Empty Low slot]" => EFFECT_LO_POWER,
                            "[Empty Rig slot]" => EFFECT_RIG_SLOT,
                            "[Empty Subsystem slot]" => EFFECT_SUBSYSTEM,
                            _ => return Err(Error::InvalidEmptySlot(line.to_string())),
                        };

                        next_index(&mut rack_indexes, rack);
                        continue;
                    }

                    /* EVE only writes "/offline"; the other three are an
                     * EVEShip.fit extension, so a fit can pin the state of a
                     * single module. */
                    if let Some(position) = line.rfind('/') {
                        let suffix = match &line[position..] {
                            "/offline" => Some(State::Offline),
                            "/online" => Some(State::Online),
                            "/active" => Some(State::Active),
                            "/overload" => Some(State::Overload),
                            _ => None,
                        };

                        if let Some(suffix) = suffix {
                            state = suffix;
                            line = line[..position].trim_end();
                        }
                    }

                    /* Can either be "<Module Name>" or "<Module Name>, <Charge Name>". */
                    let comma_pos = line.find(",");

                    let (module_name, charge_name) = match comma_pos {
                        Some(comma_pos) => {
                            let module_name = line[..comma_pos].trim();
                            let charge_name = line[comma_pos + 1..].trim();
                            (module_name, Some(charge_name))
                        }
                        None => {
                            let module_name = line.trim();
                            (module_name, None)
                        }
                    };

                    let module_type_id = type_name_to_id(info, module_name)?;
                    let charge_type_id = charge_name
                        .map(|charge_name| type_name_to_id(info, charge_name))
                        .transpose()?;

                    let Some(slot) = find_slot(info, module_type_id, &mut rack_indexes) else {
                        return Err(Error::NoSlot(module_name.to_string()));
                    };

                    fit.items.push(FitItem {
                        type_id: module_type_id,
                        slot,
                        quantity: 1,
                        state,
                        charge: charge_type_id.map(|type_id| Charge { type_id }),
                        fighter_abilities: None,
                    });
                }
            }
            Some(quantities) => {
                let mut items = Vec::new();

                let mut are_drones = true;
                let mut are_fighters = true;

                for (type_name, quantity) in quantities {
                    let type_id = type_name_to_id(info, type_name)?;

                    let category_id = info.get_type(type_id).map(|r#type| r#type.category_id());
                    are_drones = are_drones && category_id == Some(CATEGORY_DRONE);
                    are_fighters = are_fighters && category_id == Some(CATEGORY_FIGHTER);

                    items.push((type_id, quantity));
                }

                let (slot, state) = match (are_drones, are_fighters) {
                    (true, _) => (Slot::DroneBay, State::Active),
                    (_, true) => (Slot::FighterBay, State::Offline),
                    _ => (Slot::Cargo, State::Offline),
                };

                for (type_id, quantity) in items {
                    fit.items.push(FitItem {
                        type_id,
                        slot,
                        quantity,
                        state,
                        charge: None,
                        fighter_abilities: None,
                    });
                }
            }
        }
    }

    Ok(fit)
}

#[cfg(test)]
mod tests {
    use super::*;

    use esf_data::eve;
    use esf_data::flatbuffers::Vector;

    /* Knows type names only, which is enough to fail before any slot lookup. */
    struct Names;

    impl InfoName for Names {
        fn get_dogma_effects(&self, _type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>> {
            None
        }

        fn get_type(&self, _type_id: i32) -> Option<eve::Type<'_>> {
            None
        }

        fn type_name_to_id(&self, name: &str) -> Option<i32> {
            match name {
                "Rifter" => Some(587),
                "200mm AutoCannon II" => Some(2881),
                _ => None,
            }
        }
    }

    fn error(eft: &str) -> Error {
        load_eft(&Names, eft).unwrap_err()
    }

    #[test]
    fn empty_input() {
        assert_eq!(error(""), Error::Empty);
    }

    #[test]
    fn header_without_comma() {
        assert_eq!(error("[Rifter]"), Error::InvalidHeader);
    }

    #[test]
    fn unknown_empty_slot() {
        assert_eq!(
            error("[Rifter, Test]\n[Empty Hangar slot]"),
            Error::InvalidEmptySlot("[Empty Hangar slot]".to_string())
        );
    }

    #[test]
    fn unknown_ship() {
        assert_eq!(
            error("[Shuttle, Test]"),
            Error::UnknownType("Shuttle".to_string())
        );
    }

    #[test]
    fn unknown_module() {
        assert_eq!(
            error("[Rifter, Test]\nNot A Module"),
            Error::UnknownType("Not A Module".to_string())
        );
    }

    #[test]
    fn unknown_charge() {
        assert_eq!(
            error("[Rifter, Test]\n200mm AutoCannon II, Not A Charge"),
            Error::UnknownType("Not A Charge".to_string())
        );
    }

    #[test]
    fn unknown_quantity_item() {
        assert_eq!(
            error("[Rifter, Test]\n\nNot A Drone x5"),
            Error::UnknownType("Not A Drone".to_string())
        );
    }
}
