//! fleet/1, the boosters of a fit, stored next to the fits in an esf/1
//! document.
//!
//! fleet/1 belongs to this crate, not to esf/1. Readers that do not know it
//! skip it, and see the boosted fit, as it is the first fit in the document.
//!
//! # Text
//!
//! A `%fleet/1` block follows the fits. Its first line names the boosted fit,
//! and every next line names a booster. A line names a fit the way an esf/1
//! reference does: the hull, then the fit name in double quotes, if the fit
//! has one. A booster that is switched off ends its line with `!off`. Blank
//! lines and `//` comments are skipped.
//!
//! ```text
//! %esf/1
//! Rifter "Roamer"
//!
//! %esf/1
//! Claymore "Shield Boosts"
//!
//! Shield Command Burst I :Shield Harmonizing Charge
//!
//! %fleet/1
//! Rifter "Roamer"
//! Claymore "Shield Boosts"
//! ```
//!
//! # Binary
//!
//! Field 100 of the esf/1 `Document` message holds a `Fleet` message. It
//! points at fits by their index in `Document.fits`, starting at 0.
//!
//! ```protobuf
//! message Fleet {
//!   uint32 fit = 1;                // the boosted fit
//!   repeated Booster boosters = 2;
//! }
//!
//! message Booster {
//!   uint32 fit = 1;
//!   bool off = 2;                  // switched off
//! }
//! ```
//!
//! # Names
//!
//! Every fit a line names must be the only one in the document with that hull
//! and fit name. When saving, a booster without a name, or whose hull and
//! name another fit already has, is named `<hull> - Booster <N>`, with the
//! lowest `N` from 1 that is still free, like `Claymore - Booster 1`.

use std::collections::BTreeSet;

use ::esf::{Catalog, Document, Extra, fold};
use esf_data::InfoEsf;
use esf_dogma_engine::{Booster, Fit};

use super::catalog::SdeCatalog;
use super::{Error, EsfFit, from_fit, main_fit, to_fit};

const BINARY_FIELD: u32 = 100;

struct Member {
    fit: usize,
    off: bool,
}

/// Load the fit an esf/1 document is about, text or binary, with its
/// boosters. The fits have no skills.
///
/// Without a fleet/1 block, this is the first fit no other fit carries, and
/// it has no boosters.
pub fn load_fit(info: &impl InfoEsf, data: &[u8]) -> Result<Fit, Error> {
    let catalog = SdeCatalog::new(info);
    let document = ::esf::load_document(data, &catalog)?;

    let Some(members) = fleet(&document, &catalog)? else {
        let fit = main_fit(&document.fits)
            .ok_or_else(|| Error::Fit("no fit stands on its own".to_string()))?;
        return to_fit(info, fit);
    };

    let main_index = main_fit(&document.fits)
        .and_then(|main| document.fits.iter().position(|fit| std::ptr::eq(fit, main)));
    let mut members = members.into_iter();
    let main = members
        .next()
        .filter(|main| Some(main.fit) == main_index)
        .ok_or_else(|| Error::Fleet("a fleet starts with the first fit".to_string()))?;
    let mut seen = BTreeSet::from([main.fit]);
    if let Some(member) = members
        .as_slice()
        .iter()
        .find(|member| !seen.insert(member.fit))
    {
        return Err(Error::Fleet(format!(
            "a fleet names fit {} more than once",
            member.fit + 1
        )));
    }
    let mut fit = to_fit(info, &document.fits[main.fit])?;
    for member in members {
        fit.boosters.push(Booster {
            fit: to_fit(info, &document.fits[member.fit])?,
            off: member.off,
        });
    }
    Ok(fit)
}

/// Write a fit, and its boosters, as canonical esf/1 text.
pub fn save_fit(info: &impl InfoEsf, fit: &Fit) -> Result<String, Error> {
    let catalog = SdeCatalog::new(info);
    let fits = fleet_fits(info, &catalog, fit)?;

    let mut text = ::esf::save(&fits, &catalog)?;
    if fits.len() > 1 {
        text.push_str("\n%fleet/1\n");
        for (index, member) in fits.iter().enumerate() {
            text.push_str(&reference(&catalog, member));
            if index > 0 && fit.boosters[index - 1].off {
                text.push_str(" !off");
            }
            text.push('\n');
        }
    }
    Ok(text)
}

