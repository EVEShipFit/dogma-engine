use std::any::Any;
use std::fmt;
use std::sync::OnceLock;

use jni::errors::ErrorPolicy;
use jni::objects::{JByteArray, JClass, JString};
use jni::strings::JNIString;
use jni::sys::jint;
use jni::{Env, EnvUnowned, jni_mangle, jni_str};
use serde::Serialize;
use serde::de::DeserializeOwned;

use esf_data::{InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{Fit, Options};
use esf_format::esf;
use esf_format::esi::EsiFitting;
use esf_format::killmail::EsiKillmail;

static SDE: OnceLock<Sde<'static>> = OnceLock::new();
static NAMES: OnceLock<Names<'static>> = OnceLock::new();

#[derive(Debug)]
enum Error {
    /// Thrown as `IllegalArgumentException`.
    Argument(String),
    /// Thrown as `IllegalStateException`.
    State(String),
    Jni(jni::errors::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Argument(message) | Error::State(message) => formatter.write_str(message),
            Error::Jni(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Error {}

impl From<jni::errors::Error> for Error {
    fn from(error: jni::errors::Error) -> Self {
        Error::Jni(error)
    }
}

fn argument(error: impl ToString) -> Error {
    Error::Argument(error.to_string())
}

struct Throw;

impl<T: Default> ErrorPolicy<T, Error> for Throw {
    type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

    fn on_error<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut (),
        error: Error,
    ) -> jni::errors::Result<T> {
        if env.exception_check() {
            return Ok(T::default());
        }
        let class = match error {
            Error::Argument(_) => jni_str!("java/lang/IllegalArgumentException"),
            Error::State(_) => jni_str!("java/lang/IllegalStateException"),
            Error::Jni(_) => jni_str!("java/lang/RuntimeException"),
        };
        let _ = env.throw_new(class, JNIString::new(error.to_string()));
        Ok(T::default())
    }

    fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut (),
        payload: Box<dyn Any + Send + 'static>,
    ) -> jni::errors::Result<T> {
        let message = match payload.downcast::<&'static str>() {
            Ok(message) => message.to_string(),
            Err(payload) => match payload.downcast::<String>() {
                Ok(message) => *message,
                Err(_) => "unknown panic".to_string(),
            },
        };
        let _ = env.throw_new(
            jni_str!("java/lang/RuntimeException"),
            JNIString::new(format!("dogma-engine panicked: {message}")),
        );
        Ok(T::default())
    }
}

fn sde() -> Result<&'static Sde<'static>, Error> {
    SDE.get()
        .ok_or_else(|| Error::State("SDE is not loaded; call loadSde() first".to_string()))
}

fn from_json<T: DeserializeOwned>(env: &Env, json: &JString) -> Result<T, Error> {
    serde_json::from_str(&json.try_to_string(env)?).map_err(argument)
}

fn to_json(value: &impl Serialize) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|error| Error::State(error.to_string()))
}

/// Run `body` and hand its string back to the JVM, or throw what it failed with.
fn string<'local>(
    mut unowned_env: EnvUnowned<'local>,
    body: impl FnOnce(&mut Env<'local>) -> Result<String, Error>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> Result<JString<'local>, Error> {
            let value = body(env)?;
            Ok(JString::from_str(env, value)?)
        })
        .resolve::<Throw>()
}

fn int(mut unowned_env: EnvUnowned, body: impl FnOnce(&mut Env) -> Result<jint, Error>) -> jint {
    unowned_env.with_env(body).resolve::<Throw>()
}

/* Leaked, as everything read from the SDE borrows it for the rest of the
 * process. */
