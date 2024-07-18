use serde_wasm_bindgen;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

use super::calculate;
use super::data_types;
use super::info::Info;

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

pub struct InfoWasm {
    pub fit: data_types::EsfFit,
    pub skills: BTreeMap<i32, i32>,
}

impl Info for InfoWasm {
    fn get_dogma_attributes(&self, type_id: i32) -> Vec<data_types::TypeDogmaAttribute> {
        let js = get_dogma_attributes(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    fn get_dogma_attribute(&self, attribute_id: i32) -> data_types::DogmaAttribute {
        let js = get_dogma_attribute(attribute_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    fn get_dogma_effects(&self, type_id: i32) -> Vec<data_types::TypeDogmaEffect> {
        let js = get_dogma_effects(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    fn get_dogma_effect(&self, effect_id: i32) -> data_types::DogmaEffect {
        let js = get_dogma_effect(effect_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    fn get_type_id(&self, type_id: i32) -> data_types::TypeId {
        let js = get_type_id(type_id);
        serde_wasm_bindgen::from_value(js).unwrap()
    }

    fn skills(&self) -> &BTreeMap<i32, i32> {
        &self.skills
    }

    fn fit(&self) -> &data_types::EsfFit {
        &self.fit
    }
}

impl InfoWasm {
    pub fn new<'a>(fit: data_types::EsfFit, skills: BTreeMap<i32, i32>) -> InfoWasm {
        InfoWasm { fit, skills }
    }
}

#[wasm_bindgen]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn calculate(js_esf_fit: JsValue, js_skills: JsValue) -> JsValue {
    let fit: data_types::EsfFit = serde_wasm_bindgen::from_value(js_esf_fit).unwrap();
    let skills: BTreeMap<String, i32> = serde_wasm_bindgen::from_value(js_skills).unwrap();
    let skills = skills
        .into_iter()
        .map(|(k, v)| (k.parse::<i32>().unwrap(), v))
        .collect();

    let info = InfoWasm::new(fit, skills);

    let statistics = calculate::calculate(&info);
    serde_wasm_bindgen::to_value(&statistics).unwrap()
}