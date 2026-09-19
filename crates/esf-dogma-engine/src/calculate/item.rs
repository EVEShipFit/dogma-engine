use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use strum_macros::EnumIter;

use super::output::Source;
use crate::fit::{FitItem, Slot, State};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EffectCategory {
    Passive,
    Online,
    Active,
    Overload,
    Target,
    Area,
    Dungeon,
    System,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemState {
    Passive,
    Online,
    Active,
    Overload,
    /// Not a state an item is put in; whatever holds it always runs.
    AlwaysOn,
}

/* Declaration order is the order pass 3 applies operators in; do not reorder. */
/// How a modifier changes an attribute. They are applied in this order.
#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum EffectOperator {
    /// Set the value, before anything else.
    PreAssign,
    /// Multiply the value.
    PreMul,
    /// Divide the value.
    PreDiv,
    /// Add to the value.
    ModAdd,
    /// Subtract from the value.
    ModSub,
    /// Multiply the value, after adding.
    PostMul,
    /// Divide the value, after adding.
    PostDiv,
    /// Change the value by a percentage.
    PostPercent,
    /// Set the value, after everything else.
    PostAssign,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Object {
    Ship,
    Mode,
    Item(usize),
    Charge(usize),
    Skill(usize),
    Char,
    Projected(usize),
}

/// Where a modifier comes from, and where its strength is read.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Origin {
    /// A dogma effect on an object, reading its strength off an attribute there.
    Effect {
        effect_id: i32,
        source: Object,
        source_category: EffectCategory,
        attribute_id: i32,
    },
    /// A buff, which carries its own strength and has already won.
    Buff { buff_id: i32, value: f64 },
}

#[derive(Debug)]
pub struct Effect {
    pub origin: Origin,
    pub operator: EffectOperator,
    pub penalty: bool,
    pub quantity: u32,
    pub resistance: Option<i32>,
}

impl Origin {
    /// The effect that holds the modifier; `None` for a buff, which has none.
    pub fn effect_id(self) -> Option<i32> {
        match self {
            Origin::Effect { effect_id, .. } => Some(effect_id),
            Origin::Buff { .. } => None,
        }
    }

    /// The attribute the strength is read off; `None` for a buff, which carries it.
    pub fn source_attribute_id(self) -> Option<i32> {
        match self {
            Origin::Effect { attribute_id, .. } => Some(attribute_id),
            Origin::Buff { .. } => None,
        }
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub base_value: f64,
    pub value: Cell<Option<f64>>,
    pub effects: Vec<Effect>,
    /* Only filled when the calculation asks for sources. */
    pub sources: RefCell<Vec<Source>>,
}

#[derive(Debug)]
pub struct Item {
    pub type_id: i32,
    pub group_id: i32,
    pub category_id: i32,

    pub slot: Option<Slot>,
    pub quantity: u32,
    pub charge: Option<Box<Item>>,
    pub state: ItemState,
    pub max_state: ItemState,
    pub attributes: BTreeMap<i32, Attribute>,
    pub effects: Vec<i32>,
    pub fighter_abilities: Option<BTreeSet<i32>>,
    pub booster_side_effects: BTreeSet<i32>,
    pub mutation_base: Option<i32>,
}

impl Attribute {
    pub fn new(value: f64) -> Attribute {
        Attribute {
            base_value: value,
            value: Cell::new(None),
            effects: Vec::new(),
            sources: RefCell::new(Vec::new()),
        }
    }
}

impl ItemState {
    pub fn is_active(self) -> bool {
        self >= ItemState::Active
    }
}

impl EffectCategory {
    pub fn required_state(self) -> Option<ItemState> {
        match self {
            EffectCategory::Passive => Some(ItemState::Passive),
            EffectCategory::Online => Some(ItemState::Online),
            EffectCategory::Active => Some(ItemState::Active),
            EffectCategory::Overload => Some(ItemState::Overload),
            EffectCategory::Target
            | EffectCategory::Area
            | EffectCategory::Dungeon
            | EffectCategory::System => None,
        }
    }

