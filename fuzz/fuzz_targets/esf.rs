#![no_main]

use esf_format::esf;
use libfuzzer_sys::fuzz_target;

fn load(data: &[u8]) -> Result<(), esf::Error> {
    let info = esf_fuzz::info();
    let fits = esf::load_esf(&info, data)?;
    let _ = esf::main_fit(&fits);
    for fit in &fits {
        esf_fuzz::check_loaded(esf::to_fit(&info, fit)?);
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let result = match data.strip_prefix(b"link:") {
        Some(link) => {
            esf::decode_base64url(&String::from_utf8_lossy(link)).and_then(|data| load(&data))
        }
        None => load(data),
    };
    if let Err(error) = result {
        drop(error.to_string());
    }
});
