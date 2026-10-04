//! Canonical form, and writing a fit back to text.

use std::collections::BTreeMap;

use esf_data::{InfoEsf, fold_case};

use super::lookup::{Kind, Lookup};
use super::model::{Entry, Error, EsfFit, Location, State};
use super::names::{number, quote, quote_type, shortest_unique};
use super::resolve::{Fit, Item, Place, default_place};

/// How a line names its mutaplasmid and tactical mode.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Naming {
    /// The fewest words that still match.
    Shortest,
    /// The full name.
    Full,
}

fn checked_name(text: &str, what: &str) -> Result<String, Error> {
    if text.contains(['\r', '\n']) {
        return Err(Error::new(format!("{what} contains CR or LF")));
    }
    Ok(quote(text))
}

fn type_name<I: InfoEsf>(lookup: &Lookup<I>, type_id: i32, what: &str) -> Result<String, Error> {
    match lookup.r#type(type_id) {
        Some(r#type) => Ok(quote_type(r#type.name())),
        None => Err(Error::new(format!("unknown {what} type ID {type_id}"))),
    }
}

fn mutaplasmid_name<I: InfoEsf>(lookup: &Lookup<I>, mutaplasmid: i32, base: Option<i32>) -> String {
    let own = lookup.name(mutaplasmid);
    let others: Vec<&str> = base
        .map(|base| lookup.mutaplasmids(base))
        .unwrap_or_default()
        .iter()
        .copied()
        .filter(|other| *other != mutaplasmid)
        .map(|other| lookup.name(other))
        .collect();
    shortest_unique(own, &others)
}

fn mode_text<I: InfoEsf>(lookup: &Lookup<I>, hull: i32, mode: i32) -> String {
    let own = lookup.mode_name(hull, mode);
    let others: Vec<&str> = lookup
        .modes(hull)
        .into_iter()
        .filter(|other| *other != mode)
        .map(|other| lookup.mode_name(hull, other))
        .collect();
    shortest_unique(own, &others)
}

/// A line without its count.
fn body<I: InfoEsf>(lookup: &Lookup<I>, entry: &Entry, naming: Naming) -> Result<String, Error> {
    let mut parts = vec![match entry.type_id {
        Some(type_id) => type_name(lookup, type_id, "item")?,
        None => "-".to_string(),
    }];
    if let Some(fit_name) = &entry.fit_name {
        parts.push(checked_name(fit_name, "a fit name")?);
    }
    if entry.charge_count.is_some() && entry.charge.is_none() {
        return Err(Error::new("a charge count is set without a charge"));
    }
    if let Some(charge) = entry.charge {
        let count = entry
            .charge_count
            .map(|count| format!("{count}x "))
            .unwrap_or_default();
        parts.push(format!(":{count}{}", type_name(lookup, charge, "charge")?));
    }
    if let Some(mutaplasmid) = entry.mutaplasmid {
        let name = match naming {
            Naming::Shortest => mutaplasmid_name(lookup, mutaplasmid, entry.type_id),
            Naming::Full => type_name(lookup, mutaplasmid, "mutaplasmid")?,
        };
        parts.push(format!("+{name}"));
    }
    if !entry.overrides.is_empty() {
        let pairs = entry
            .overrides
            .iter()
            .map(|(attribute_id, value)| {
                let name = lookup
                    .attribute_name(*attribute_id)
                    .ok_or_else(|| Error::new(format!("unknown attribute ID {attribute_id}")))?;
                if !value.is_finite() {
                    return Err(Error::new("an override value is not finite"));
                }
                Ok(format!("{name} {}", number(*value)))
            })
            .collect::<Result<Vec<_>, _>>()?;
        parts.push(format!("{{{}}}", pairs.join(", ")));
    }
    if let Some(state) = entry.state {
        parts.push(format!("!{}", state.name()));
    }
    if let Some(location) = entry.location {
        parts.push(format!("@{}", location.name()));
    }
    Ok(parts.join(" "))
}

fn line_text<I: InfoEsf>(
    lookup: &Lookup<I>,
    entry: &Entry,
    naming: Naming,
) -> Result<String, Error> {
    let body = body(lookup, entry, naming)?;
    Ok(match entry.count {
        Some(count) => format!("{count}x {body}"),
        None => body,
    })
}

fn hull_text<I: InfoEsf>(
    lookup: &Lookup<I>,
    fit: &EsfFit,
    naming: Naming,
) -> Result<String, Error> {
    let mut parts = vec![match fit.hull {
        Some(hull) => type_name(lookup, hull, "hull")?,
        None => "-".to_string(),
    }];
    if let Some(name) = &fit.name {
        parts.push(checked_name(name, "a fit name")?);
    }
    if let Some(mode) = fit.mode {
        let text = match (naming, fit.hull) {
            (Naming::Shortest, Some(hull)) => mode_text(lookup, hull, mode),
            _ => type_name(lookup, mode, "tactical mode")?,
        };
        parts.push(format!("/{text}"));
    }
    Ok(parts.join(" "))
}

