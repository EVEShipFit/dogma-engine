#![no_main]

use esf_format::esf;
use libfuzzer_sys::fuzz_target;

fn load(data: &[u8]) {
    let info = esf_fuzz::info();
    let Ok(fits) = esf::load_esf(&info, data) else {
        return;
    };
    let _ = esf::main_fit(&fits);
    for fit in &fits {
        if let Ok(fit) = esf::to_fit(&info, fit) {
            esf_fuzz::check_loaded(fit);
        }
    }
}

fuzz_target!(|data: &[u8]| {
    match data.strip_prefix(b"link:") {
        Some(link) => {
            if let Ok(decoded) = esf::decode_base64url(&String::from_utf8_lossy(link)) {
                load(&decoded);
            }
        }
        None => load(data),
    }
});
