use serde::Serialize;
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
    pub value: Option<f64>,
    pub effects: Vec<Effect>,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum SlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
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
            value: None,
            effects: Vec::new(),
        }
    }
}

impl EffectCategory {
    pub fn is_active(&self) -> bool {
        match self {
            EffectCategory::Active | EffectCategory::Overload => true,
            _ => false,
        }
    }
}

impl Slot {
    pub fn is_module(&self) -> bool {
        match self.r#type {
            SlotType::High
            | SlotType::Medium
            | SlotType::Low
            | SlotType::Rig
            | SlotType::SubSystem => true,
            _ => false,
        }
    }
}

impl Item {
    pub fn new_charge(type_id: i32) -> Item {
        Item {
            type_id,
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
