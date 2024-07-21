use std::collections::BTreeMap;

use super::Data;
use crate::data_types;
use crate::info::{Info, InfoName};

pub struct InfoMain<'a> {
    pub fit: data_types::EsfFit,
    pub skills: BTreeMap<i32, i32>,
    pub data: &'a Data,
}

pub struct InfoNameMain<'a> {
    pub data: &'a Data,
}

impl Info for InfoMain<'_> {
    fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute> {
        match self.data.type_dogma.get(&type_id) {
            None => vec![],
            Some(type_dogma) => {
                let mut attributes = vec![];
                for attribute in &type_dogma.dogma_attributes {
                    attributes.push(data_types::TypeDogmaAttribute {
                        attributeID: attribute.attribute_id,
                        value: attribute.value as f64,
                    });
                }
                attributes
            }
        }
    }

    fn get_dogma_attribute(&self, attribute_id: i32) -> data_types::DogmaAttribute {
        match self.data.dogma_attributes.get(&attribute_id) {
            None => data_types::DogmaAttribute {
                defaultValue: 0.0,
                highIsGood: false,
                stackable: false,
            },
            Some(attribute) => data_types::DogmaAttribute {
                defaultValue: attribute.default_value as f64,
                highIsGood: attribute.high_is_good,
                stackable: attribute.stackable,
            },
        }
    }

    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        match self.data.type_dogma.get(&type_id) {
            None => vec![],
            Some(type_dogma) => {
                let mut effects = vec![];
                for effect in &type_dogma.dogma_effects {
                    effects.push(data_types::TypeDogmaEffect {
                        effectID: effect.effect_id,
                        isDefault: effect.is_default,
                    });
                }
                effects
            }
        }
    }

    fn get_dogma_effect(&self, effect_id: i32) -> data_types::DogmaEffect {
        match self.data.dogma_effects.get(&effect_id) {
            None => data_types::DogmaEffect {
                dischargeAttributeID: None,
                durationAttributeID: None,
                effectCategory: 0,
                electronicChance: false,
                isAssistance: false,
                isOffensive: false,
                isWarpSafe: false,
                propulsionChance: false,
                rangeChance: false,
                rangeAttributeID: None,
                falloffAttributeID: None,
                trackingSpeedAttributeID: None,
                fittingUsageChanceAttributeID: None,
                resistanceAttributeID: None,
                modifierInfo: vec![],
            },
            Some(effect) => {
                let mut modifier_info = vec![];
                for modifier in &effect.modifier_info {
                    modifier_info.push(data_types::DogmaEffectModifierInfo {
                        domain: modifier.domain.into(),
                        operation: modifier.operation,
                        func: modifier.func.into(),
                        groupID: modifier.group_id,
                        skillTypeID: modifier.skill_type_id,
                        modifiedAttributeID: modifier.modified_attribute_id,
                        modifyingAttributeID: modifier.modifying_attribute_id,
                    });
                }

                data_types::DogmaEffect {
                    dischargeAttributeID: effect.discharge_attribute_id,
                    durationAttributeID: effect.duration_attribute_id,
                    effectCategory: effect.effect_category,
                    electronicChance: effect.electronic_chance,
                    isAssistance: effect.is_assistance,
                    isOffensive: effect.is_offensive,
                    isWarpSafe: effect.is_warp_safe,
                    propulsionChance: effect.propulsion_chance,
                    rangeChance: effect.range_chance,
                    rangeAttributeID: effect.range_attribute_id,
                    falloffAttributeID: effect.falloff_attribute_id,
                    trackingSpeedAttributeID: effect.tracking_speed_attribute_id,
                    fittingUsageChanceAttributeID: effect.fitting_usage_chance_attribute_id,
                    resistanceAttributeID: effect.resistance_attribute_id,
                    modifierInfo: modifier_info,
                }
            }
        }
    }

    fn get_type(&self, type_id: i32) -> data_types::Type {
        match self.data.types.get(&type_id) {
            None => data_types::Type {
                groupID: 0,
                categoryID: 0,
                capacity: None,
                mass: None,
                volume: None,
                radius: None,
            },
            Some(type_) => data_types::Type {
                groupID: type_.group_id,
                categoryID: type_.category_id,
                capacity: type_.capacity.map(|x| x as f64),
                mass: type_.mass.map(|x| x as f64),
                volume: type_.volume.map(|x| x as f64),
                radius: type_.radius.map(|x| x as f64),
            },
        }
    }

    fn attribute_name_to_id(&self, name: &str) -> i32 {
        for (attribute_id, attribute) in &self.data.dogma_attributes {
            if attribute.name == name {
                return *attribute_id;
            }
        }
        0
    }

    fn skills(&self) -> &BTreeMap<i32, i32> {
        &self.skills
    }

    fn fit(&self) -> &data_types::EsfFit {
        &self.fit
    }
}

impl InfoName for InfoNameMain<'_> {
    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        match self.data.type_dogma.get(&type_id) {
            None => vec![],
            Some(type_dogma) => {
                let mut effects = vec![];
                for effect in &type_dogma.dogma_effects {
                    effects.push(data_types::TypeDogmaEffect {
                        effectID: effect.effect_id,
                        isDefault: effect.is_default,
                    });
                }
                effects
            }
        }
    }

    fn get_type(&self, type_id: i32) -> data_types::Type {
        match self.data.types.get(&type_id) {
            None => data_types::Type {
                groupID: 0,
                categoryID: 0,
                capacity: None,
                mass: None,
                volume: None,
                radius: None,
            },
            Some(type_) => data_types::Type {
                groupID: type_.group_id,
                categoryID: type_.category_id,
                capacity: type_.capacity.map(|x| x as f64),
                mass: type_.mass.map(|x| x as f64),
                volume: type_.volume.map(|x| x as f64),
                radius: type_.radius.map(|x| x as f64),
            },
        }
    }

    fn type_name_to_id(&self, name: &str) -> i32 {
        for (type_id, type_) in &self.data.types {
            if type_.name == name {
                return *type_id;
            }
        }
        0
    }
}

impl InfoMain<'_> {
    pub fn new<'a>(fit: data_types::EsfFit, skills: BTreeMap<i32, i32>, data: &Data) -> InfoMain {
        InfoMain { fit, skills, data }
    }
}

impl InfoNameMain<'_> {
    pub fn new<'a>(data: &Data) -> InfoNameMain {
        InfoNameMain { data }
    }
}
