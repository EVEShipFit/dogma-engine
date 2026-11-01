#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    match esf_format::eft::load_eft(&esf_fuzz::info_name(), data) {
        Ok(fit) => esf_fuzz::check_loaded(fit),
        Err(error) => drop(error.to_string()),
    }
});
