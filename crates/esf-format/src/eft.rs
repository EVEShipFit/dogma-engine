//! EFT, the text format EVE copies a fit to the clipboard in.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use esf_data::{InfoName, eve};
use esf_dogma_engine::{
    Character, Charge, Environment, Fit, FitItem, Mutation, Projection, Ship, Slot, State,
};

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
    /// The type exists, but is not a module, implant or booster.
    NoSlot(String),
    /// A line refers to a mutation, like `[1]`, that no section describes.
    UnknownMutation(u32),
    /// A mutation is not `[<n>] <Base>`, `<Mutaplasmid>`, `<attribute> <value>, ...`;
    /// or it describes another item than the line referring to it; or that
    /// line is a stack of more than one.
    InvalidMutation(String),
    /// No attribute has this name.
    UnknownAttribute(String),
    /// A mutation leaves out an attribute its mutaplasmid rolls.
    MissingRoll {
        /// The first line of the mutation.
        mutation: String,
        /// The attribute left out.
        attribute_id: i32,
    },
    /// The mutaplasmid cannot mutate the item.
    NotMutable {
        /// The name of the item.
        item: String,
        /// The name of the mutaplasmid.
        mutaplasmid: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty => write!(f, "empty EFT"),
            Error::InvalidHeader => write!(f, "invalid EFT header"),
            Error::InvalidEmptySlot(line) => write!(f, "invalid empty slot {line}"),
            Error::UnknownType(name) => write!(f, "unknown type {name}"),
            Error::NoSlot(name) => write!(f, "{name} does not fit in any slot"),
            Error::UnknownMutation(reference) => write!(f, "unknown mutation [{reference}]"),
            Error::InvalidMutation(line) => write!(f, "invalid mutation {line}"),
            Error::UnknownAttribute(name) => write!(f, "unknown attribute {name}"),
            Error::MissingRoll {
                mutation,
                attribute_id,
            } => write!(f, "{mutation} has no roll for attribute {attribute_id}"),
            Error::NotMutable { item, mutaplasmid } => {
                write!(f, "{mutaplasmid} does not apply to {item}")
            }
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

/* Implants and boosters have no rack; the slot they go in is an attribute. */
const ATTRIBUTE_IMPLANTNESS: i32 = 331;
const ATTRIBUTE_BOOSTERNESS: i32 = 1087;

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
        .or_else(|| find_character_slot(info, type_id))
}

fn find_character_slot(info: &impl InfoName, type_id: i32) -> Option<Slot> {
    info.get_dogma_attributes(type_id)
        .into_iter()
        .flatten()
        .find_map(|attribute| {
            let index = attribute.value() as i64;
            match attribute.attribute_id() {
                ATTRIBUTE_IMPLANTNESS => Some(Slot::Implant(index.try_into().ok()?)),
                ATTRIBUTE_BOOSTERNESS => Some(Slot::Booster(index.try_into().ok()?)),
                _ => None,
            }
        })
}

fn type_name_to_id(info: &impl InfoName, name: &str) -> Result<i32, Error> {
    info.type_name_to_id(name)
        .ok_or_else(|| Error::UnknownType(name.to_string()))
}

/* Split "<Type Name> x<Quantity>" on its last token, as type names can contain an "x" too. */
fn parse_quantity(line: &str) -> Option<(&str, u32, Option<u32>)> {
    let (line, reference) = split_mutation_reference(line);
    let (type_name, quantity) = line.rsplit_once(char::is_whitespace)?;
    let quantity = quantity.strip_prefix('x')?.parse().ok()?;
    Some((type_name.trim(), quantity, reference))
}

/* A mutation section, as PyFa writes it, with its names looked up. */
struct MutationSection<'a> {
    header: &'a str,
    base: i32,
    mutaplasmid: i32,
    mutaplasmid_name: &'a str,
    attributes: BTreeMap<i32, f64>,
}

/* "[<n>] <Base Name>" */
fn parse_mutation_header(line: &str) -> Option<(u32, &str)> {
    let (reference, base_name) = line.trim().strip_prefix('[')?.split_once(']')?;
    Some((reference.parse().ok()?, base_name.trim()))
}