/// Write a fit, and its boosters, as binary esf/1.
pub fn save_fit_binary(info: &impl InfoEsf, fit: &Fit) -> Result<Vec<u8>, Error> {
    let catalog = SdeCatalog::new(info);
    let fits = fleet_fits(info, &catalog, fit)?;

    let mut binary = ::esf::save_binary(&fits, &catalog)?;
    if fits.len() > 1 {
        let mut message = Vec::new();
        for (index, booster) in fit.boosters.iter().enumerate() {
            let mut payload = Vec::new();
            wire::uint(&mut payload, 1, index as u64 + 1);
            wire::uint(&mut payload, 2, booster.off as u64);
            wire::record(&mut message, 2, &payload);
        }
        binary.extend(::esf::binary_field(BINARY_FIELD, &message));
    }
    Ok(binary)
}

fn fleet_fits(info: &impl InfoEsf, catalog: &dyn Catalog, fit: &Fit) -> Result<Vec<EsfFit>, Error> {
    let mut fits = vec![from_fit(info, fit)?];
    for booster in &fit.boosters {
        fits.push(from_fit(info, &booster.fit)?);
    }

    let mut taken = BTreeSet::new();
    let unique: Vec<bool> = fits
        .iter()
        .map(|fit| fit.name.is_some() && taken.insert(key(fit)))
        .collect();
    taken.insert(key(&fits[0]));

    for (fit, unique) in fits.iter_mut().zip(unique).skip(1) {
        if !unique {
            let hull = hull_name(catalog, fit.hull);
            fit.name = (1..)
                .map(|number| format!("{hull} - Booster {number}"))
                .find(|name| taken.insert((fit.hull, Some(fold(name)))));
        }
    }
    Ok(fits)
}

fn key(fit: &EsfFit) -> (Option<u32>, Option<String>) {
    (fit.hull, fit.name.as_deref().map(fold))
}

fn hull_name(catalog: &dyn Catalog, hull: Option<u32>) -> String {
    hull.and_then(|hull| catalog.type_by_id(hull))
        .map_or_else(|| "-".to_string(), |record| record.name.clone())
}

fn fleet(document: &Document, catalog: &dyn Catalog) -> Result<Option<Vec<Member>>, Error> {
    let mut found = None;
    for extra in &document.extra {
        let members = match extra {
            Extra::Text {
                name,
                version,
                text,
            } if name == "fleet" => {
                if *version != 1 {
                    return Err(Error::Fleet(format!("unsupported version fleet/{version}")));
                }
                parse(text, &document.fits, catalog)?
            }
            Extra::Binary { field, bytes } if *field == BINARY_FIELD => {
                decode(bytes, document.fits.len())?
            }
            _ => continue,
        };
        if found.replace(members).is_some() {
            return Err(Error::Fleet("a document holds one fleet".to_string()));
        }
    }
    Ok(found)
}

const SIGILS: &[u8] = b":+{!@\"/";

fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

fn reference(catalog: &dyn Catalog, fit: &EsfFit) -> String {
    let mut line = match fit.hull.and_then(|hull| catalog.type_by_id(hull)) {
        Some(record) => {
            let name = &record.name;
            let mut text = if needs_quotes(name) {
                quote(name)
            } else {
                name.clone()
            };
            if catalog
                .type_by_name(&fold(name))
                .is_none_or(|found| found.id != record.id)
            {
                text = format!("{text}#{}", record.id);
            }
            text
        }
        None => "-".to_string(),
    };
    if let Some(name) = &fit.name {
        line.push(' ');
        line.push_str(&quote(name));
    }
    line
}

fn needs_quotes(name: &str) -> bool {
    let first = name.split(' ').next().unwrap_or_default();
    first.starts_with(['-', '%'])
        || is_count(first)
        || split_type_id(name).is_some()
        || name
            .split(' ')
            .any(|word| word.as_bytes().first().is_none_or(|b| SIGILS.contains(b)))
}

