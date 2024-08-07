use std::collections::BTreeMap;

use crate::data_types;

pub trait Info {
    fn skills(&self) -> &BTreeMap<i32, i32>;
    fn fit(&self) -> &data_types::EsfFit;

    fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute>;
    fn get_dogma_attribute(&self, attribute_id: i32) -> data_types::DogmaAttribute;
    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect>;
    fn get_dogma_effect(&self, effect_id: i32) -> data_types::DogmaEffect;
    fn get_type(&self, type_id: i32) -> data_types::Type;
    fn attribute_name_to_id(&self, name: &str) -> i32;
}

pub trait InfoName {
    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect>;
    fn get_type(&self, type_id: i32) -> data_types::Type;
    fn type_name_to_id(&self, name: &str) -> i32;
}
