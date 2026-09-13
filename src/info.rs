use std::collections::BTreeMap;

use flatbuffers::Vector;

use crate::data_types;
use crate::sde::eve;

pub trait Info {
    fn skills(&self) -> &BTreeMap<i32, i32>;
    fn fit(&self) -> &data_types::EsfFit;

    fn get_dogma_attributes(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaAttribute>>;
    fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'_>>;
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>>;
    fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'_>>;
    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>>;
    fn attribute_name_to_id(&self, name: &str) -> i32;
}

pub trait InfoName {
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>>;
    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>>;
    fn type_name_to_id(&self, name: &str) -> i32;
}
