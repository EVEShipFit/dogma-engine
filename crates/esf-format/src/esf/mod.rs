//! esf/1, the EVEShip.fit format, as text and as binary.

mod catalog;
mod fit;
pub(crate) mod lookup;

use std::fmt;

use esf_data::InfoEsf;

pub use ::esf::{Entry, EsfFit, Location, State};
pub use fit::{from_fit, main_fit, to_fit};

use catalog::SdeCatalog;

/// Why a fit could not be read, written or turned into an engine fit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The document or fit breaks a rule of esf/1.
    Esf(::esf::Error),
    /// The fit does not map onto the dogma engine.
    Fit(String),
}

impl From<::esf::Error> for Error {
    fn from(error: ::esf::Error) -> Self {
        Error::Esf(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Esf(error) => error.fmt(f),
            Error::Fit(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

/// Whether a document is text, rather than binary.
pub fn is_text(data: &[u8]) -> bool {
    ::esf::is_text(data)
}

/// Load the fits of an esf/1 document, text or binary, in canonical form.
pub fn load_esf(info: &impl InfoEsf, data: &[u8]) -> Result<Vec<EsfFit>, Error> {
    Ok(::esf::load(data, &SdeCatalog::new(info))?)
}

/// Write fits as canonical esf/1 text.
pub fn save_esf(info: &impl InfoEsf, fits: &[EsfFit]) -> Result<String, Error> {
    Ok(::esf::save(fits, &SdeCatalog::new(info))?)
}

/// Write fits as binary esf/1.
pub fn save_esf_binary(info: &impl InfoEsf, fits: &[EsfFit]) -> Result<Vec<u8>, Error> {
    Ok(::esf::save_binary(fits, &SdeCatalog::new(info))?)
}

/// Bytes as base64url without padding.
pub fn encode_base64url(data: &[u8]) -> String {
    ::esf::to_base64url(data)
}

/// Bytes from base64url, with or without padding.
pub fn decode_base64url(text: &str) -> Result<Vec<u8>, Error> {
    Ok(::esf::from_base64url(text)?)
}
