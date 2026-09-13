use std::sync::OnceLock;

use wasm_bindgen::prelude::*;

use esf_data::{InfoSde, Sde};
use esf_dogma_engine::Options;
use esf_dogma_engine::fit::Fit;

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
    let sde = Sde::new(bytes)?;

    let build_number = sde.build_number();
    let _ = SDE.set(sde);

    Ok(build_number)
}

/// `js_options` may be left out; it then uses the defaults.
#[wasm_bindgen]
pub fn calculate(js_fit: JsValue, js_options: JsValue) -> Result<JsValue, JsError> {
    let Some(sde) = SDE.get() else {
        return Err(JsError::new("SDE is not loaded; call load_sde() first"));
    };

    let fit: Fit = serde_wasm_bindgen::from_value(js_fit)?;
    let options: Options = match js_options.is_undefined() || js_options.is_null() {
        true => Options::default(),
        false => serde_wasm_bindgen::from_value(js_options)?,
    };

    let info = InfoSde::new(sde);

    let calculation = esf_dogma_engine::calculate(&info, &fit, &options);
    Ok(serde_wasm_bindgen::to_value(&calculation)?)
}