/// The text of fits as they are, with every name in full, to read back in.
pub(super) fn source<I: InfoEsf>(lookup: &Lookup<I>, fits: &[EsfFit]) -> Result<String, Error> {
    let mut out = String::new();
    for fit in fits {
        out.push_str("%esf/1\n");
        out.push_str(&hull_text(lookup, fit, Naming::Full)?);
        out.push('\n');
        for entry in &fit.entries {
            out.push_str(&line_text(lookup, entry, Naming::Full)?);
            out.push('\n');
        }
    }
    Ok(out)
}

/// The group a canonical line is written in.
fn group<I: InfoEsf>(lookup: &Lookup<I>, entry: &Entry, cargo_only: bool) -> Place {
    match (entry.type_id, entry.location) {
        (None, location) => Place::At(location.unwrap_or(Location::Cargo)),
        (Some(type_id), Some(Location::Bay)) => default_place(lookup.classify(type_id), false),
        (Some(_), Some(location)) => Place::At(location),
        (Some(type_id), None) => default_place(lookup.classify(type_id), cargo_only),
    }
}

/// Canonical fits as text.
pub(super) fn render<I: InfoEsf>(lookup: &Lookup<I>, fits: &[EsfFit]) -> Result<String, Error> {
    let mut texts = Vec::new();
    for fit in fits {
        let cargo_only = fit.hull.is_none_or(|hull| lookup.is_container(hull));
        let mut out = format!("%esf/1\n{}\n", hull_text(lookup, fit, Naming::Shortest)?);
        let mut previous = None;
        for entry in &fit.entries {
            let group = group(lookup, entry, cargo_only);
            if previous != Some(group) {
                out.push('\n');
                previous = Some(group);
            }
            out.push_str(&line_text(lookup, entry, Naming::Shortest)?);
            out.push('\n');
        }
        texts.push(out);
    }
    Ok(texts.join("\n"))
}

/// The state that canonical form leaves out; `None` is running.
fn default_state<I: InfoEsf>(lookup: &Lookup<I>, item: &Item) -> Option<State> {
    let active = item
        .type_id
        .is_some_and(|type_id| lookup.is_active(type_id));
    match item.place.is_plugged() || !active {
        true => Some(State::On),
        false => None,
    }
}

fn line<I: InfoEsf>(lookup: &Lookup<I>, fits: &[Fit], fit: &Fit, item: &Item) -> Entry {
    let line = item.line;
    let mut entry = Entry {
        type_id: item.type_id,
        ..Entry::default()
    };

    if let Some(reference) = item.reference {
        entry.fit_name = fits[reference].block.fit_name.clone();
    }

    if let (Some(charge), Some(type_id)) = (item.charge, item.type_id) {
        entry.charge = Some(charge);
        if line.charge_count != lookup.full_load(type_id, charge) {
            entry.charge_count = line.charge_count;
        }
    }

    let mut overrides: BTreeMap<i32, f64> = item.overrides.iter().copied().collect();
    if let (Some(mutaplasmid), Some(type_id)) = (item.mutaplasmid, item.type_id) {
        entry.mutaplasmid = Some(mutaplasmid);
        for attribute_id in lookup.rollable(mutaplasmid) {
            overrides
                .entry(attribute_id)
                .or_insert_with(|| lookup.base_value(type_id, attribute_id));
        }
    }
    entry.overrides = overrides
        .into_iter()
        .map(|(attribute_id, value)| (attribute_id, if value == 0.0 { 0.0 } else { value }))
        .collect();
    entry.overrides.sort_by_cached_key(|(attribute_id, _)| {
        let name = lookup.attribute_name(*attribute_id).unwrap_or_default();
        (fold_case(name), name)
    });

    if line.state.is_some() && line.state != default_state(lookup, item) {
        entry.state = line.state;
    }

    if let Place::At(location) = item.place
        && !location.is_rack()
        && item.place != default_place(item.kind, fit.cargo_only)
    {
        entry.location = Some(location);
    }
    entry
}

fn same_line(left: &Entry, right: &Entry) -> bool {
    Entry {
        count: None,
        ..left.clone()
    } == Entry {
        count: None,
        ..right.clone()
    }
}

