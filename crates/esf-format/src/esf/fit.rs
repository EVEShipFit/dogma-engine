//! Turning esf/1 fits into fits the dogma engine calculates, and back.

use std::collections::BTreeSet;

use esf_data::{InfoEsf, fold_case};
use esf_dogma_engine::{
    Character, Charge, Environment, Fit, FitItem, Mutation, Projection, Ship, Slot,
    State as EngineState,
};

use super::canonicalise;
use super::lookup::{Kind, Lookup};
use super::model::{Entry, Error, EsfFit, Location, State};
use super::resolve::rack_of;

const ATTRIBUTE_IMPLANTNESS: i32 = 331;
const ATTRIBUTE_BOOSTERNESS: i32 = 1087;

fn rack_slot(rack: Location, index: u8) -> Slot {
    match rack {
        Location::High => Slot::High(index),
        Location::Mid => Slot::Medium(index),
        Location::Low => Slot::Low(index),
        Location::Rig => Slot::Rig(index),
        Location::Sub => Slot::Subsystem(index),
        _ => Slot::Service(index),
    }
}

fn slot_rack(slot: Slot) -> Option<(Location, u8)> {
    match slot {
        Slot::High(index) => Some((Location::High, index)),
        Slot::Medium(index) => Some((Location::Mid, index)),
        Slot::Low(index) => Some((Location::Low, index)),
        Slot::Rig(index) => Some((Location::Rig, index)),
        Slot::Subsystem(index) => Some((Location::Sub, index)),
        Slot::Service(index) => Some((Location::Svc, index)),
        _ => None,
    }
}

fn next_index(counters: &mut [u32; 6], rack: Location, count: u32) -> Result<u8, Error> {
    let counter = &mut counters[rack.number() as usize - 1];
    let index = u8::try_from(*counter)
        .map_err(|_| Error::new(format!("too many {} slots", rack.name())))?;
    *counter += count;
    Ok(index)
}

/// The fit a document is about: the first one no other fit carries.
pub fn main_fit(fits: &[EsfFit]) -> Option<&EsfFit> {
    let carried = |fit: &EsfFit| {
        fits.iter().flat_map(|other| &other.entries).any(|entry| {
            entry.type_id == fit.hull
                && entry
                    .fit_name
                    .as_deref()
                    .zip(fit.name.as_deref())
                    .is_some_and(|(left, right)| fold_case(left) == fold_case(right))
        })
    };
    fits.iter().find(|fit| !carried(fit))
}

/// Turn a fit into one the dogma engine calculates, without skills.
pub fn to_fit(info: &impl InfoEsf, fit: &EsfFit) -> Result<Fit, Error> {
    let lookup = Lookup { info };
    let Some(hull) = fit
        .hull
        .filter(|hull| lookup.classify(*hull) == Kind::Hull && !lookup.is_container(*hull))
    else {
        return Err(Error::new(
            "only a fit of a ship or structure can be calculated",
        ));
    };

    let mut items = Vec::new();
    let mut counters = [0; 6];
    let mut tubes = 0;

    for entry in &fit.entries {
        let count = entry.count.unwrap_or(1);
        let Some(type_id) = entry.type_id else {
            if let Some(rack) = entry.location {
                next_index(&mut counters, rack, count)?;
            }
            continue;
        };
        let kind = lookup.classify(type_id);

        let state = match entry.state {
            Some(State::Off) => EngineState::Offline,
            Some(State::On) => EngineState::Online,
            Some(State::Heat) => EngineState::Overload,
            None if matches!(kind, Kind::Implant | Kind::Booster) => EngineState::Online,
            None if lookup.is_active(type_id) => EngineState::Active,
            None => EngineState::Online,
        };

        let mut item = FitItem {
            type_id,
            slot: Slot::Cargo,
            quantity: count,
            state: EngineState::Offline,
            charge: entry.charge.map(|type_id| Charge { type_id }),
            mutation: None,
            fighter_abilities: None,
            booster_side_effects: BTreeSet::new(),
            spool: None,
        };
        if let Some(mutaplasmid) = entry.mutaplasmid {
            item.type_id = lookup.mutated(mutaplasmid, type_id).ok_or_else(|| {
                Error::new(format!(
                    "mutaplasmid {mutaplasmid} does not apply to {type_id}"
                ))
            })?;
        }
        if entry.mutaplasmid.is_some() || !entry.overrides.is_empty() {
            item.mutation = Some(Mutation {
                base: type_id,
                attributes: entry.overrides.iter().copied().collect(),
            });
        }

        let character_slot = |attribute_id| {
            lookup
                .attribute(type_id, attribute_id)
                .map(|index| index as i64)
        };

        match (kind, entry.location) {
            _ if entry.fit_name.is_some() => {}
            (_, None) if rack_of(kind).is_some() => {
                let rack = rack_of(kind).unwrap();
                let first = next_index(&mut counters, rack, count)?;
                for index in 0..count {
                    let index = u8::try_from(u32::from(first) + index)
                        .map_err(|_| Error::new(format!("too many {} slots", rack.name())))?;
                    items.push(FitItem {
                        slot: rack_slot(rack, index),
                        quantity: 1,
                        state,
                        ..item.clone()
                    });
                }
                continue;
            }
            (Kind::Drone, None) => {
                item.slot = Slot::DroneBay;
                item.state = state;
            }
            (Kind::Drone, Some(Location::Bay)) => item.slot = Slot::DroneBay,
            (Kind::Fighter, None) => {
                item.slot = Slot::FighterTube(tubes);
                item.state = state;
                item.quantity = entry
                    .count
                    .or_else(|| lookup.squadron_size(type_id))
                    .unwrap_or(1);
                tubes = tubes.saturating_add(1);
            }
            (Kind::Fighter, Some(Location::Bay)) => item.slot = Slot::FighterBay,
            (Kind::Implant, None) => {
                if let Some(index) =
                    character_slot(ATTRIBUTE_IMPLANTNESS).and_then(|index| u8::try_from(index).ok())
                {
                    item.slot = Slot::Implant(index);
                    item.state = state;
                }
            }
            (Kind::Booster, None) => {
                if let Some(index) = character_slot(ATTRIBUTE_BOOSTERNESS)
                    .and_then(|index| u16::try_from(index).ok())
                {
                    item.slot = Slot::Booster(index);
                    item.state = state;
                }
            }
            _ => {}
        }
        items.push(item);
    }

    Ok(Fit {
        name: fit.name.clone(),
        ship: Ship {
            type_id: hull,
            mode: fit.mode,
        },
        items,
        character: Character::default(),
        environment: Environment::default(),
        incoming: Projection::default(),
    })
}

