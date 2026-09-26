//! The formats that list each item with where it is, rather than grouping
//! items by rack like EFT does.

use std::collections::{BTreeSet, HashMap};

use esf_data::Info;
use esf_dogma_engine::{
    Character, Charge, Environment, Fit, FitItem, Projection, Ship, Slot, State,
};

use crate::flags::Place;

const CATEGORY_CHARGE: i32 = 8;

const ATTRIBUTE_IMPLANTNESS: i32 = 331;
const ATTRIBUTE_BOOSTERNESS: i32 = 1087;

/// What placing an item looks up. [`Info`] and [`esf_data::InfoName`] both
/// answer it, so a format can take whichever it needs for the rest.
pub(crate) trait Types {
    fn category_id(&self, type_id: i32) -> Option<i32>;
    fn attribute(&self, type_id: i32, attribute_id: i32) -> Option<f32>;
}

pub(crate) struct ByInfo<'a, I>(pub &'a I);

impl<I: Info> Types for ByInfo<'_, I> {
    fn category_id(&self, type_id: i32) -> Option<i32> {
        self.0.get_type(type_id).map(|r#type| r#type.category_id())
    }

    fn attribute(&self, type_id: i32, attribute_id: i32) -> Option<f32> {
        self.0
            .get_dogma_attributes(type_id)?
            .iter()
            .find(|attribute| attribute.attribute_id() == attribute_id)
            .map(|attribute| attribute.value())
    }
}

/// An item as a format that lists items by their flag has it.
pub(crate) struct Listed {
    pub place: Place,
    pub type_id: i32,
    pub quantity: u32,
    /// When the format says; otherwise the state fitting it gives.
    pub state: Option<State>,
    pub charge: Option<i32>,
}

impl Listed {
    pub fn new(place: Place, type_id: i32, quantity: u32) -> Self {
        Self {
            place,
            type_id,
            quantity,
            state: None,
            charge: None,
        }
    }
}

/* The order EFT writes them in, so a fit reads the same whichever format it
 * came from. */
fn order(slot: Slot) -> (u8, u16) {
    match slot {
        Slot::Low(index) => (0, index.into()),
        Slot::Medium(index) => (1, index.into()),
        Slot::High(index) => (2, index.into()),
        Slot::Rig(index) => (3, index.into()),
        Slot::Subsystem(index) => (4, index.into()),
        Slot::Service(index) => (5, index.into()),
        Slot::DroneBay => (6, 0),
        Slot::FighterTube(index) => (7, index.into()),
        Slot::FighterBay => (8, 0),
        Slot::Cargo => (9, 0),
        Slot::Implant(index) => (10, index.into()),
        Slot::Booster(index) => (11, index),
    }
}

fn in_rack(slot: Slot) -> bool {
    matches!(
        slot,
        Slot::High(_)
            | Slot::Medium(_)
            | Slot::Low(_)
            | Slot::Rig(_)
            | Slot::Subsystem(_)
            | Slot::Service(_)
    )
}

/// An implant or booster goes in the slot its type is made for; anything
/// else listed there goes in the cargo.
fn to_slot(types: &dyn Types, place: Place, type_id: i32) -> Slot {
    let character_slot = match place {
        Place::Slot(slot) => return slot,
        Place::Implant => types
            .attribute(type_id, ATTRIBUTE_IMPLANTNESS)
            .and_then(|index| u8::try_from(index as i64).ok())
            .map(Slot::Implant),
        Place::Booster => types
            .attribute(type_id, ATTRIBUTE_BOOSTERNESS)
            .and_then(|index| u16::try_from(index as i64).ok())
            .map(Slot::Booster),
    };
    character_slot.unwrap_or(Slot::Cargo)
}

fn item(type_id: i32, slot: Slot, quantity: u32, state: State, charge: Option<i32>) -> FitItem {
    FitItem {
        type_id,
        slot,
        quantity,
        state,
        charge: charge.map(|type_id| Charge { type_id }),
        mutation: None,
        fighter_abilities: None,
        booster_side_effects: BTreeSet::new(),
        spool: None,
    }
}

/// Listed items as fit items, in the order EFT writes them. A charge listed
/// in a module's slot is loaded in that module, and bay items of the same
/// type and state share a stack.
pub(crate) fn to_fit_items(types: &dyn Types, listed: Vec<Listed>) -> Vec<FitItem> {
    let (charges, others): (Vec<_>, Vec<_>) = listed.into_iter().partition(|listed| {
        matches!(listed.place, Place::Slot(slot) if in_rack(slot))
            && types.category_id(listed.type_id) == Some(CATEGORY_CHARGE)
    });
    let charges: HashMap<Place, i32> = charges
        .into_iter()
        .map(|charge| (charge.place, charge.type_id))
        .collect();

    let mut items: Vec<FitItem> = Vec::new();
    for listed in others {
        let slot = to_slot(types, listed.place, listed.type_id);
        let state = listed.state.unwrap_or(match slot {
            Slot::Cargo | Slot::FighterBay => State::Offline,
            _ => State::Active,
        });

        match slot {
            _ if in_rack(slot) => {
                let charge = listed
                    .charge
                    .or_else(|| charges.get(&listed.place).copied());
                items.push(item(listed.type_id, slot, 1, state, charge));
            }
            Slot::Implant(_) | Slot::Booster(_) => {
                items.push(item(listed.type_id, slot, 1, state, None));
            }
            Slot::FighterTube(_) => {
                items.push(item(listed.type_id, slot, listed.quantity, state, None));
            }
            _ => {
                let stack = items.iter_mut().find(|item| {
                    item.slot == slot && item.type_id == listed.type_id && item.state == state
                });
                match stack {
                    Some(stack) => stack.quantity += listed.quantity,
                    None => items.push(item(listed.type_id, slot, listed.quantity, state, None)),
                }
            }
        }
    }

    items.sort_by_key(|item| order(item.slot));
    items
}

/// A fit of these items, with nothing else set.
pub(crate) fn fit(name: Option<String>, ship_type_id: i32, items: Vec<FitItem>) -> Fit {
    Fit {
        name,
        ship: Ship {
            type_id: ship_type_id,
            mode: None,
        },
        items,
        character: Character::default(),
        environment: Environment::default(),
        incoming: Projection::default(),
    }
}
