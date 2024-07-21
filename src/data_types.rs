use serde::Deserialize;
use serde_repr::*;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Type {
    pub groupID: i32,
    pub categoryID: i32,
    pub capacity: Option<f64>,
    pub mass: Option<f64>,
    pub radius: Option<f64>,
    pub volume: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TypeDogmaAttribute {
    pub attributeID: i32,
    pub value: f64,
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug)]
pub struct TypeDogmaEffect {
    pub effectID: i32,
    pub isDefault: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct DogmaAttribute {
    pub defaultValue: f64,
    pub highIsGood: bool,
    pub stackable: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize_repr, Debug)]
#[repr(i32)]
pub enum DogmaEffectModifierInfoDomain {
    ItemID = 0,
    ShipID = 1,
    CharID = 2,
    OtherID = 3,
    StructureID = 4,
    Target = 5,
    TargetID = 6,
}

#[allow(non_snake_case)]
#[derive(Deserialize_repr, Debug)]
#[repr(i32)]
pub enum DogmaEffectModifierInfoFunc {
    ItemModifier = 0,
    LocationGroupModifier = 1,
    LocationModifier = 2,
    LocationRequiredSkillModifier = 3,
    OwnerRequiredSkillModifier = 4,
    EffectStopper = 5,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct DogmaEffectModifierInfo {
    pub domain: DogmaEffectModifierInfoDomain,
    pub func: DogmaEffectModifierInfoFunc,
    pub modifiedAttributeID: Option<i32>,
    pub modifyingAttributeID: Option<i32>,
    pub operation: Option<i32>,
    pub groupID: Option<i32>,
    pub skillTypeID: Option<i32>,
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug)]
pub struct DogmaEffect {
    pub dischargeAttributeID: Option<i32>,
    pub durationAttributeID: Option<i32>,
    pub effectCategory: i32,
    pub electronicChance: bool,
    pub isAssistance: bool,
    pub isOffensive: bool,
    pub isWarpSafe: bool,
    pub propulsionChance: bool,
    pub rangeChance: bool,
    pub rangeAttributeID: Option<i32>,
    pub falloffAttributeID: Option<i32>,
    pub trackingSpeedAttributeID: Option<i32>,
    pub fittingUsageChanceAttributeID: Option<i32>,
    pub resistanceAttributeID: Option<i32>,
    pub modifierInfo: Vec<DogmaEffectModifierInfo>,
}

#[derive(Deserialize, Debug)]
pub enum EsfState {
    Passive,
    Online,
    Active,
    Overload,
}

#[derive(Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum EsfSlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
}

#[derive(Deserialize, Debug)]
pub struct EsfCharge {
    pub type_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct EsfSlot {
    pub r#type: EsfSlotType,
    pub index: i32,
}

#[derive(Deserialize, Debug)]
pub struct EsfModule {
    pub type_id: i32,
    pub slot: EsfSlot,
    pub state: EsfState,
    pub charge: Option<EsfCharge>,
}

#[derive(Deserialize, Debug)]
pub struct EsfDrone {
    pub type_id: i32,
    pub state: EsfState,
}

#[derive(Deserialize, Debug)]
pub struct EsfFit {
    pub ship_type_id: i32,
    pub modules: Vec<EsfModule>,
    pub drones: Vec<EsfDrone>,
}

impl From<i32> for DogmaEffectModifierInfoDomain {
    fn from(value: i32) -> DogmaEffectModifierInfoDomain {
        match value {
            0 => DogmaEffectModifierInfoDomain::ItemID,
            1 => DogmaEffectModifierInfoDomain::ShipID,
            2 => DogmaEffectModifierInfoDomain::CharID,
            3 => DogmaEffectModifierInfoDomain::OtherID,
            4 => DogmaEffectModifierInfoDomain::StructureID,
            5 => DogmaEffectModifierInfoDomain::Target,
            6 => DogmaEffectModifierInfoDomain::TargetID,
            _ => DogmaEffectModifierInfoDomain::ItemID,
        }
    }
}

impl From<i32> for DogmaEffectModifierInfoFunc {
    fn from(value: i32) -> DogmaEffectModifierInfoFunc {
        match value {
            0 => DogmaEffectModifierInfoFunc::ItemModifier,
            1 => DogmaEffectModifierInfoFunc::LocationGroupModifier,
            2 => DogmaEffectModifierInfoFunc::LocationModifier,
            3 => DogmaEffectModifierInfoFunc::LocationRequiredSkillModifier,
            4 => DogmaEffectModifierInfoFunc::OwnerRequiredSkillModifier,
            5 => DogmaEffectModifierInfoFunc::EffectStopper,
            _ => DogmaEffectModifierInfoFunc::ItemModifier,
        }
    }
}
