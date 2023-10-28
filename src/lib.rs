use serde_wasm_bindgen;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

mod calculate;
mod console;
mod data_types;

#[wasm_bindgen]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn calculate(js_ship_layout: JsValue, js_skills: JsValue) -> JsValue {
    let ship_layout: data_types::ShipLayout =
        serde_wasm_bindgen::from_value(js_ship_layout).unwrap();
    let skills: BTreeMap<String, i32> = serde_wasm_bindgen::from_value(js_skills).unwrap();
    let skills = skills
        .into_iter()
        .map(|(k, v)| (k.parse::<i32>().unwrap(), v))
        .collect();

    let statistics = calculate::calculate(&ship_layout, &skills);
    serde_wasm_bindgen::to_value(&statistics).unwrap()
}
