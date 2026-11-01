#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    match esf_format::dna::load_dna(&esf_fuzz::info(), data) {
        Ok(fit) => esf_fuzz::check_loaded(fit),
        Err(error) => drop(error.to_string()),
    }
});
