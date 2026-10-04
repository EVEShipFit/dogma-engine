//! DNA, the fits EVE links to in chat as `fitting:<dna>`.

use std::collections::HashMap;
use std::fmt;

use esf_data::Info;
use esf_dogma_engine::{Fit, Slot};

use crate::flags::{EFFECT_RACKS, Place};
use crate::listed::{ByInfo, Listed, fit, to_fit_items};

/// Why a DNA could not be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// No ship is given.
    NoShip,
    /// A type id or quantity is not a number.
    InvalidNumber(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoShip => write!(f, "the DNA has no ship"),
            Error::InvalidNumber(field) => write!(f, "{field} is not a number"),
        }
    }
}

impl std::error::Error for Error {}

const CATEGORY_DRONE: i32 = 18;
const CATEGORY_FIGHTER: i32 = 87;

const SLOTS_PER_RACK: u8 = 8;

fn number<T: std::str::FromStr>(field: &str) -> Result<T, Error> {
    field
        .parse()
        .map_err(|_| Error::InvalidNumber(field.to_string()))
}

/// Load a fit from a DNA, with or without its `fitting:` prefix. The fit has
/// no skills and no name.
///
/// A DNA is `<ship>:<type>;<quantity>:...::`; a type ending in `_` is in the
/// cargo. A module fills the next free slots of its rack, up to eight; drones
/// and fighters go in their bay, and anything else in the cargo, charges and
/// implants too. A DNA without its closing `::` was cut short, and its last
/// item is left out, as are types the SDE does not know.
pub fn load_dna(info: &impl Info, dna: &str) -> Result<Fit, Error> {
    let dna = dna.trim();
    let dna = dna.strip_prefix("fitting:").unwrap_or(dna);
    let dna = match dna.strip_suffix("::") {
        Some(dna) => dna,
        None => &dna[..dna.rfind(':').unwrap_or(0)],
    };

    let mut ship_type_id = None;
    let mut rack_indexes: HashMap<i32, u8> = HashMap::new();
    let mut listed = Vec::new();

    for piece in dna.split(':').filter(|piece| !piece.is_empty()) {
        let Some((type_id, quantity)) = piece.split_once(';') else {
            ship_type_id = Some(number(piece)?);
            continue;
        };
        let (type_id, in_cargo) = match type_id.strip_suffix('_') {
            Some(type_id) => (type_id, true),
            None => (type_id, false),
        };
        let type_id: i32 = number(type_id)?;
        let quantity: u32 = number(quantity)?;

        let Some(r#type) = info.get_type(type_id) else {
            continue;
        };

        let rack = info
            .get_dogma_effects(type_id)
            .into_iter()
            .flatten()
            .find_map(|effect| {
                EFFECT_RACKS
                    .iter()
                    .find(|(rack, _)| *rack == effect.effect_id())
            });

        match rack {
            Some((rack, slot)) if !in_cargo => {
                let index = rack_indexes.entry(*rack).or_insert(0);
                for _ in 0..quantity {
                    if *index >= SLOTS_PER_RACK {
                        break;
                    }
                    listed.push(Listed::new(Place::Slot(slot(*index)), type_id, 1));
                    *index += 1;
                }
            }
            _ => {
                let slot = match r#type.category_id() {
                    CATEGORY_DRONE if !in_cargo => Slot::DroneBay,
                    CATEGORY_FIGHTER if !in_cargo => Slot::FighterBay,
                    _ => Slot::Cargo,
                };
                listed.push(Listed::new(Place::Slot(slot), type_id, quantity));
            }
        }
    }

    let ship_type_id = ship_type_id.ok_or(Error::NoShip)?;
    Ok(fit(None, ship_type_id, to_fit_items(&ByInfo(info), listed)))
}
