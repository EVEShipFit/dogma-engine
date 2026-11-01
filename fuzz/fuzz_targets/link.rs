#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let (version, payload) = data.split_once(':').unwrap_or(("v3", data));
    if let Ok(fit) = esf_format::link::load_link(&esf_fuzz::info_name(), version, payload) {
        esf_fuzz::check_loaded(fit);
    }
});
