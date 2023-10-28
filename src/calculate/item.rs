use serde::Serialize;
use std::collections::BTreeMap;
use strum_macros::EnumIter;

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum EffectCategory {
    Passive,
    Active,
    Target,
    Area,
    Online,
    Overload,
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
    Skill(usize),
    Char,
    Structure,
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
    pub base_value: f32,
    pub value: Option<f32>,
    pub effects: Vec<Effect>,
}

#[derive(Serialize, Debug)]
pub struct Item {
    pub type_id: i32,
    pub attributes: BTreeMap<i32, Attribute>,
    pub effects: Vec<i32>,
}

impl Attribute {
    pub fn new(value: f32) -> Attribute {
        Attribute {
            base_value: value,
            value: None,
            effects: Vec::new(),
        }
    }
}

impl Item {
    pub fn new(type_id: i32) -> Item {
        Item {
            type_id,
            attributes: BTreeMap::new(),
            effects: Vec::new(),
        }
    }
}
