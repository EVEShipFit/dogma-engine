#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    if let Ok(fit) = esf_format::dna::load_dna(&esf_fuzz::info(), data) {
        esf_fuzz::check_loaded(fit);
    }
});
