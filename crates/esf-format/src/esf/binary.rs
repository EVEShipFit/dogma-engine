//! The binary form: Protocol Buffers, as `esf.proto` describes.

use super::model::{Entry, Error, EsfFit, Location, State};

const VARINT: u8 = 0;
const FIXED64: u8 = 1;
const LENGTH: u8 = 2;
const FIXED32: u8 = 5;

fn invalid(message: impl Into<String>) -> Error {
    Error::new(format!("binary: {}", message.into()))
}

struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    fn varint(&mut self) -> Result<u64, Error> {
        let mut value = 0u64;
        for (index, byte) in self.data.iter().enumerate().take(10) {
            value |= u64::from(byte & 0x7f) << (7 * index);
            if byte & 0x80 == 0 {
                self.data = &self.data[index + 1..];
                return Ok(value);
            }
        }
        Err(invalid("truncated or overlong varint"))
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], Error> {
        if self.data.len() < length {
            return Err(invalid("truncated"));
        }
        let (taken, rest) = self.data.split_at(length);
        self.data = rest;
        Ok(taken)
    }

    fn bytes(&mut self) -> Result<&'a [u8], Error> {
        let length = usize::try_from(self.varint()?).map_err(|_| invalid("truncated"))?;
        self.take(length)
    }

    fn tag(&mut self) -> Result<Option<(u64, u8)>, Error> {
        if self.data.is_empty() {
            return Ok(None);
        }
        let tag = self.varint()?;
        let field = tag >> 3;
        if field == 0 {
            return Err(invalid("field number 0"));
        }
        Ok(Some((field, (tag & 7) as u8)))
    }

    fn skip(&mut self, wire: u8) -> Result<(), Error> {
        match wire {
            VARINT => self.varint().map(|_| ()),
            FIXED64 => self.take(8).map(|_| ()),
            LENGTH => self.bytes().map(|_| ()),
            FIXED32 => self.take(4).map(|_| ()),
            _ => Err(invalid(format!("unsupported wire type {wire}"))),
        }
    }
}

fn uint32(value: u64) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| invalid(format!("{value} does not fit in uint32")))
}

fn id(value: u64) -> Result<i32, Error> {
    i32::try_from(value).map_err(|_| invalid(format!("unknown ID {value}")))
}

fn string(bytes: &[u8]) -> Result<Option<String>, Error> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("a string is not valid UTF-8"))?;
    Ok((!text.is_empty()).then(|| text.to_string()))
}

fn not_in_schema(message: &str) -> Error {
    invalid(format!("{message} has a field that is not in the schema"))
}

fn decode_entry(data: &[u8]) -> Result<Entry, Error> {
    let mut reader = Reader { data };
    let mut entry = Entry::default();
    let mut attributes = Vec::new();
    let mut values = Vec::new();

    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, VARINT) => entry.type_id = Some(id(reader.varint()?)?),
            (2, VARINT) => entry.count = Some(uint32(reader.varint()?)?),
            (3, LENGTH) => entry.fit_name = string(reader.bytes()?)?,
            (4, VARINT) => entry.charge = Some(id(reader.varint()?)?),
            (5, VARINT) => entry.charge_count = Some(uint32(reader.varint()?)?),
            (6, VARINT) => entry.mutaplasmid = Some(id(reader.varint()?)?),
            (7, VARINT) => attributes.push(id(reader.varint()?)?),
            (7, LENGTH) => {
                let mut packed = Reader {
                    data: reader.bytes()?,
                };
                while !packed.data.is_empty() {
                    attributes.push(id(packed.varint()?)?);
                }
            }
            (8, FIXED64) => values.push(f64::from_le_bytes(reader.take(8)?.try_into().unwrap())),
            (8, LENGTH) => {
                let packed = reader.bytes()?;
                if packed.len() % 8 != 0 {
                    return Err(invalid("truncated"));
                }
                values.extend(
                    packed
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|chunk| f64::from_le_bytes(*chunk)),
                );
            }
            (9, VARINT) => {
                entry.state = match reader.varint()? {
                    0 => None,
                    value => Some(
                        State::ALL
                            .into_iter()
                            .find(|state| state.number() == value)
                            .ok_or_else(|| invalid(format!("unknown State value {value}")))?,
                    ),
                };
            }
            (10, VARINT) => {
                entry.location = match reader.varint()? {
                    0 => None,
                    value => Some(
                        Location::from_number(value)
                            .ok_or_else(|| invalid(format!("unknown Location value {value}")))?,
                    ),
                };
            }
            _ => return Err(not_in_schema("Entry")),
        }
    }

    if attributes.len() != values.len() {
        return Err(invalid("the override fields have different lengths"));
    }
    entry.overrides = attributes.into_iter().zip(values).collect();
    Ok(entry)
}

