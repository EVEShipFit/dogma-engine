use std::panic::{self, AssertUnwindSafe};
use std::sync::OnceLock;

use serde::Serialize;
use serde::de::DeserializeOwned;

use esf_data::{InfoNameSde, Names, Sde};
use esf_dogma_engine::{Fit, Options};
use esf_format::esf;
use esf_format::esi::EsiFitting;
use esf_format::killmail::EsiKillmail;

static SDE: OnceLock<Sde> = OnceLock::new();
static NAMES: OnceLock<Names> = OnceLock::new();

enum Error {
    /// Thrown as `ArgumentException`.
    Argument(String),
    /// Thrown as `InvalidOperationException`.
    State(String),
}

fn argument(error: impl ToString) -> Error {
    Error::Argument(error.to_string())
}

fn state(error: impl ToString) -> Error {
    Error::State(error.to_string())
}

#[repr(i32)]
enum Status {
    Ok = 0,
    Argument = 1,
    State = 2,
    Panic = 3,
}

/// What every call returns; `text` is the result or the error, and is freed with `esf_free`.
#[repr(C)]
pub struct EsfResult {
    status: i32,
    value: i32,
    text: *mut u8,
    length: usize,
}

impl EsfResult {
    fn new(status: Status, value: i32, text: Option<String>) -> Self {
        let (text, length) = match text {
            Some(text) => {
                let bytes = Box::into_raw(text.into_bytes().into_boxed_slice());
                (bytes.cast::<u8>(), bytes.len())
            }
            None => (std::ptr::null_mut(), 0),
        };
        Self {
            status: status as i32,
            value,
            text,
            length,
        }
    }

    fn text(text: String) -> Self {
        Self::new(Status::Ok, 0, Some(text))
    }

    fn number(value: i32) -> Self {
        Self::new(Status::Ok, value, None)
    }
}

fn guard(body: impl FnOnce() -> Result<EsfResult, Error>) -> EsfResult {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(Ok(result)) => result,
        Ok(Err(Error::Argument(message))) => EsfResult::new(Status::Argument, 0, Some(message)),
        Ok(Err(Error::State(message))) => EsfResult::new(Status::State, 0, Some(message)),
        Err(payload) => {
            let message = match payload.downcast::<&'static str>() {
                Ok(message) => message.to_string(),
                Err(payload) => match payload.downcast::<String>() {
                    Ok(message) => *message,
                    Err(_) => "unknown panic".to_string(),
                },
            };
            EsfResult::new(
                Status::Panic,
                0,
                Some(format!("dogma-engine panicked: {message}")),
            )
        }
    }
}

/// Free the `text` of an `EsfResult`.
///
/// # Safety
///
/// `text` and `length` must come from one `EsfResult`, and be freed only once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn esf_free(text: *mut u8, length: usize) {
    if !text.is_null() {
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(text, length)) });
    }
}

unsafe fn bytes<'a>(data: *const u8, length: usize) -> &'a [u8] {
    if length == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, length) }
    }
}

unsafe fn text<'a>(data: *const u8, length: usize) -> Result<&'a str, Error> {
    std::str::from_utf8(unsafe { bytes(data, length) }).map_err(argument)
}

fn sde() -> Result<&'static Sde, Error> {
    SDE.get()
        .ok_or_else(|| state("SDE is not loaded; call LoadSde() first"))
}

fn info_name_sde() -> Result<InfoNameSde<'static>, Error> {
    InfoNameSde::new(sde()?, NAMES.get()).map_err(state)
}

fn from_json<T: DeserializeOwned>(json: &str) -> Result<T, Error> {
    serde_json::from_str(json).map_err(argument)
}

fn to_json(value: &impl Serialize) -> Result<EsfResult, Error> {
    Ok(EsfResult::text(
        serde_json::to_string(value).map_err(state)?,
    ))
}

/// Load `sde.dat`; `value` is its build number.
///
/// # Safety
///
/// `data` must point to `length` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn esf_load_sde(data: *const u8, length: usize) -> EsfResult {
    let data = unsafe { bytes(data, length) };
    guard(|| {
        if SDE.get().is_some() {
            return Err(state("SDE is already loaded"));
        }

        let sde = Sde::new(data.to_vec()).map_err(argument)?;
        let build_number = sde.build_number();
        SDE.set(sde).map_err(|_| state("SDE is already loaded"))?;

        Ok(EsfResult::number(build_number))
    })
}

/// Load `names.dat`; `value` is its build number.
///
/// # Safety
///
/// `data` must point to `length` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn esf_load_names(data: *const u8, length: usize) -> EsfResult {
    let data = unsafe { bytes(data, length) };
    guard(|| {
        let sde = sde()?;
        if NAMES.get().is_some() {
            return Err(state("names are already loaded"));
        }

        let names = Names::new(data.to_vec()).map_err(argument)?;
        let build_number = names.build_number();
        if sde.build_number() != build_number {
            return Err(argument(esf_data::Error::BuildMismatch {
                sde: sde.build_number(),
                names: build_number,
            }));
        }
        NAMES
            .set(names)
            .map_err(|_| state("names are already loaded"))?;

        Ok(EsfResult::number(build_number))
    })
}

