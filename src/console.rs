use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

#[allow(unused_macros)]
macro_rules! log {
    ($($t:tt)*) => (crate::console::js_log(&format_args!($($t)*).to_string()))
}

#[allow(unused_imports)]
pub(crate) use log;
