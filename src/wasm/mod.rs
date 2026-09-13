use std::collections::BTreeMap;
use std::sync::OnceLock;

use wasm_bindgen::prelude::*;

use crate::calculate;
use crate::data_types;
use crate::sde::{InfoSde, Sde};

/// The SDE is handed over once and then read straight out of WASM memory, so
/// no lookup crosses back into JavaScript.
static SDE_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static SDE: OnceLock<Sde<'static>> = OnceLock::new();

#[wasm_bindgen]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

/// Load `sde.dat`. Has to be called before `calculate`; calling it twice is an
/// error, as the first buffer is borrowed for the rest of the session.
#[wasm_bindgen]
pub fn load_sde(bytes: Vec<u8>) -> Result<i32, JsError> {
    if SDE.get().is_some() {
        return Err(JsError::new("SDE is already loaded"));
    }

    let bytes = SDE_BYTES.get_or_init(|| bytes);
    let sde = Sde::new(bytes).map_err(|error| JsError::new(&error))?;

    let build_number = sde.build_number();
    let _ = SDE.set(sde);

    Ok(build_number)
}

#[wasm_bindgen]
pub fn calculate(js_esf_fit: JsValue, js_skills: JsValue) -> Result<JsValue, JsError> {
    let Some(sde) = SDE.get() else {
        return Err(JsError::new("SDE is not loaded; call load_sde() first"));
    };

    let fit: data_types::EsfFit = serde_wasm_bindgen::from_value(js_esf_fit)?;
    let skills: BTreeMap<String, i32> = serde_wasm_bindgen::from_value(js_skills)?;
    let skills = skills
        .into_iter()
        .map(|(skill_id, level)| (skill_id.parse::<i32>().unwrap(), level))
        .collect();

    let info = InfoSde::new(fit, skills, sde);

    let statistics = calculate::calculate(&info);
    Ok(serde_wasm_bindgen::to_value(&statistics)?)
}
