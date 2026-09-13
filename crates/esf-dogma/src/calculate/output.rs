use std::collections::BTreeMap;

use serde::Serialize;

use super::Objects;
use super::item::{EffectOperator, Item, Object};
use crate::fit::State;

#[derive(Serialize, Debug)]
pub struct Calculation {
    pub ship: ItemResult,
    /// Index-parallel to `Fit::items`: same length, same order.
    pub items: Vec<ItemResult>,
    pub character: ItemResult,
}

#[derive(Serialize, Debug)]
pub struct ItemResult {
    pub attributes: BTreeMap<i32, AttributeValue>,
    /// The state actually reached, which may be below what was requested.
    pub state: State,
    pub max_state: State,
    pub charge: Option<Box<ItemResult>>,
}

#[derive(Serialize, Debug)]
pub struct AttributeValue {
    pub base: f64,
    pub value: f64,
    /// In the order pass 3 applied them; empty unless `Options::sources` is set.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Source {
    pub from: SourceRef,
    pub effect_id: i32,
    /// The attribute on the source that holds `value`.
    pub source_attribute_id: i32,
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

/// `Item` and `Charge` index into `Fit::items`. Skills are not in the result, so they carry their type.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceRef {
    Ship,
    Character,
    Item { index: usize },
    Charge { index: usize },
    Skill { type_id: i32 },
}

impl SourceRef {
    pub(super) fn new(object: Object, type_id: i32) -> SourceRef {
        match object {
            Object::Ship => SourceRef::Ship,
            Object::Char => SourceRef::Character,
            Object::Item(index) => SourceRef::Item { index },
            Object::Charge(index) => SourceRef::Charge { index },
            Object::Skill(_) => SourceRef::Skill { type_id },
            Object::Structure | Object::Target => {
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
            items: objects.items.iter().map(ItemResult::new).collect(),
            character: ItemResult::new(&objects.char),
        }
    }
}