/* Split "<line> [<n>]" into the line and the mutation it refers to. */
fn split_mutation_reference(line: &str) -> (&str, Option<u32>) {
    let line = line.trim();
    let split = line
        .strip_suffix(']')
        .and_then(|rest| rest.rsplit_once('['))
        .and_then(|(rest, reference)| Some((rest.trim_end(), reference.parse().ok()?)));

    match split {
        Some((rest, reference)) => (rest, Some(reference)),
        None => (line, None),
    }
}

/* "<attribute> <value>, <attribute> <value>, ..." */
fn parse_mutation_attributes(
    info: &impl InfoName,
    line: &str,
) -> Result<BTreeMap<i32, f64>, Error> {
    let invalid = || Error::InvalidMutation(line.to_string());

    line.split(',')
        .map(|pair| {
            let (name, value) = pair
                .trim()
                .split_once(char::is_whitespace)
                .ok_or_else(invalid)?;
            let value = value.trim().parse().map_err(|_| invalid())?;
            let attribute_id = info
                .attribute_name_to_id(name)
                .ok_or_else(|| Error::UnknownAttribute(name.to_string()))?;
            Ok((attribute_id, value))
        })
        .collect()
}

/* A section holds one or more mutations, each a header, a mutaplasmid and
 * optionally the rolled attributes. */
fn parse_mutations<'a>(
    info: &impl InfoName,
    section: &[&'a str],
    mutations: &mut HashMap<u32, MutationSection<'a>>,
) -> Result<(), Error> {
    let mut lines = section.iter().copied().map(str::trim).peekable();

    while let Some(header) = lines.next() {
        let invalid = || Error::InvalidMutation(header.to_string());
        let (reference, base_name) = parse_mutation_header(header).ok_or_else(invalid)?;

        let mutaplasmid_name = lines
            .next_if(|line| parse_mutation_header(line).is_none())
            .ok_or_else(invalid)?;
        let attributes = match lines.next_if(|line| parse_mutation_header(line).is_none()) {
            Some(line) => parse_mutation_attributes(info, line)?,
            None => BTreeMap::new(),
        };

        mutations.insert(
            reference,
            MutationSection {
                header,
                base: type_name_to_id(info, base_name)?,
                mutaplasmid: type_name_to_id(info, mutaplasmid_name)?,
                mutaplasmid_name,
                attributes,
            },
        );
    }

    Ok(())
}

fn resulting_type_id(mutaplasmid: eve::Mutaplasmid, base: i32) -> Option<i32> {
    mutaplasmid
        .mappings()?
        .iter()
        .find(|mapping| {
            mapping
                .applicable_type_ids()
                .is_some_and(|type_ids| type_ids.iter().any(|type_id| type_id == base))
        })
        .map(|mapping| mapping.resulting_type_id())
}

/* An item that refers to a mutation becomes the type the mutation results in. */
fn mutate(
    info: &impl InfoName,
    mutations: &HashMap<u32, MutationSection>,
    reference: Option<u32>,
    type_name: &str,
    type_id: i32,
) -> Result<(i32, Option<Mutation>), Error> {
    let Some(reference) = reference else {
        return Ok((type_id, None));
    };

    let section = mutations
        .get(&reference)
        .ok_or(Error::UnknownMutation(reference))?;
    if section.base != type_id {
        return Err(Error::InvalidMutation(section.header.to_string()));
    }

    let mutaplasmid = info.get_mutaplasmid(section.mutaplasmid);
    let result = mutaplasmid
        .and_then(|mutaplasmid| resulting_type_id(mutaplasmid, type_id))
        .ok_or_else(|| Error::NotMutable {
            item: type_name.to_string(),
            mutaplasmid: section.mutaplasmid_name.to_string(),
        })?;

    let missing = mutaplasmid
        .and_then(|mutaplasmid| mutaplasmid.attributes())
        .into_iter()
        .flatten()
        .find(|attribute| !section.attributes.contains_key(&attribute.attribute_id()));
    if let Some(attribute) = missing {
        return Err(Error::MissingRoll {
            mutation: section.header.to_string(),
            attribute_id: attribute.attribute_id(),
        });
    }

    Ok((
        result,
        Some(Mutation {
            base: type_id,
            attributes: section.attributes.clone(),
        }),
    ))
}

