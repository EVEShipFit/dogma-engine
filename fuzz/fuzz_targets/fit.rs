#![no_main]

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(fit) = esf_fuzz::fit::fit(&mut Unstructured::new(data)) {
        esf_fuzz::check(&fit);
    }
});