fn decode_fit(data: &[u8]) -> Result<EsfFit, Error> {
    let mut reader = Reader { data };
    let mut fit = EsfFit::default();
    let mut version = 0;

    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, VARINT) => version = reader.varint()?,
            (2, VARINT) => fit.hull = Some(id(reader.varint()?)?),
            (3, LENGTH) => fit.name = string(reader.bytes()?)?,
            (4, VARINT) => fit.mode = Some(id(reader.varint()?)?),
            (5, LENGTH) => fit.entries.push(decode_entry(reader.bytes()?)?),
            _ => return Err(not_in_schema("Fit")),
        }
    }

    if version != 1 {
        return Err(invalid(format!("unsupported Fit.version {version}")));
    }
    Ok(fit)
}

/// The fits of a binary document, as it holds them.
pub(super) fn decode(data: &[u8]) -> Result<Vec<EsfFit>, Error> {
    if data.is_empty() {
        return Err(invalid("the document is empty"));
    }
    let mut reader = Reader { data };
    let mut fits = Vec::new();
    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, LENGTH) => fits.push(decode_fit(reader.bytes()?)?),
            (1, _) => return Err(invalid("Document.fits has the wrong wire type")),
            _ => reader.skip(wire)?,
        }
    }
    Ok(fits)
}

#[derive(Default)]
struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.out.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.out.push(value as u8);
    }

    fn tag(&mut self, field: u64, wire: u8) {
        self.varint(field << 3 | u64::from(wire));
    }

    fn uint(&mut self, field: u64, value: Option<impl Into<u64>>) {
        if let Some(value) = value {
            self.tag(field, VARINT);
            self.varint(value.into());
        }
    }

    fn id(&mut self, field: u64, value: Option<i32>) {
        self.uint(field, value.map(|value| value as u32));
    }

    fn bytes(&mut self, field: u64, bytes: &[u8]) {
        self.tag(field, LENGTH);
        self.varint(bytes.len() as u64);
        self.out.extend_from_slice(bytes);
    }

    fn string(&mut self, field: u64, text: &Option<String>) {
        if let Some(text) = text.as_ref().filter(|text| !text.is_empty()) {
            self.bytes(field, text.as_bytes());
        }
    }
}

fn encode_entry(entry: &Entry) -> Vec<u8> {
    let mut writer = Writer::default();
    writer.id(1, entry.type_id);
    writer.uint(2, entry.count);
    writer.string(3, &entry.fit_name);
    writer.id(4, entry.charge);
    writer.uint(5, entry.charge_count);
    writer.id(6, entry.mutaplasmid);
    if !entry.overrides.is_empty() {
        let mut attributes = Writer::default();
        let mut values = Vec::new();
        for (attribute_id, value) in &entry.overrides {
            attributes.varint(u64::from(*attribute_id as u32));
            values.extend_from_slice(&value.to_le_bytes());
        }
        writer.bytes(7, &attributes.out);
        writer.bytes(8, &values);
    }
    writer.uint(9, entry.state.map(State::number));
    writer.uint(10, entry.location.map(Location::number));
    writer.out
}

/// A binary document of fits in canonical form.
pub(super) fn encode(fits: &[EsfFit]) -> Vec<u8> {
    let mut document = Writer::default();
    for fit in fits {
        let mut writer = Writer::default();
        writer.uint(1, Some(1u32));
        writer.id(2, fit.hull);
        writer.string(3, &fit.name);
        writer.id(4, fit.mode);
        for entry in &fit.entries {
            writer.bytes(5, &encode_entry(entry));
        }
        document.bytes(1, &writer.out);
    }
    document.out
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Bytes as base64url without padding.
pub fn encode_base64url(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let bits = chunk.iter().enumerate().fold(0u32, |bits, (index, byte)| {
            bits | u32::from(*byte) << (16 - 8 * index)
        });
        for index in 0..=chunk.len() {
            out.push(ALPHABET[(bits >> (18 - 6 * index) & 63) as usize] as char);
        }
    }
    out
}

/// Bytes from base64url, with or without padding.
pub fn decode_base64url(text: &str) -> Result<Vec<u8>, Error> {
    let text = text.trim_end_matches('=');
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut bits = 0u32;
    let mut count = 0;
    for c in text.bytes() {
        let value = ALPHABET
            .iter()
            .position(|letter| *letter == c)
            .ok_or_else(|| Error::new(format!("{:?} is not base64url", c as char)))?;
        bits = bits << 6 | value as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url() {
        for data in [&b""[..], b"f", b"fo", b"foo", b"foob", b"\xfb\xff"] {
            assert_eq!(decode_base64url(&encode_base64url(data)).unwrap(), data);
        }
        assert_eq!(encode_base64url(b"\xfb\xff"), "-_8");
    }
}
