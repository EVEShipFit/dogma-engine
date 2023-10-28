use serde_wasm_bindgen;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

use super::super::data_types;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn get_dogma_attributes(type_id: i32) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn get_dogma_attribute(attribute_id: i32) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn get_dogma_effects(type_id: i32) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn get_dogma_effect(effect_id: i32) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn get_type_id(type_id: i32) -> JsValue;
}

pub struct Info<'a> {
    pub ship_layout: &'a data_types::ShipLayout,
    pub skills: &'a BTreeMap<i32, i32>,
}

impl Info<'_> {
    pub fn new<'a>(
        ship_layout: &'a data_types::ShipLayout,
        skills: &'a BTreeMap<i32, i32>,
    ) -> Info<'a> {
        Info {
            ship_layout,
            skills,
        }
    }

    pub fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute> {
        let js = get_dogma_attributes(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    pub fn get_dogma_attribute(&self, attribute_id: i32) -> data_types::DogmaAttribute {
        let js = get_dogma_attribute(attribute_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    pub fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        let js = get_dogma_effects(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    pub fn get_dogma_effect(&self, effect_id: i32) -> data_types::DogmaEffect {
        let js = get_dogma_effect(effect_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    pub fn get_type_id(&self, type_id: i32) -> data_types::TypeId {
        let js = get_type_id(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }
}
