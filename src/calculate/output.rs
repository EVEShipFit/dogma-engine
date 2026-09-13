use std::collections::BTreeMap;

use serde::Serialize;

use super::Ship;
use super::item::Item;
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
    pub(super) fn new(ship: &Ship) -> Calculation {
        Calculation {
            ship: ItemResult::new(&ship.hull),
            items: ship.items.iter().map(ItemResult::new).collect(),
            character: ItemResult::new(&ship.char),
        }
    }
}
