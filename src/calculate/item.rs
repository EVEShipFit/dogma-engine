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
    PostAssignment,
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

#[derive(Serialize, Debug)]
pub struct Item {
    pub type_id: i32,
    pub quantity: i32,
    pub flag: i32,
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

impl Item {
    pub fn new_esi(
        type_id: i32,
        quantity: i32,
        flag: i32,
        charge_type_id: Option<i32>,
        state: EffectCategory,
    ) -> Item {
        Item {
            type_id,
            quantity,
            flag,
            charge: charge_type_id.map(|charge_type_id| {
                Box::new(Item::new_esi(
                    charge_type_id,
                    1,
                    -1,
                    None,
                    EffectCategory::Passive,
                ))
            }),
            state,
            max_state: EffectCategory::Passive,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_fake(type_id: i32) -> Item {
        return Self::new_esi(type_id, 1, -1, None, EffectCategory::Active);
    }
}