/// Merge adjacent identical lines that repeat into one count.
fn collapse(lines: Vec<(bool, Entry)>) -> Vec<Entry> {
    let mut out: Vec<(bool, Entry)> = Vec::new();
    for (repeatable, entry) in lines {
        if let Some((true, previous)) = out.last_mut()
            && repeatable
            && same_line(previous, &entry)
        {
            previous.count = Some(previous.count.unwrap_or(1) + entry.count.unwrap_or(1));
            continue;
        }
        out.push((repeatable, entry));
    }
    out.into_iter()
        .map(|(_, mut entry)| {
            entry.count = entry.count.filter(|count| *count != 1);
            entry
        })
        .collect()
}

fn rack<I: InfoEsf>(lookup: &Lookup<I>, fits: &[Fit], fit: &Fit, rack: Location) -> Vec<Entry> {
    let mut runs = fit.racks.get(&rack).cloned().unwrap_or_default();
    while runs
        .last()
        .is_some_and(|(_, _, index)| index.is_none_or(|index| fit.items[index].type_id.is_none()))
    {
        runs.pop();
    }

    let lines = runs
        .into_iter()
        .map(|(_, length, index)| {
            let mut entry = match index.map(|index| &fit.items[index]) {
                Some(item) if item.type_id.is_some() => line(lookup, fits, fit, item),
                _ => Entry {
                    location: Some(rack),
                    ..Entry::default()
                },
            };
            entry.count = Some(length);
            (true, entry)
        })
        .collect();
    collapse(lines)
}

fn sorted<I: InfoEsf>(lookup: &Lookup<I>, fits: &[Fit], fit: &Fit, items: &[&Item]) -> Vec<Entry> {
    let mut lines: Vec<((String, String), bool, Entry)> = items
        .iter()
        .map(|item| {
            let mut entry = line(lookup, fits, fit, item);
            let repeatable = !item.place.is_stored();
            let count = item.line.count.unwrap_or(1);
            let text = if repeatable {
                let text = body(lookup, &entry, Naming::Shortest).unwrap_or_default();
                entry.count = Some(count);
                text
            } else {
                entry.count = Some(count).filter(|count| *count != 1);
                line_text(lookup, &entry, Naming::Shortest).unwrap_or_default()
            };
            let name = fold_case(lookup.name(item.type_id.unwrap_or_default()));
            ((name, text), repeatable, entry)
        })
        .collect();
    lines.sort_by(|left, right| left.0.cmp(&right.0));
    collapse(
        lines
            .into_iter()
            .map(|(_, repeatable, entry)| (repeatable, entry))
            .collect(),
    )
}

fn fighters<I: InfoEsf>(
    lookup: &Lookup<I>,
    fits: &[Fit],
    fit: &Fit,
    items: &[&Item],
) -> Vec<Entry> {
    let (tubes, bay): (Vec<&Item>, Vec<&Item>) =
        items.iter().partition(|item| item.place == Place::Fighters);
    let mut lines: Vec<Entry> = tubes
        .into_iter()
        .map(|item| {
            let mut entry = line(lookup, fits, fit, item);
            let squadron = item
                .type_id
                .and_then(|type_id| lookup.squadron_size(type_id));
            entry.count = item.line.count.filter(|count| Some(*count) != squadron);
            entry
        })
        .collect();
    lines.extend(sorted(lookup, fits, fit, &bay));
    lines
}

fn groups() -> Vec<Place> {
    let racks = [
        Location::Sub,
        Location::High,
        Location::Mid,
        Location::Low,
        Location::Rig,
        Location::Svc,
    ];
    let holds = Location::ALL
        .iter()
        .copied()
        .filter(|location| location.number() >= Location::Ammo.number());
    racks
        .into_iter()
        .map(Place::At)
        .chain([Place::Drones, Place::Fighters, Place::At(Location::Cargo)])
        .chain(holds.map(Place::At))
        .chain([Place::Implants, Place::Boosters])
        .collect()
}

pub(super) fn canonical_fit<I: InfoEsf>(lookup: &Lookup<I>, fits: &[Fit], fit: &Fit) -> EsfFit {
    let mut grouped: BTreeMap<Place, Vec<&Item>> = BTreeMap::new();
    for item in fit.items.iter().filter(|item| !item.place.is_rack()) {
        let group = match item.place {
            Place::At(Location::Bay) if item.kind == Kind::Fighter => Place::Fighters,
            Place::At(Location::Bay) => Place::Drones,
            place => place,
        };
        grouped.entry(group).or_default().push(item);
    }

    let mut entries = Vec::new();
    for group in groups() {
        let items = grouped.remove(&group).unwrap_or_default();
        entries.extend(match group {
            Place::At(location) if location.is_rack() => rack(lookup, fits, fit, location),
            Place::Fighters => fighters(lookup, fits, fit, &items),
            _ => sorted(lookup, fits, fit, &items),
        });
    }

    EsfFit {
        hull: fit.hull,
        name: fit.block.fit_name.clone(),
        mode: fit.mode,
        entries,
    }
}