fn entry(info: &impl InfoEsf, item: &FitItem) -> Entry {
    let mut entry = Entry {
        type_id: Some(item.type_id),
        count: Some(item.quantity),
        charge: item.charge.as_ref().map(|charge| charge.type_id),
        ..Entry::default()
    };

    if let Some(mutation) = &item.mutation {
        entry.overrides = mutation
            .attributes
            .iter()
            .map(|(id, value)| (*id, *value))
            .collect();
        if mutation.base != item.type_id
            && let Some(mutaplasmid) =
                info.find_mutaplasmid(mutation.base, item.type_id, &mutation.attributes)
        {
            entry.type_id = Some(mutation.base);
            entry.mutaplasmid = Some(mutaplasmid);
        }
    }

    let state = match item.state {
        EngineState::Offline => Some(State::Off),
        EngineState::Online => Some(State::On),
        EngineState::Active => None,
        EngineState::Overload => Some(State::Heat),
    };
    match item.slot {
        Slot::DroneBay | Slot::FighterBay if item.state == EngineState::Offline => {
            entry.location = Some(Location::Bay);
        }
        Slot::Implant(_) | Slot::Booster(_) => {
            entry.count = None;
            entry.state = state.filter(|state| *state == State::Off);
        }
        Slot::Cargo => entry.location = Some(Location::Cargo),
        _ => entry.state = state,
    }
    entry
}

/// Turn a fit the dogma engine calculates into an esf/1 fit.
pub fn from_fit(info: &impl InfoEsf, fit: &Fit) -> Result<EsfFit, Error> {
    let mut racks: Vec<(Location, [Option<&FitItem>; 256])> = Vec::new();
    let mut entries = Vec::new();

    for item in fit.items.iter().filter(|item| item.quantity > 0) {
        let Some((rack, index)) = slot_rack(item.slot) else {
            entries.push(entry(info, item));
            continue;
        };
        let at = match racks.iter().position(|(location, _)| *location == rack) {
            Some(at) => at,
            None => {
                racks.push((rack, [None; 256]));
                racks.len() - 1
            }
        };
        racks[at].1[usize::from(index)] = Some(item);
    }

    for (rack, slots) in racks {
        let mut position = 0;
        let slots = (0..=u8::MAX).zip(slots);
        for (index, item) in slots.filter_map(|(index, item)| Some((index, item?))) {
            if index > position {
                entries.push(Entry {
                    count: Some(u32::from(index - position)),
                    location: Some(rack),
                    ..Entry::default()
                });
            }
            entries.push(Entry {
                count: None,
                ..entry(info, item)
            });
            position = index.saturating_add(1);
        }
    }

    let esf_fit = EsfFit {
        hull: Some(fit.ship.type_id),
        name: fit.name.clone().filter(|name| !name.is_empty()),
        mode: fit.ship.mode,
        entries,
    };
    let mut fits = canonicalise(&Lookup { info }, &[esf_fit])?;
    Ok(fits.remove(0))
}
