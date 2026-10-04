//! esf/1, the EVEShip.fit format, as text and as binary.

mod binary;
mod canonical;
mod fit;
pub(crate) mod lookup;
mod model;
mod names;
mod resolve;
mod text;

use esf_data::InfoEsf;

pub use binary::{decode_base64url, encode_base64url};
pub use fit::{from_fit, main_fit, to_fit};
pub use model::{Entry, Error, EsfFit, Location, State};

use lookup::Lookup;

/// Whether a document is text, rather than binary.
pub fn is_text(data: &[u8]) -> bool {
    matches!(data.first(), Some(b'%' | b' '))
}

/// Read a text document into canonical fits, checking IDs against `given`.
fn read_text<I: InfoEsf>(
    lookup: &Lookup<I>,
    data: &[u8],
    given: Option<&[EsfFit]>,
) -> Result<Vec<EsfFit>, Error> {
    let blocks = text::parse(data)?;
    let fits = resolve::resolve(&blocks, lookup)?;
    if let Some(given) = given {
        verify(&fits, given)?;
    }
    Ok(fits
        .iter()
        .map(|fit| canonical::canonical_fit(lookup, &fits, fit))
        .collect())
}

fn verify(fits: &[resolve::Fit], given: &[EsfFit]) -> Result<(), Error> {
    let same = |resolved: Option<i32>, given: Option<i32>, what: &str| match resolved == given {
        true => Ok(()),
        false => Err(Error::new(format!(
            "binary: {what} ID {given:?} is not what its name resolves to ({resolved:?})"
        ))),
    };

    for (fit, given) in fits.iter().zip(given) {
        same(fit.hull, given.hull, "hull")?;
        same(fit.mode, given.mode, "tactical mode")?;
        for (item, entry) in fit.items.iter().zip(&given.entries) {
            same(item.type_id, entry.type_id, "type")?;
            same(item.charge, entry.charge, "charge")?;
            same(item.mutaplasmid, entry.mutaplasmid, "mutaplasmid")?;
            let resolved = item.overrides.iter().map(|(attribute_id, _)| *attribute_id);
            let wanted = entry
                .overrides
                .iter()
                .map(|(attribute_id, _)| *attribute_id);
            if !resolved.eq(wanted) {
                return Err(Error::new(
                    "binary: an attribute ID is not what its name resolves to",
                ));
            }
        }
    }
    Ok(())
}

/// Check fits against the rules, and put them in canonical form.
fn canonicalise<I: InfoEsf>(lookup: &Lookup<I>, fits: &[EsfFit]) -> Result<Vec<EsfFit>, Error> {
    let source = canonical::source(lookup, fits)?;
    read_text(lookup, source.as_bytes(), Some(fits))
}

/// Load the fits of an esf/1 document, text or binary, in canonical form.
pub fn load_esf(info: &impl InfoEsf, data: &[u8]) -> Result<Vec<EsfFit>, Error> {
    let lookup = Lookup { info };
    match is_text(data) {
        true => read_text(&lookup, data, None),
        false => canonicalise(&lookup, &binary::decode(data)?),
    }
}

/// Write fits as canonical esf/1 text.
pub fn save_esf(info: &impl InfoEsf, fits: &[EsfFit]) -> Result<String, Error> {
    let lookup = Lookup { info };
    canonical::render(&lookup, &canonicalise(&lookup, fits)?)
}

/// Write fits as binary esf/1.
pub fn save_esf_binary(info: &impl InfoEsf, fits: &[EsfFit]) -> Result<Vec<u8>, Error> {
    let lookup = Lookup { info };
    Ok(binary::encode(&canonicalise(&lookup, fits)?))
}
