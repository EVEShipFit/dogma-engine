use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use strum_macros::EnumIter;

use super::output::Source;
use crate::fit::{FitItem, Slot, State};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    Target,
}

#[derive(Debug)]
pub struct Effect {
    pub effect_id: i32,
    pub operator: EffectOperator,
    pub penalty: bool,
    pub source: Object,
    pub source_category: EffectCategory,
    pub source_attribute_id: i32,
    pub quantity: u32,
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
    pub state: EffectCategory,
    pub max_state: EffectCategory,
    pub attributes: BTreeMap<i32, Attribute>,
    pub effects: Vec<i32>,
    pub fighter_abilities: Option<BTreeSet<i32>>,
    pub booster_side_effects: BTreeSet<i32>,
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

impl EffectCategory {
    pub fn is_active(&self) -> bool {
        matches!(self, EffectCategory::Active | EffectCategory::Overload)
    }
}

impl From<State> for EffectCategory {
    fn from(state: State) -> EffectCategory {
        match state {
            State::Offline => EffectCategory::Passive,
            State::Online => EffectCategory::Online,
            State::Active => EffectCategory::Active,
            State::Overload => EffectCategory::Overload,
        }
    }
}

impl From<EffectCategory> for State {
    fn from(category: EffectCategory) -> State {
        match category {
            EffectCategory::Passive => State::Offline,
            EffectCategory::Online => State::Online,
            EffectCategory::Active => State::Active,
            EffectCategory::Overload => State::Overload,
            category => unreachable!("{category:?} is not an item state"),
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
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: None,
            booster_side_effects: BTreeSet::new(),
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
            max_state: EffectCategory::Passive,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: fit_item.fighter_abilities.clone(),
            booster_side_effects: fit_item.booster_side_effects.clone(),
        };

        match item.slot {
            Some(Slot::DroneBay | Slot::FighterTube(_)) => {
                if item.state != EffectCategory::Passive {
                    item.state = EffectCategory::Active;
                }
                item.max_state = EffectCategory::Active;
            }
            Some(Slot::FighterBay) => item.state = EffectCategory::Passive,
            _ if !item.is_calculated() => item.state = EffectCategory::Passive,
            _ => {}
        }

        item
    }

    pub fn new_fake(type_id: i32) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: None,
            quantity: 1,
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
            fighter_abilities: None,
            booster_side_effects: BTreeSet::new(),
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
