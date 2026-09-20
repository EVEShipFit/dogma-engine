use std::sync::OnceLock;

use wasm_bindgen::prelude::*;

use esf_data::{Error, InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{Fit, Options};

/// The SDE is handed over once and then read straight out of WASM memory, so
/// no lookup crosses back into JavaScript.
static SDE_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static SDE: OnceLock<Sde<'static>> = OnceLock::new();

/// `names.dat` is only read for a name `sde.dat` does not know, so it stays
/// optional; it is the bigger of the two files.
static NAMES_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static NAMES: OnceLock<Names<'static>> = OnceLock::new();

fn sde() -> Result<&'static Sde<'static>, JsError> {
    SDE.get()
        .ok_or_else(|| JsError::new("SDE is not loaded; call load_sde() first"))
}

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

/// Load `names.dat`, so an EFT written in another language than English also
/// imports. Optional; without it only English names match. Has to be called
/// after `load_sde`, and calling it twice is an error.
#[wasm_bindgen]
pub fn load_names(bytes: Vec<u8>) -> Result<i32, JsError> {
    let sde = sde()?;

    if NAMES.get().is_some() {
        return Err(JsError::new("names are already loaded"));
    }

    let bytes = NAMES_BYTES.get_or_init(|| bytes);
    let names = Names::new(bytes)?;

    /* Reject a mismatched pair here, rather than on every load_eft(). */
    let build_number = names.build_number();
    if sde.build_number() != build_number {
        return Err(Error::BuildMismatch {
            sde: sde.build_number(),
            names: build_number,
        }
        .into());
    }

    let _ = NAMES.set(names);

    Ok(build_number)
}

/// Load a fit from EFT, the text format EVE copies a fit to the clipboard in.
#[wasm_bindgen]
pub fn load_eft(eft: &str) -> Result<JsValue, JsError> {
    let sde = sde()?;

    let info = InfoNameSde::new(sde, NAMES.get())?;

    let fit =
        esf_format::eft::load_eft(&info, eft).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(serde_wasm_bindgen::to_value(&fit)?)
}

/// `js_options` may be left out; it then uses the defaults.
#[wasm_bindgen]
pub fn calculate(js_fit: JsValue, js_options: JsValue) -> Result<JsValue, JsError> {
    let sde = sde()?;

    let fit: Fit = serde_wasm_bindgen::from_value(js_fit)?;
    let options: Options = match js_options.is_undefined() || js_options.is_null() {
        true => Options::default(),
        false => serde_wasm_bindgen::from_value(js_options)?,
    };

    let info = InfoSde::new(sde);

    let calculation = esf_dogma_engine::calculate(&info, &fit, &options);
    Ok(serde_wasm_bindgen::to_value(&calculation)?)
}

/// What a beacon in space hands to every fit in there with it. Put the result
/// in `incoming` of a fit to have it applied.
#[wasm_bindgen]
pub fn beacon(type_id: i32) -> Result<JsValue, JsError> {
    let sde = sde()?;

    let info = InfoSde::new(sde);

    let projection = esf_dogma_engine::beacon(&info, type_id);
    Ok(serde_wasm_bindgen::to_value(&projection)?)
}