fn is_count(token: &str) -> bool {
    token
        .strip_suffix('x')
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn split_type_id(name: &str) -> Option<(&str, u32)> {
    let (name, digits) = name.rsplit_once('#')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) || name.ends_with(' ') {
        return None;
    }
    Some((name, digits.parse().ok()?))
}

enum Token {
    Word(String),
    Quoted(String, Option<u32>),
}

fn tokens(line: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut rest = line.trim_matches(' ');
    while !rest.is_empty() {
        if let Some(quoted) = rest.strip_prefix('"') {
            let mut text = String::new();
            let mut chars = quoted.char_indices();
            let end = loop {
                match chars.next() {
                    Some((index, '"')) if quoted[index + 1..].starts_with('"') => {
                        text.push('"');
                        chars.next();
                    }
                    Some((index, '"')) => break index + 1,
                    Some((_, char)) => text.push(char),
                    None => return Err("a quoted name is not closed".to_string()),
                }
            };
            rest = &quoted[end..];
            let mut id = None;
            if let Some(after) = rest.strip_prefix('#') {
                let digits = after.find(' ').unwrap_or(after.len());
                id = Some(
                    after[..digits]
                        .parse()
                        .map_err(|_| format!("invalid type id {:?}", &after[..digits]))?,
                );
                rest = &after[digits..];
            }
            tokens.push(Token::Quoted(text, id));
        } else {
            let end = rest.find(' ').unwrap_or(rest.len());
            if rest.starts_with("//") {
                break;
            }
            tokens.push(Token::Word(rest[..end].to_string()));
            rest = &rest[end..];
        }
        rest = rest.trim_start_matches(' ');
    }
    Ok(tokens)
}

fn parse(text: &str, fits: &[EsfFit], catalog: &dyn Catalog) -> Result<Vec<Member>, Error> {
    let mut members = Vec::new();
    for (line, number) in text.lines().zip(1..) {
        let member = parse_line(line, fits, catalog)
            .map_err(|message| Error::Fleet(format!("fleet line {number}: {message}")))?;
        members.extend(member);
    }
    Ok(members)
}

fn parse_line(
    line: &str,
    fits: &[EsfFit],
    catalog: &dyn Catalog,
) -> Result<Option<Member>, String> {
    let mut tokens = tokens(line)?.into_iter().peekable();
    if tokens.peek().is_none() {
        return Ok(None);
    }

    let (hull, id) = match tokens.next() {
        Some(Token::Quoted(name, id)) => (name, id),
        Some(Token::Word(word)) if word == "-" => (word, None),
        Some(Token::Word(word)) => {
            let mut words = vec![word];
            while let Some(Token::Word(word)) = tokens.peek() {
                if word.as_bytes().first().is_some_and(|b| SIGILS.contains(b)) {
                    break;
                }
                words.push(word.clone());
                tokens.next();
            }
            let hull = words.join(" ");
            match split_type_id(&hull) {
                Some((name, id)) => (name.to_string(), Some(id)),
                None => (hull, None),
            }
        }
        None => unreachable!(),
    };
    let name = match tokens.peek() {
        Some(Token::Quoted(name, None)) => {
            let name = name.clone();
            tokens.next();
            Some(name)
        }
        _ => None,
    };
    let off = match tokens.next() {
        None => false,
        Some(Token::Word(word)) if word == "!off" => true,
        Some(Token::Word(word) | Token::Quoted(word, _)) => {
            return Err(format!("unexpected {word:?}"));
        }
    };
    if let Some(Token::Word(word) | Token::Quoted(word, _)) = tokens.next() {
        return Err(format!("unexpected {word:?}"));
    }

    let hull_id = match (hull.as_str(), id) {
        ("-", None) => None,
        (_, Some(id)) => {
            let record = catalog
                .type_by_id(id)
                .ok_or_else(|| format!("no such type: #{id}"))?;
            if fold(&record.name) != fold(&hull) {
                return Err(format!("type #{id} is {:?}, not {hull:?}", record.name));
            }
            Some(id)
        }
        (_, None) => Some(
            catalog
                .type_by_name(&fold(&hull))
                .ok_or_else(|| format!("no such type: {hull}"))?
                .id,
        ),
    };
    let wanted = (hull_id, name.as_deref().map(fold));
    let mut matches = fits
        .iter()
        .enumerate()
        .filter(|(_, fit)| key(fit) == wanted)
        .map(|(index, _)| index);
    match (matches.next(), matches.next()) {
        (Some(fit), None) => Ok(Some(Member { fit, off })),
        (None, _) => Err(format!("no fit is {line:?}")),
        (Some(_), Some(_)) => Err(format!("more than one fit is {line:?}")),
    }
}

