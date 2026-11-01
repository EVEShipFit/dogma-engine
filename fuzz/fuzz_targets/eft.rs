#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    if let Ok(fit) = esf_format::eft::load_eft(&esf_fuzz::info_name(), data) {
        esf_fuzz::check_loaded(fit);
    }
});
