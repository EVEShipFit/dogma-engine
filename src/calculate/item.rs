use serde::Serialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use strum_macros::EnumIter;

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum EffectOperator {
    PreAssign,
    PreMul,
    PreDiv,
    ModAdd,
    ModSub,
    PostMul,
    PostDiv,
    PostPercent,
    PostAssign,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub enum Object {
    Ship,
    Item(usize),
    Charge(usize),
    Skill(usize),
    Char,
    Structure,
    Target,
}

#[derive(Serialize, Debug)]
pub struct Effect {
    pub operator: EffectOperator,
    pub penalty: bool,
    pub source: Object,
    pub source_category: EffectCategory,
    pub source_attribute_id: i32,
}

#[derive(Serialize, Debug)]
pub struct Attribute {
    pub base_value: f64,
    pub value: Cell<Option<f64>>,
    pub effects: Vec<Effect>,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum SlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
    Service,
    DroneBay,
    Charge,
    None,
}

#[derive(Serialize, Debug)]
pub struct Slot {
    pub r#type: SlotType,
    pub index: Option<i32>,
}

#[derive(Serialize, Debug)]
pub struct Item {
    pub type_id: i32,
    #[serde(skip)]
    pub group_id: i32,
    #[serde(skip)]
    pub category_id: i32,

    pub slot: Slot,
    pub charge: Option<Box<Item>>,
    pub state: EffectCategory,
    pub max_state: EffectCategory,
    pub attributes: BTreeMap<i32, Attribute>,
    pub effects: Vec<i32>,
}

impl Attribute {
    pub fn new(value: f64) -> Attribute {
        Attribute {
            base_value: value,
            value: Cell::new(None),
            effects: Vec::new(),
        }
    }
}

impl EffectCategory {
    pub fn is_active(&self) -> bool {
        matches!(self, EffectCategory::Active | EffectCategory::Overload)
    }
}

impl Slot {
    pub fn is_module(&self) -> bool {
        matches!(
            self.r#type,
            SlotType::High | SlotType::Medium | SlotType::Low | SlotType::Rig | SlotType::SubSystem
        )
    }

    /* Drones are owned by the character, but not located in the ship. */
    pub fn is_in_ship(&self) -> bool {
        !matches!(self.r#type, SlotType::DroneBay)
    }
}

impl Item {
    pub fn new_charge(type_id: i32) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: Slot {
                r#type: SlotType::Charge,
                index: None,
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_module(
        type_id: i32,
        slot: Slot,
        charge_type_id: Option<i32>,
        state: EffectCategory,
    ) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot,
            charge: charge_type_id.map(|charge_type_id| Box::new(Item::new_charge(charge_type_id))),
            state,
            max_state: EffectCategory::Passive,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_drone(type_id: i32, state: EffectCategory) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: Slot {
                r#type: SlotType::DroneBay,
                index: None,
            },
            charge: None,
            state,
            max_state: EffectCategory::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_fake(type_id: i32) -> Item {
        Item {
            type_id,
            group_id: 0,
            category_id: 0,
            slot: Slot {
                r#type: SlotType::None,
                index: None,
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
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