fn decode(bytes: &[u8], fits: usize) -> Result<Vec<Member>, Error> {
    let invalid = |why: &str| Error::Fleet(format!("binary fleet: {why}"));
    let index = |value: u64| {
        usize::try_from(value)
            .ok()
            .filter(|index| *index < fits)
            .ok_or_else(|| invalid("points at a fit that is not there"))
    };

    let mut main = 0;
    let mut members = Vec::new();
    let mut reader = wire::Reader(bytes);
    while let Some((field, value)) = reader.field().map_err(invalid)? {
        match (field, value) {
            (1, wire::Value::Varint(value)) => main = value,
            (2, wire::Value::Len(payload)) => {
                let (mut fit, mut off) = (0, false);
                let mut reader = wire::Reader(payload);
                while let Some((field, value)) = reader.field().map_err(invalid)? {
                    match (field, value) {
                        (1, wire::Value::Varint(value)) => fit = value,
                        (2, wire::Value::Varint(value)) => off = value != 0,
                        _ => {}
                    }
                }
                members.push(Member {
                    fit: index(fit)?,
                    off,
                });
            }
            _ => {}
        }
    }
    members.insert(
        0,
        Member {
            fit: index(main)?,
            off: false,
        },
    );
    Ok(members)
}

mod wire {
    pub enum Value<'a> {
        Varint(u64),
        Len(&'a [u8]),
        Other,
    }

    pub fn varint(out: &mut Vec<u8>, mut value: u64) {
        while value >= 0x80 {
            out.push(value as u8 | 0x80);
            value >>= 7;
        }
        out.push(value as u8);
    }

    pub fn uint(out: &mut Vec<u8>, field: u32, value: u64) {
        if value != 0 {
            varint(out, u64::from(field) << 3);
            varint(out, value);
        }
    }

    pub fn record(out: &mut Vec<u8>, field: u32, payload: &[u8]) {
        varint(out, (u64::from(field) << 3) | 2);
        varint(out, payload.len() as u64);
        out.extend_from_slice(payload);
    }

    pub struct Reader<'a>(pub &'a [u8]);

    impl<'a> Reader<'a> {
        fn take(&mut self, length: u64) -> Result<&'a [u8], &'static str> {
            let length = usize::try_from(length)
                .ok()
                .filter(|length| *length <= self.0.len())
                .ok_or("a field runs past the end")?;
            let (taken, rest) = self.0.split_at(length);
            self.0 = rest;
            Ok(taken)
        }

        fn varint(&mut self) -> Result<u64, &'static str> {
            let mut value = 0u64;
            for shift in (0..64).step_by(7) {
                let byte = *self.take(1)?.first().ok_or("a varint is cut off")?;
                value |= u64::from(byte & 0x7f) << shift;
                if byte & 0x80 == 0 {
                    return Ok(value);
                }
            }
            Err("a varint is too long")
        }

        pub fn field(&mut self) -> Result<Option<(u32, Value<'a>)>, &'static str> {
            if self.0.is_empty() {
                return Ok(None);
            }
            let key = self.varint()?;
            let field = u32::try_from(key >> 3).map_err(|_| "a field number is too large")?;
            let value = match key & 7 {
                0 => Value::Varint(self.varint()?),
                1 => {
                    self.take(8)?;
                    Value::Other
                }
                2 => {
                    let length = self.varint()?;
                    Value::Len(self.take(length)?)
                }
                5 => {
                    self.take(4)?;
                    Value::Other
                }
                _ => return Err("a field has an unknown wire type"),
            };
            Ok(Some((field, value)))
        }
    }
}
