use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use super::esf_data;
use super::protobuf;
use crate::data_types;
use crate::info::Info;

pub struct InfoMain {
    pub fit: data_types::EsfFit,
    pub skills: BTreeMap<i32, i32>,
    pub types: HashMap<i32, esf_data::types::Type>,
    pub type_dogma: HashMap<i32, esf_data::type_dogma::TypeDogmaEntry>,
    pub dogma_attributes: HashMap<i32, esf_data::dogma_attributes::DogmaAttribute>,
    pub dogma_effects: HashMap<i32, esf_data::dogma_effects::DogmaEffect>,
}

impl Info for InfoMain {
    fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute> {
        match self.type_dogma.get(&type_id) {
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
        match self.dogma_attributes.get(&attribute_id) {
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
        match self.type_dogma.get(&type_id) {
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
        match self.dogma_effects.get(&effect_id) {
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

    fn get_type_id(&self, type_id: i32) -> data_types::TypeId {
        match self.types.get(&type_id) {
            None => data_types::TypeId {
                groupID: 0,
                categoryID: 0,
                capacity: None,
                mass: None,
                volume: None,
                radius: None,
            },
            Some(type_) => data_types::TypeId {
                groupID: type_.group_id,
                categoryID: type_.category_id,
                capacity: type_.capacity.map(|x| x as f64),
                mass: type_.mass.map(|x| x as f64),
                volume: type_.volume.map(|x| x as f64),
                radius: type_.radius.map(|x| x as f64),
            },
        }
    }

    fn skills(&self) -> &BTreeMap<i32, i32> {
        &self.skills
    }

    fn fit(&self) -> &data_types::EsfFit {
        &self.fit
    }
}

impl InfoMain {
    pub fn new<'a>(
        fit: data_types::EsfFit,
        skills: BTreeMap<i32, i32>,
        protobuf_location: &PathBuf,
    ) -> InfoMain {
        let dogma_attributes: esf_data::DogmaAttributes =
            protobuf::load_from_npm(protobuf_location, "dogmaAttributes").unwrap();
        let dogma_effects: esf_data::DogmaEffects =
            protobuf::load_from_npm(protobuf_location, "dogmaEffects").unwrap();
        let type_dogma: esf_data::TypeDogma =
            protobuf::load_from_npm(protobuf_location, "typeDogma").unwrap();
        let types: esf_data::Types = protobuf::load_from_npm(protobuf_location, "types").unwrap();

        InfoMain {
            fit,
            skills,
            types: types.entries,
            type_dogma: type_dogma.entries,
            dogma_attributes: dogma_attributes.entries,
            dogma_effects: dogma_effects.entries,
        }
    }
}
