//! EVEShip.fit links, which carry a fit in the `fit` value of their URL.
//!
//! A link is `<version>:<payload>`, where the payload is gzipped and then
//! base64-encoded. Unpacking it is left to the caller, as a browser does that
//! natively; this reads the payload once unpacked.

use std::fmt;

use esf_data::InfoName;
use esf_dogma_engine::{Fit, Slot, State};

use crate::eft;
use crate::flags::{Place, place_of_flag};
use crate::listed::{ByName, Listed, fit, to_fit_items};

/// Why a link could not be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// No version of the link was ever written like this.
    UnknownVersion(String),
    /// A `killmail:` link, which holds no fit; fetch the killmail from ESI
    /// and load it with [`crate::killmail::load_killmail`].
    Killmail,
    /// A field that has to be a number is not.
    InvalidNumber(String),
    /// An `eft:` link that is not a valid EFT.
    Eft(eft::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownVersion(version) => write!(f, "unknown link version {version}"),
            Error::Killmail => write!(f, "a killmail link holds no fit"),
            Error::InvalidNumber(field) => write!(f, "{field} is not a number"),
            Error::Eft(error) => write!(f, "invalid EFT in link: {error}"),
        }
    }
}

impl std::error::Error for Error {}

/// Load a fit from the unpacked payload of an EVEShip.fit link, of any
/// version ever written. The fit has no skills.
///
/// - `v1` and `v2` are a line `<ship>,<name>,<description>`, then per item
///   `<flag>,<type>,<quantity>`. `v2` adds `,<charge>,<state>` to that.
/// - `v3` is a line `ship,<ship>,<name>,<description>`, then per item one of
///   `module,<rack>,<position from 1>,<type>,<state>,<charge>`,
///   `drone,<type>,<active>,<passive>` or `cargo,<type>,<quantity>`.
/// - `eft` is an EFT, with a type id instead of a name for a type the site
///   that wrote it did not know.
pub fn load_link(info: &impl InfoName, version: &str, payload: &str) -> Result<Fit, Error> {
    match version {
        "v1" | "v2" => load_v1_or_v2(info, payload),
        "v3" => load_v3(info, payload),
        "eft" => eft::load_eft(info, &with_type_names(info, payload)).map_err(Error::Eft),
        "killmail" => Err(Error::Killmail),
        _ => Err(Error::UnknownVersion(version.to_string())),
    }
}

fn parse(csv: &str) -> (Vec<&str>, impl Iterator<Item = Vec<&str>>) {
    let mut lines = csv.trim().lines().map(|line| line.split(',').collect());
    (lines.next().unwrap_or_default(), lines)
}

fn number<T: std::str::FromStr>(field: Option<&&str>) -> Result<T, Error> {
    let field = field.copied().unwrap_or_default();
    field
        .parse()
        .map_err(|_| Error::InvalidNumber(field.to_string()))
}

fn state(field: Option<&&str>) -> Option<State> {
    match field.copied()? {
        "Passive" | "Offline" => Some(State::Offline),
        "Online" => Some(State::Online),
        "Active" => Some(State::Active),
        "Overload" => Some(State::Overload),
        _ => None,
    }
}

fn charge(field: Option<&&str>) -> Result<Option<i32>, Error> {
    match field {
        Some(field) if !field.is_empty() => Ok(Some(number(Some(field))?)),
        _ => Ok(None),
    }
}

fn name(field: Option<&&str>) -> Option<String> {
    field
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
}

fn load_v1_or_v2(info: &impl InfoName, csv: &str) -> Result<Fit, Error> {
    let (header, lines) = parse(csv);

    let mut listed = Vec::new();
    for fields in lines {
        let Some(place) = place_of_flag(number(fields.first())?) else {
            continue;
        };
        listed.push(Listed {
            charge: charge(fields.get(3))?,
            state: state(fields.get(4)),
            ..Listed::new(place, number(fields.get(1))?, number(fields.get(2))?)
        });
    }

    Ok(fit(
        name(header.get(1)),
        number(header.first())?,
        to_fit_items(&ByName(info), listed),
    ))
}

fn load_v3(info: &impl InfoName, csv: &str) -> Result<Fit, Error> {
    let (header, lines) = parse(csv);

    let mut listed = Vec::new();
    for fields in lines {
        match fields[0] {
            "module" => {
                let rack: fn(u8) -> Slot = match fields.get(1).copied() {
                    Some("High") => Slot::High,
                    Some("Medium") => Slot::Medium,
                    Some("Low") => Slot::Low,
                    Some("Rig") => Slot::Rig,
                    Some("SubSystem") => Slot::Subsystem,
                    _ => continue,
                };
                let position: u8 = number(fields.get(2))?;
                let index = position
                    .checked_sub(1)
                    .ok_or_else(|| Error::InvalidNumber(position.to_string()))?;

                listed.push(Listed {
                    state: state(fields.get(4)),
                    charge: charge(fields.get(5))?,
                    ..Listed::new(Place::Slot(rack(index)), number(fields.get(3))?, 1)
                });
            }
            "drone" => {
                let type_id = number(fields.get(1))?;
                let active = number(fields.get(2))?;
                let passive = number(fields.get(3))?;

                for (quantity, state) in [(active, State::Active), (passive, State::Offline)] {
                    if quantity > 0 {
                        listed.push(Listed {
                            state: Some(state),
                            ..Listed::new(Place::Slot(Slot::DroneBay), type_id, quantity)
                        });
                    }
                }
            }
            "cargo" => listed.push(Listed::new(
                Place::Slot(Slot::Cargo),
                number(fields.get(1))?,
                number(fields.get(2))?,
            )),
            _ => {}
        }
    }

    Ok(fit(
        name(header.get(2)),
        number(header.get(1))?,
        to_fit_items(&ByName(info), listed),
    ))
}

/* "<type id>" or "<type id> x<quantity>" becomes its name, where the SDE
 * knows it. */
fn with_type_names(info: &impl InfoName, eft: &str) -> String {
    eft.lines()
        .map(|line| {
            let (id, quantity) = match line.split_once(" x") {
                Some((id, quantity)) if is_number(quantity) => (id, Some(quantity)),
                _ => (line, None),
            };
            let name = is_number(id)
                .then(|| info.get_type(id.parse().ok()?).map(|r#type| r#type.name()))
                .flatten();

            match (name, quantity) {
                (Some(name), Some(quantity)) => format!("{name} x{quantity}"),
                (Some(name), None) => name.to_string(),
                (None, _) => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_number(field: &str) -> bool {
    !field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit())
}