fn leak(env: &Env, bytes: &JByteArray) -> Result<&'static [u8], Error> {
    Ok(Box::leak(env.convert_byte_array(bytes)?.into_boxed_slice()))
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_sde<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    bytes: JByteArray<'local>,
) -> jint {
    int(unowned_env, |env| {
        if SDE.get().is_some() {
            return Err(Error::State("SDE is already loaded".to_string()));
        }

        let sde = Sde::new(leak(env, &bytes)?).map_err(argument)?;
        let build_number = sde.build_number();
        SDE.set(sde)
            .map_err(|_| Error::State("SDE is already loaded".to_string()))?;

        Ok(build_number)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_names<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    bytes: JByteArray<'local>,
) -> jint {
    int(unowned_env, |env| {
        let sde = sde()?;
        if NAMES.get().is_some() {
            return Err(Error::State("names are already loaded".to_string()));
        }

        let names = Names::new(leak(env, &bytes)?).map_err(argument)?;
        let build_number = names.build_number();
        if sde.build_number() != build_number {
            return Err(argument(esf_data::Error::BuildMismatch {
                sde: sde.build_number(),
                names: build_number,
            }));
        }
        NAMES
            .set(names)
            .map_err(|_| Error::State("names are already loaded".to_string()))?;

        Ok(build_number)
    })
}

fn info_name_sde() -> Result<InfoNameSde<'static>, Error> {
    InfoNameSde::new(sde()?, NAMES.get()).map_err(|error| Error::State(error.to_string()))
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_eft<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    eft: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let eft = eft.try_to_string(env)?;
        let fit = esf_format::eft::load_eft(&info_name_sde()?, &eft).map_err(argument)?;
        to_json(&fit)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn save_eft<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let fit: Fit = from_json(env, &fit)?;
        esf_format::eft::save_eft(&InfoSde::new(sde()?), &fit).map_err(argument)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_esi_fitting<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fitting: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let fitting: EsiFitting = from_json(env, &fitting)?;
        to_json(&esf_format::esi::load_esi_fitting(
            &InfoSde::new(sde()?),
            &fitting,
        ))
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn save_esi_fitting<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let fit: Fit = from_json(env, &fit)?;
        to_json(&esf_format::esi::save_esi_fitting(
            &InfoSde::new(sde()?),
            &fit,
        ))
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn post_load<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let mut fit: Fit = from_json(env, &fit)?;
        esf_format::post_load(&InfoSde::new(sde()?), &mut fit);
        to_json(&fit)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_killmail<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    killmail: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let killmail: EsiKillmail = from_json(env, &killmail)?;
        to_json(&esf_format::killmail::load_killmail(
            &InfoSde::new(sde()?),
            &killmail,
        ))
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_dna<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    dna: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let dna = dna.try_to_string(env)?;
        let fit = esf_format::dna::load_dna(&InfoSde::new(sde()?), &dna).map_err(argument)?;
        to_json(&fit)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_link<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    version: JString<'local>,
    payload: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let version = version.try_to_string(env)?;
        let payload = payload.try_to_string(env)?;
        let fit =
            esf_format::link::load_link(&info_name_sde()?, &version, &payload).map_err(argument)?;
        to_json(&fit)
    })
}

fn load_esf_fit(data: &[u8]) -> Result<Fit, Error> {
    let info = InfoSde::new(sde()?);

    let fits = esf::load_esf(&info, data).map_err(argument)?;
    let fit = esf::main_fit(&fits).ok_or_else(|| argument("no fit stands on its own"))?;
    esf::to_fit(&info, fit).map_err(argument)
}

fn to_esf_fit(info: &InfoSde, env: &Env, fit: &JString) -> Result<esf::EsfFit, Error> {
    let fit: Fit = from_json(env, fit)?;
    esf::from_fit(info, &fit).map_err(argument)
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_esf<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    text: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let text = text.try_to_string(env)?;
        to_json(&load_esf_fit(text.as_bytes())?)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn load_esf_link<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    link: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let link = link.try_to_string(env)?;
        to_json(&load_esf_fit(
            &esf::decode_base64url(&link).map_err(argument)?,
        )?)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn save_esf<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let info = InfoSde::new(sde()?);
        let esf_fit = to_esf_fit(&info, env, &fit)?;
        esf::save_esf(&info, &[esf_fit]).map_err(argument)
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn save_esf_link<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let info = InfoSde::new(sde()?);
        let esf_fit = to_esf_fit(&info, env, &fit)?;
        let binary = esf::save_esf_binary(&info, &[esf_fit]).map_err(argument)?;
        Ok(esf::encode_base64url(&binary))
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn calculate<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fit: JString<'local>,
    options: JString<'local>,
) -> JString<'local> {
    string(unowned_env, |env| {
        let fit: Fit = from_json(env, &fit)?;
        let options: Options = from_json(env, &options)?;
        to_json(&esf_dogma_engine::calculate(
            &InfoSde::new(sde()?),
            &fit,
            &options,
        ))
    })
}

#[jni_mangle("fit.eveship.dogma.Native")]
pub fn beacon<'local>(
    unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    type_id: jint,
) -> JString<'local> {
    string(unowned_env, |_env| {
        to_json(&esf_dogma_engine::beacon(&InfoSde::new(sde()?), type_id))
    })
}
