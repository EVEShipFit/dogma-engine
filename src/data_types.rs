use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
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
    Service,
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
