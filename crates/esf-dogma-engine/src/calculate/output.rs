use std::collections::BTreeMap;

use serde::Serialize;

use super::Objects;
use super::item::{EffectOperator, Item, Object};
use crate::fit::State;

/// The result of [`calculate()`](crate::calculate).
#[derive(Serialize, Debug)]
pub struct Calculation {
    /// The ship.
    pub ship: ItemResult,
    /// The active mode, if the ship has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ItemResult>,
    /// Index-parallel to `Fit::items`: same length, same order.
    pub items: Vec<ItemResult>,
    /// The character.
    pub character: ItemResult,
}

/// The calculated attributes of the ship, its mode, the character, or one item.
#[derive(Serialize, Debug)]
pub struct ItemResult {
    /// Every attribute, by attribute id.
    pub attributes: BTreeMap<i32, AttributeValue>,
    /// The state actually reached, which may be below what was requested.
    pub state: State,
    /// The highest state the item can reach.
    pub max_state: State,
    /// The charge loaded in the module, if any.
    pub charge: Option<Box<ItemResult>>,
}

/// One attribute, before and after the effects on it.
#[derive(Serialize, Debug)]
pub struct AttributeValue {
    /// The value from the SDE.
    pub base: f64,
    /// The value after every effect is applied.
    pub value: f64,
    /// In the order pass 3 applied them; empty unless `Options::sources` is set.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
}

/// One modifier on an attribute, and where it came from.
#[derive(Serialize, Debug, Clone)]
pub struct Source {
    /// The object the effect belongs to.
    pub from: SourceRef,
    /// The effect that holds the modifier.
    pub effect_id: i32,
    /// The attribute on the source that holds `value`.
    pub source_attribute_id: i32,
    /// How `value` changes the attribute.
    pub operator: EffectOperator,
    /// The source attribute's value, as pass 3 used it.
    pub value: f64,
    /// A penalised stack is split into one entry per item, as each gets its own penalty.
    pub quantity: u32,
    /// The stacking penalty factor applied; `None` when not penalised.
    pub penalty: Option<f64>,
    /// False when the source's state is too low for the effect.
    pub applied: bool,
}

/// `Item` and `Charge` index into `Fit::items`. Skills and beacons are not in the result, so they carry their type.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceRef {
    /// The ship.
    Ship,
    /// The active mode of the ship.
    Mode,
    /// The character.
    Character,
    /// An item of the fit.
    Item {
        /// The position in `Fit::items`.
        index: usize,
    },
    /// The charge in an item of the fit.
    Charge {
        /// The position in `Fit::items` of the item holding the charge.
        index: usize,
    },
    /// A skill of the character.
    Skill {
        /// The type id of the skill.
        type_id: i32,
    },
    /// An effect beacon of the solar system.
    System {
        /// The type id of the beacon.
        type_id: i32,
    },
}

impl SourceRef {
    pub(super) fn new(object: Object, type_id: i32) -> SourceRef {
        match object {
            Object::Ship => SourceRef::Ship,
            Object::Mode => SourceRef::Mode,
            Object::Char => SourceRef::Character,
            Object::Item(index) => SourceRef::Item { index },
            Object::Charge(index) => SourceRef::Charge { index },
            Object::Skill(_) => SourceRef::Skill { type_id },
            Object::System(_) => SourceRef::System { type_id },
            Object::Target => {
                unreachable!("{object:?} is never the source of an effect")
            }
        }
    }
}

impl ItemResult {
    fn new(item: &Item) -> ItemResult {
        ItemResult {
            attributes: item
                .attributes
                .iter()
                .map(|(attribute_id, attribute)| {
                    let value = AttributeValue {
                        base: attribute.base_value,
                        value: attribute.value.get().unwrap_or(attribute.base_value),
                        sources: attribute.sources.borrow().clone(),
                    };
                    (*attribute_id, value)
                })
                .collect(),
            state: item.state.into(),
            max_state: item.max_state.into(),
            charge: item
                .charge
                .as_deref()
                .map(|charge| Box::new(ItemResult::new(charge))),
        }
    }
}

impl Calculation {
    pub(super) fn new(objects: &Objects) -> Calculation {
        Calculation {
            ship: ItemResult::new(&objects.ship),
            mode: objects.mode.as_ref().map(ItemResult::new),
            items: objects.items.iter().map(ItemResult::new).collect(),
            character: ItemResult::new(&objects.char),
        }
    }
}
