#![no_main]

use esf_format::esi::{EsiFitting, load_esi_fitting};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(fitting) = serde_json::from_slice::<EsiFitting>(data) {
        esf_fuzz::check_loaded(load_esi_fitting(&esf_fuzz::info(), &fitting));
    }
});
