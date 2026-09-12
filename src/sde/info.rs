use std::collections::BTreeMap;

use super::{Names, Sde, eve};
use crate::data_types;
use crate::info::{Info, InfoName};

pub struct InfoSde<'a> {
    pub fit: data_types::EsfFit,
    pub skills: BTreeMap<i32, i32>,
    pub sde: &'a Sde<'a>,
}

pub struct InfoNameSde<'a> {
    pub sde: &'a Sde<'a>,
    pub names: Option<&'a Names<'a>>,
}

/// Flatbuffers has no null for scalars; an id that was never set reads as
/// zero, which is not a valid attribute id.
fn optional_id(id: i32) -> Option<i32> {
    if id == 0 { None } else { Some(id) }
}

fn convert_type(type_: Option<eve::Type>) -> data_types::Type {
    match type_ {
        None => data_types::Type {
            groupID: 0,
            categoryID: 0,
            capacity: None,
            mass: None,
            radius: None,
            volume: None,
        },
        Some(type_) => data_types::Type {
            groupID: type_.group_id(),
            categoryID: type_.category_id(),
            capacity: type_.capacity().map(|value| value as f64),
            mass: type_.mass().map(|value| value as f64),
            radius: type_.radius().map(|value| value as f64),
            volume: type_.volume().map(|value| value as f64),
        },
    }
}

fn convert_dogma_attributes(type_: Option<eve::Type>) -> Vec<data_types::TypeDogmaAttribute> {
    let Some(attributes) = type_.and_then(|type_| type_.dogma_attributes()) else {
        return vec![];
    };

    attributes
        .iter()
        .map(|attribute| data_types::TypeDogmaAttribute {
            attributeID: attribute.attribute_id(),
            value: attribute.value() as f64,
        })
        .collect()
}

fn convert_dogma_effects(type_: Option<eve::Type>) -> Vec<data_types::TypeDogmaEffect> {
    let Some(effects) = type_.and_then(|type_| type_.dogma_effects()) else {
        return vec![];
    };

    effects
        .iter()
        .map(|effect| data_types::TypeDogmaEffect {
            effectID: effect.effect_id(),
            isDefault: effect.is_default(),
        })
        .collect()
}

fn convert_modifier(modifier: &eve::Modifier) -> data_types::DogmaEffectModifierInfo {
    let operation = modifier.operation();

    data_types::DogmaEffectModifierInfo {
        domain: (modifier.domain().0 as i32).into(),
        func: (modifier.func().0 as i32).into(),
        modifiedAttributeID: optional_id(modifier.modified_attribute_id()),
        modifyingAttributeID: optional_id(modifier.modifying_attribute_id()),
        operation: match operation {
            eve::ModifierOperation::Unset => None,
            operation => Some(operation.0 as i32),
        },
        groupID: optional_id(modifier.group_id()),
        skillTypeID: optional_id(modifier.skill_type_id()),
    }
}

fn convert_dogma_effect(effect: Option<eve::DogmaEffect>) -> data_types::DogmaEffect {
    match effect {
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
        Some(effect) => data_types::DogmaEffect {
            dischargeAttributeID: optional_id(effect.discharge_attribute_id()),
            durationAttributeID: optional_id(effect.duration_attribute_id()),
            effectCategory: effect.effect_category().0 as i32,
            electronicChance: effect.electronic_chance(),
            isAssistance: effect.is_assistance(),
            isOffensive: effect.is_offensive(),
            isWarpSafe: effect.is_warp_safe(),
            propulsionChance: effect.propulsion_chance(),
            rangeChance: effect.range_chance(),
            rangeAttributeID: optional_id(effect.range_attribute_id()),
            falloffAttributeID: optional_id(effect.falloff_attribute_id()),
            trackingSpeedAttributeID: optional_id(effect.tracking_speed_attribute_id()),
            fittingUsageChanceAttributeID: optional_id(effect.fitting_usage_chance_attribute_id()),
            resistanceAttributeID: optional_id(effect.resistance_attribute_id()),
            modifierInfo: match effect.modifiers() {
                None => vec![],
                Some(modifiers) => modifiers.iter().map(convert_modifier).collect(),
            },
        },
    }
}

impl Info for InfoSde<'_> {
    fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute> {
        convert_dogma_attributes(self.sde.get_type(type_id))
    }

    fn get_dogma_attribute(&self, attribute_id: i32) -> data_types::DogmaAttribute {
        match self.sde.get_dogma_attribute(attribute_id) {
            None => data_types::DogmaAttribute {
                defaultValue: 0.0,
                highIsGood: false,
                stackable: false,
            },
            Some(attribute) => data_types::DogmaAttribute {
                defaultValue: attribute.default_value() as f64,
                highIsGood: attribute.high_is_good(),
                stackable: attribute.stackable(),
            },
        }
    }

    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        convert_dogma_effects(self.sde.get_type(type_id))
    }

    fn get_dogma_effect(&self, effect_id: i32) -> data_types::DogmaEffect {
        convert_dogma_effect(self.sde.get_dogma_effect(effect_id))
    }

    fn get_type(&self, type_id: i32) -> data_types::Type {
        convert_type(self.sde.get_type(type_id))
    }

    fn attribute_name_to_id(&self, name: &str) -> i32 {
        self.sde.attribute_name_to_id(name).unwrap_or(0)
    }

    fn skills(&self) -> &BTreeMap<i32, i32> {
        &self.skills
    }

    fn fit(&self) -> &data_types::EsfFit {
        &self.fit
    }
}

impl InfoName for InfoNameSde<'_> {
    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        convert_dogma_effects(self.sde.get_type(type_id))
    }

    fn get_type(&self, type_id: i32) -> data_types::Type {
        convert_type(self.sde.get_type(type_id))
    }

    fn type_name_to_id(&self, name: &str) -> i32 {
        self.sde
            .type_name_to_id(name)
            .or_else(|| self.names.and_then(|names| names.type_name_to_id(name)))
            .unwrap_or(0)
    }
}

impl<'a> InfoSde<'a> {
    pub fn new(
        fit: data_types::EsfFit,
        skills: BTreeMap<i32, i32>,
        sde: &'a Sde<'a>,
    ) -> InfoSde<'a> {
        InfoSde { fit, skills, sde }
    }
}

impl<'a> InfoNameSde<'a> {
    /// The type ids in `names.dat` only mean anything against the SDE it was
    /// built from, so refuse a pair that does not match.
    pub fn new(sde: &'a Sde<'a>, names: Option<&'a Names<'a>>) -> Result<InfoNameSde<'a>, String> {
        if let Some(names) = names
            && sde.build_number() != names.build_number()
        {
            return Err(format!(
                "SDE is build {} but the names are build {}",
                sde.build_number(),
                names.build_number()
            ));
        }

        Ok(InfoNameSde { sde, names })
    }
}