/// Declare a function taking one UTF-8 string, and returning a string.
macro_rules! text_function {
    ($name:ident, |$input:ident| $body:expr) => {
        /// # Safety
        ///
        /// `data` must point to `length` readable bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(data: *const u8, length: usize) -> EsfResult {
            let input = unsafe { text(data, length) };
            guard(|| {
                let $input = input?;
                $body
            })
        }
    };
}

text_function!(esf_load_eft, |eft| {
    to_json(&esf_format::eft::load_eft(&info_name_sde()?, eft).map_err(argument)?)
});

text_function!(esf_save_eft, |fit| {
    let fit: Fit = from_json(fit)?;
    Ok(EsfResult::text(
        esf_format::eft::save_eft(sde()?, &fit).map_err(argument)?,
    ))
});

text_function!(esf_load_esi_fitting, |fitting| {
    let fitting: EsiFitting = from_json(fitting)?;
    to_json(&esf_format::esi::load_esi_fitting(sde()?, &fitting))
});

text_function!(esf_save_esi_fitting, |fit| {
    let fit: Fit = from_json(fit)?;
    to_json(&esf_format::esi::save_esi_fitting(sde()?, &fit))
});

text_function!(esf_post_load, |fit| {
    let mut fit: Fit = from_json(fit)?;
    esf_format::post_load(sde()?, &mut fit);
    to_json(&fit)
});

text_function!(esf_load_killmail, |killmail| {
    let killmail: EsiKillmail = from_json(killmail)?;
    to_json(&esf_format::killmail::load_killmail(sde()?, &killmail))
});

text_function!(esf_load_dna, |dna| {
    to_json(&esf_format::dna::load_dna(sde()?, dna).map_err(argument)?)
});

fn load_esf_fit(data: &[u8]) -> Result<Fit, Error> {
    let sde = sde()?;

    let fits = esf::load_esf(sde, data).map_err(argument)?;
    let fit = esf::main_fit(&fits).ok_or_else(|| argument("no fit stands on its own"))?;
    esf::to_fit(sde, fit).map_err(argument)
}

fn to_esf_fit(sde: &Sde, fit: &str) -> Result<esf::EsfFit, Error> {
    let fit: Fit = from_json(fit)?;
    esf::from_fit(sde, &fit).map_err(argument)
}

text_function!(esf_load_esf, |esf_text| {
    to_json(&load_esf_fit(esf_text.as_bytes())?)
});

text_function!(esf_load_esf_link, |link| {
    to_json(&load_esf_fit(
        &esf::decode_base64url(link).map_err(argument)?,
    )?)
});

text_function!(esf_save_esf, |fit| {
    let sde = sde()?;
    let esf_fit = to_esf_fit(sde, fit)?;
    Ok(EsfResult::text(
        esf::save_esf(sde, &[esf_fit]).map_err(argument)?,
    ))
});

text_function!(esf_save_esf_link, |fit| {
    let sde = sde()?;
    let esf_fit = to_esf_fit(sde, fit)?;
    let binary = esf::save_esf_binary(sde, &[esf_fit]).map_err(argument)?;
    Ok(EsfResult::text(esf::encode_base64url(&binary)))
});

/// Load a fit from an EVEShip.fit link, given its version and payload.
///
/// # Safety
///
/// Each of `version` and `payload` must point to its length of readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn esf_load_link(
    version: *const u8,
    version_length: usize,
    payload: *const u8,
    payload_length: usize,
) -> EsfResult {
    let version = unsafe { text(version, version_length) };
    let payload = unsafe { text(payload, payload_length) };
    guard(|| {
        to_json(
            &esf_format::link::load_link(&info_name_sde()?, version?, payload?)
                .map_err(argument)?,
        )
    })
}

/// Calculate a fit, both given as JSON.
///
/// # Safety
///
/// Each of `fit` and `options` must point to its length of readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn esf_calculate(
    fit: *const u8,
    fit_length: usize,
    options: *const u8,
    options_length: usize,
) -> EsfResult {
    let fit = unsafe { text(fit, fit_length) };
    let options = unsafe { text(options, options_length) };
    guard(|| {
        let fit: Fit = from_json(fit?)?;
        let options: Options = from_json(options?)?;
        to_json(&esf_dogma_engine::calculate(sde()?, &fit, &options))
    })
}

/// What a beacon in space hands to every fit in there with it.
#[unsafe(no_mangle)]
pub extern "C" fn esf_beacon(type_id: i32) -> EsfResult {
    guard(|| to_json(&esf_dogma_engine::beacon(sde()?, type_id)))
}