/// Load a fit from EFT text. The fit has no skills.
///
/// Modules are active, unless the line ends in `/offline`. As an EVEShip.fit
/// extension, `/online`, `/active` and `/overload` work too. An implant or
/// booster goes in the slot its type is made for. A section where
/// every line ends in `x<quantity>` goes in the drone bay if it holds only
/// drones, in the fighter bay if it holds only fighters, and in the cargo hold
/// otherwise.
///
/// A mutated module or drone is written the way PyFa writes it: the line of
/// its base type ends in `[<n>]`, and a section describes each mutation as
/// `[<n>] <Base Type>`, then `<Mutaplasmid>`, then the rolled
/// `<attribute> <value>, ...`. Every attribute the mutaplasmid rolls must be
/// there.
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
    let (ship_type_name, name) = (ship_type_name.trim(), name.trim());

    let mut fit = Fit {
        name: Some(name.to_string()),
        ship: Ship {
            type_id: type_name_to_id(info, ship_type_name)?,
            mode: None,
        },
        items: Vec::new(),
        character: Character::default(),
        environment: Environment::default(),
        incoming: Projection::default(),
    };

    /* An EFT has sections, which are seperated by a new line. */
    let (mutation_sections, item_sections): (Vec<_>, Vec<_>) =
        section_iter(eft_lines).partition(|section| parse_mutation_header(section[0]).is_some());

    /* PyFa writes the mutations after the items that refer to them. */
    let mut mutations = HashMap::new();
    for section in &mutation_sections {
        parse_mutations(info, section, &mut mutations)?;
    }

    for section in item_sections {
        /* A quantity section only if every line ends with "x<quantity>". */
        let quantities: Option<Vec<_>> = section.iter().map(|line| parse_quantity(line)).collect();

        match quantities {
            None => {
                let mut rack_indexes: HashMap<i32, u8> = HashMap::new();

                for line in section {
                    let (mut line, reference) = split_mutation_reference(line);
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
                    let (type_id, mutation) =
                        mutate(info, &mutations, reference, module_name, module_type_id)?;
                    let charge_type_id = charge_name
                        .map(|charge_name| type_name_to_id(info, charge_name))
                        .transpose()?;

                    let Some(slot) = find_slot(info, type_id, &mut rack_indexes) else {
                        return Err(Error::NoSlot(module_name.to_string()));
                    };

                    fit.items.push(FitItem {
                        type_id,
                        slot,
                        quantity: 1,
                        state,
                        charge: charge_type_id.map(|type_id| Charge { type_id }),
                        mutation,
                        fighter_abilities: None,
                        booster_side_effects: BTreeSet::new(),
                        spool: None,
                    });
                }
            }
            Some(quantities) => {
                let mut items = Vec::new();

                let mut are_drones = true;
                let mut are_fighters = true;

                for (line, (type_name, quantity, reference)) in section.iter().zip(quantities) {
                    if reference.is_some() && quantity != 1 {
                        return Err(Error::InvalidMutation(line.trim().to_string()));
                    }

                    let base_type_id = type_name_to_id(info, type_name)?;
                    let (type_id, mutation) =
                        mutate(info, &mutations, reference, type_name, base_type_id)?;

                    let category_id = info.get_type(type_id).map(|r#type| r#type.category_id());
                    are_drones = are_drones && category_id == Some(CATEGORY_DRONE);
                    are_fighters = are_fighters && category_id == Some(CATEGORY_FIGHTER);

                    items.push((type_id, quantity, mutation));
                }

                let (slot, state) = match (are_drones, are_fighters) {
                    (true, _) => (Slot::DroneBay, State::Active),
                    (_, true) => (Slot::FighterBay, State::Offline),
                    _ => (Slot::Cargo, State::Offline),
                };

                for (type_id, quantity, mutation) in items {
                    fit.items.push(FitItem {
                        type_id,
                        slot,
                        quantity,
                        state,
                        charge: None,
                        mutation,
                        fighter_abilities: None,
                        booster_side_effects: BTreeSet::new(),
                        spool: None,
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

        fn get_dogma_attributes(
            &self,
            _type_id: i32,
        ) -> Option<Vector<'_, eve::TypeDogmaAttribute>> {
            None
        }

        fn get_type(&self, _type_id: i32) -> Option<eve::Type<'_>> {
            None
        }

        fn type_name_to_id(&self, name: &str) -> Option<i32> {
            match name {
                "Rifter" => Some(587),
                "200mm AutoCannon II" => Some(2881),
                "Warp Scrambler II" => Some(448),
                "Unstable Warp Scrambler Mutaplasmid" => Some(47730),
                "Hobgoblin II" => Some(2456),
                _ => None,
            }
        }

        fn attribute_name_to_id(&self, name: &str) -> Option<i32> {
            match name {
                "cpu" => Some(50),
                _ => None,
            }
        }

        /* Without mutaplasmids, nothing is mutable. */
        fn get_mutaplasmid(&self, _type_id: i32) -> Option<eve::Mutaplasmid<'_>> {
            None
        }
    }

    const MUTATION: &str =
        "[1] Warp Scrambler II\n  Unstable Warp Scrambler Mutaplasmid\n  cpu 30.5";

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
    fn header_is_trimmed() {
        let fit = load_eft(&Names, "[ Rifter , My Rifter ]").unwrap();

        assert_eq!(fit.ship.type_id, 587);
        assert_eq!(fit.name.as_deref(), Some("My Rifter"));
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

    #[test]
    fn splits_mutation_reference() {
        assert_eq!(
            split_mutation_reference("Warp Scrambler II /offline [1]"),
            ("Warp Scrambler II /offline", Some(1))
        );
        assert_eq!(
            split_mutation_reference("[Empty Med slot]"),
            ("[Empty Med slot]", None)
        );
    }

    #[test]
    fn unknown_mutation() {
        assert_eq!(
            error("[Rifter, Test]\nWarp Scrambler II [2]"),
            Error::UnknownMutation(2)
        );
    }

    #[test]
    fn mutation_without_mutaplasmid() {
        assert_eq!(
            error("[Rifter, Test]\n\n[1] Warp Scrambler II"),
            Error::InvalidMutation("[1] Warp Scrambler II".to_string())
        );
    }

    #[test]
    fn mutation_with_invalid_value() {
        assert_eq!(
            error(
                "[Rifter, Test]\n\n[1] Warp Scrambler II\nUnstable Warp Scrambler Mutaplasmid\ncpu fast"
            ),
            Error::InvalidMutation("cpu fast".to_string())
        );
    }

    #[test]
    fn mutation_with_unknown_attribute() {
        assert_eq!(
            error(
                "[Rifter, Test]\n\n[1] Warp Scrambler II\nUnstable Warp Scrambler Mutaplasmid\nspeed 5"
            ),
            Error::UnknownAttribute("speed".to_string())
        );
    }

    #[test]
    fn mutation_of_another_item() {
        assert_eq!(
            error(&format!(
                "[Rifter, Test]\n200mm AutoCannon II [1]\n\n{MUTATION}"
            )),
            Error::InvalidMutation("[1] Warp Scrambler II".to_string())
        );
    }

    #[test]
    fn mutaplasmid_not_applicable() {
        assert_eq!(
            error(&format!(
                "[Rifter, Test]\nWarp Scrambler II [1]\n\n{MUTATION}"
            )),
            Error::NotMutable {
                item: "Warp Scrambler II".to_string(),
                mutaplasmid: "Unstable Warp Scrambler Mutaplasmid".to_string(),
            }
        );
    }

    #[test]
    fn mutated_stack() {
        assert_eq!(
            error("[Rifter, Test]\n\nHobgoblin II x2 [1]"),
            Error::InvalidMutation("Hobgoblin II x2 [1]".to_string())
        );
    }
}
