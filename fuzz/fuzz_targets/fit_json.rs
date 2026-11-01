#![no_main]

use esf_dogma_engine::Fit;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(fit) = serde_json::from_slice::<Fit>(data) {
        esf_fuzz::check(&fit);
    }
});