    /// Whether an effect of this category runs on an item in `state`.
    pub fn runs_at(self, state: ItemState) -> bool {
        match self.required_state() {
            Some(required) => state >= required,
            /* Asks for no state, so it runs the moment the item is active. */
            None => state.is_active(),
        }
    }
}

impl From<State> for ItemState {
    fn from(state: State) -> ItemState {
        match state {
            State::Offline => ItemState::Passive,
            State::Online => ItemState::Online,
            State::Active => ItemState::Active,
            State::Overload => ItemState::Overload,
        }
    }
}

impl From<ItemState> for State {
    fn from(state: ItemState) -> State {
        match state {
            ItemState::Passive => State::Offline,
            ItemState::Online => State::Online,
            ItemState::Active => State::Active,
            ItemState::Overload => State::Overload,
            /* Only a beacon is always-on, and beacons are not in the result. */
            ItemState::AlwaysOn => unreachable!("an always-on item has no fit state"),
        }
    }
}

impl Item {
    pub fn is_module(&self) -> bool {
        matches!(
            self.slot,
            Some(
                Slot::High(_) | Slot::Medium(_) | Slot::Low(_) | Slot::Rig(_) | Slot::Subsystem(_)
            )
        )
    }

    /* Drones are owned by the character, but not located in the ship. */
    pub fn is_in_ship(&self) -> bool {
        self.is_module() || matches!(self.slot, Some(Slot::Service(_)))
    }

    pub fn is_fighter(&self) -> bool {
        matches!(self.slot, Some(Slot::FighterTube(_) | Slot::FighterBay))
    }

    pub fn is_on_char(&self) -> bool {
        matches!(self.slot, Some(Slot::Implant(_) | Slot::Booster(_)))
    }

    pub fn is_calculated(&self) -> bool {
        self.is_in_ship()
            || self.is_fighter()
            || self.is_on_char()
            || self.slot == Some(Slot::DroneBay)
    }

    pub fn new_charge(type_id: i32) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: None,
            quantity: 1,
            charge: None,
            state: ItemState::Active,
            max_state: ItemState::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: None,
            booster_side_effects: BTreeSet::new(),
            mutation_base: None,
        }
    }

    pub fn new_fit(fit_item: &FitItem) -> Item {
        let mut item = Item {
            type_id: fit_item.type_id,
            group_id: 0,
            category_id: 0,
            slot: Some(fit_item.slot),
            quantity: fit_item.quantity,
            charge: fit_item
                .charge
                .as_ref()
                .map(|charge| Box::new(Item::new_charge(charge.type_id))),
            state: fit_item.state.into(),
            max_state: ItemState::Passive,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: fit_item.fighter_abilities.clone(),
            booster_side_effects: fit_item.booster_side_effects.clone(),
            mutation_base: fit_item.mutation.as_ref().map(|mutation| mutation.base),
        };

        match item.slot {
            Some(Slot::DroneBay | Slot::FighterTube(_)) => {
                if item.state != ItemState::Passive {
                    item.state = ItemState::Active;
                }
                item.max_state = ItemState::Active;
            }
            Some(Slot::FighterBay) => item.state = ItemState::Passive,
            _ if !item.is_calculated() => item.state = ItemState::Passive,
            _ => {}
        }

        item
    }

    pub fn new_projected(type_id: i32) -> Item {
        Item {
            state: ItemState::AlwaysOn,
            max_state: ItemState::AlwaysOn,
            ..Item::new_fake(type_id)
        }
    }

    pub fn new_fake(type_id: i32) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: None,
            quantity: 1,
            charge: None,
            state: ItemState::Active,
            max_state: ItemState::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: None,
            booster_side_effects: BTreeSet::new(),
            mutation_base: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffectOperator;
    use strum::IntoEnumIterator;

    #[test]
    fn effect_operator_iterates_in_application_order() {
        assert_eq!(
            EffectOperator::iter().collect::<Vec<_>>(),
            [
                EffectOperator::PreAssign,
                EffectOperator::PreMul,
                EffectOperator::PreDiv,
                EffectOperator::ModAdd,
                EffectOperator::ModSub,
                EffectOperator::PostMul,
                EffectOperator::PostDiv,
                EffectOperator::PostPercent,
                EffectOperator::PostAssign,
            ]
        );
    }
}
