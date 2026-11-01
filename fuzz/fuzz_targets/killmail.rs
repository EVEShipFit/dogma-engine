#![no_main]

use esf_format::killmail::{EsiKillmail, load_killmail};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(killmail) = serde_json::from_slice::<EsiKillmail>(data) {
        esf_fuzz::check_loaded(load_killmail(&esf_fuzz::info(), &killmail));
    }
});
