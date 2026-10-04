//! Resolving names against the SDE, and the rules that need it.

use std::collections::{BTreeMap, HashSet};

use esf_data::{InfoEsf, fold_case};

use super::lookup::{Kind, Lookup};
use super::model::{Error, Location, State};
use super::names::is_word_prefix;
use super::text::{Block, Line, LineKind};

/// Where an item ends up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Place {
    At(Location),
    Drones,
    Fighters,
    Implants,
    Boosters,
}

impl Place {
    pub fn is_rack(self) -> bool {
        matches!(self, Place::At(location) if location.is_rack())
    }

    pub fn is_stored(self) -> bool {
        matches!(self, Place::At(location) if !location.is_rack())
    }

    pub fn is_plugged(self) -> bool {
        matches!(self, Place::Implants | Place::Boosters)
    }
}

pub(super) fn rack_of(kind: Kind) -> Option<Location> {
    match kind {
        Kind::Sub => Some(Location::Sub),
        Kind::High => Some(Location::High),
        Kind::Mid => Some(Location::Mid),
        Kind::Low => Some(Location::Low),
        Kind::Rig => Some(Location::Rig),
        Kind::Svc => Some(Location::Svc),
        _ => None,
    }
}

pub(super) fn default_place(kind: Kind, cargo_only: bool) -> Place {
    if cargo_only {
        return Place::At(Location::Cargo);
    }
    match kind {
        Kind::Drone => Place::Drones,
        Kind::Fighter => Place::Fighters,
        Kind::Implant => Place::Implants,
        Kind::Booster => Place::Boosters,
        _ => Place::At(rack_of(kind).unwrap_or(Location::Cargo)),
    }
}

pub(super) struct Item<'b> {
    pub line: &'b Line,
    pub type_id: Option<i32>,
    pub kind: Kind,
    pub place: Place,
    pub charge: Option<i32>,
    pub mutaplasmid: Option<i32>,
    pub overrides: Vec<(i32, f64)>,
    pub reference: Option<usize>,
}

/// A run of slots in a rack: the first slot, how many, and what fills them.
pub(super) type Run = (u32, u32, Option<usize>);

pub(super) struct Fit<'b> {
    pub block: &'b Block,
    pub hull: Option<i32>,
    pub mode: Option<i32>,
    pub cargo_only: bool,
    pub items: Vec<Item<'b>>,
    pub racks: BTreeMap<Location, Vec<Run>>,
}

fn pick(candidates: Vec<i32>, given: &str, what: &str, line: usize) -> Result<i32, Error> {
    match candidates[..] {
        [one] => Ok(one),
        [] => Err(Error::at(line, format!("no {what} matches {given:?}"))),
        _ => Err(Error::at(
            line,
            format!("{given:?} matches more than one {what}"),
        )),
    }
}

fn resolve_hull<'b, I: InfoEsf>(block: &'b Block, lookup: &Lookup<I>) -> Result<Fit<'b>, Error> {
    let line = block.hull_line;
    let mut fit = Fit {
        block,
        hull: None,
        mode: None,
        cargo_only: true,
        items: Vec::new(),
        racks: BTreeMap::new(),
    };
    let Some(name) = &block.hull else {
        return Ok(fit);
    };

    let hull = lookup.type_by_name(name, line)?;
    if lookup.classify(hull) != Kind::Hull {
        return Err(Error::at(
            line,
            format!("{name:?} is not a ship, structure or container"),
        ));
    }

    if let Some(given) = &block.mode {
        let modes = lookup.modes(hull);
        if modes.is_empty() {
            return Err(Error::at(line, format!("{name:?} has no tactical modes")));
        }
        let matches = modes
            .into_iter()
            .filter(|mode| {
                is_word_prefix(given, lookup.name(*mode))
                    || is_word_prefix(given, lookup.mode_name(hull, *mode))
            })
            .collect();
        fit.mode = Some(pick(matches, given, "tactical mode", line)?);
    }

    fit.hull = Some(hull);
    fit.cargo_only = lookup.is_container(hull);
    Ok(fit)
}

fn resolve_line<'b, I: InfoEsf>(
    line: &'b Line,
    fit: &Fit,
    fits: &[Fit],
    lookup: &Lookup<I>,
) -> Result<Item<'b>, Error> {
    let number = line.number;
    let error = |message: String| Err(Error::at(number, message));

    if line.kind == LineKind::Empty {
        if fit.cargo_only {
            return error("a fit without a ship has no slots to keep empty".into());
        }
        return Ok(Item {
            line,
            type_id: None,
            kind: Kind::Other,
            place: Place::At(line.location.unwrap()),
            charge: None,
            mutaplasmid: None,
            overrides: Vec::new(),
            reference: None,
        });
    }

    let type_id = lookup.type_by_name(&line.name, number)?;
    let kind = lookup.classify(type_id);
    if kind == Kind::Modifier {
        return error("a tactical mode is only written on the hull line".into());
    }

    let mut item = Item {
        line,
        type_id: Some(type_id),
        kind,
        place: default_place(kind, fit.cargo_only),
        charge: None,
        mutaplasmid: None,
        overrides: Vec::new(),
        reference: None,
    };

    if let Some(fit_name) = &line.fit_name {
        if kind != Kind::Hull {
            return error(format!("{:?} cannot take a fit name", line.name));
        }
        let wanted = fold_case(fit_name);
        let matches: Vec<usize> = fits
            .iter()
            .enumerate()
            .filter(|(_, other)| {
                other.hull == Some(type_id)
                    && other
                        .block
                        .fit_name
                        .as_ref()
                        .is_some_and(|name| fold_case(name) == wanted)
            })
            .map(|(index, _)| index)
            .collect();
        match matches[..] {
            [one] => item.reference = Some(one),
            [] => return error(format!("no fit matches {} {fit_name:?}", line.name)),
            _ => {
                return error(format!(
                    "more than one fit matches {} {fit_name:?}",
                    line.name
                ));
            }
        }
    }

    if let Some(location) = line.location {
        if line.index.is_some() {
            if fit.cargo_only {
                return error("a fit without a ship stores everything in cargo".into());
            }
            if rack_of(kind) != Some(location) {
                return error(format!(
                    "{:?} does not go in the {} rack",
                    line.name,
                    location.name()
                ));
            }
        } else {
            if fit.cargo_only && location != Location::Cargo {
                return error("a fit without a ship stores everything in cargo".into());
            }
            if location == Location::Bay && !matches!(kind, Kind::Drone | Kind::Fighter) {
                return error("@bay is only for drones and fighters".into());
            }
        }
        item.place = Place::At(location);
    }

    if item.place.is_plugged() && line.count.is_some() {
        return error("a plugged-in implant or booster takes no count".into());
    }

    if let Some(state) = line.state {
        if item.place.is_stored() {
            return error("a stored item has no state".into());
        }
        if item.place.is_plugged() && state != State::Off {
            return error("an implant or booster takes !off only".into());
        }
    }

    if let Some(charge) = &line.charge {
        let charge_id = lookup.type_by_name(charge, number)?;
        if !lookup.is_charge(charge_id) {
            return error(format!("{charge:?} is not a charge"));
        }
        item.charge = Some(charge_id);
    }

    if let Some(given) = &line.mutaplasmid {
        let matches = lookup
            .mutaplasmids(type_id)
            .into_iter()
            .filter(|mutaplasmid| is_word_prefix(given, lookup.name(*mutaplasmid)))
            .collect();
        item.mutaplasmid = Some(pick(matches, given, "mutaplasmid", number)?);
    }

    let mut seen = HashSet::new();
    for (name, value) in &line.overrides {
        let Some(attribute_id) = lookup.info.attribute_name_to_id_ignoring_case(name) else {
            return error(format!("unknown attribute {name:?}"));
        };
        if !seen.insert(attribute_id) {
            return error(format!("attribute {name:?} is overridden twice"));
        }
        let number: f64 = value.parse().unwrap_or(f64::INFINITY);
        if !number.is_finite() {
            return error(format!("value {value} is out of range"));
        }
        item.overrides.push((attribute_id, number));
    }

    Ok(item)
}

fn check_cycles(fits: &[Fit]) -> Result<(), Error> {
    fn visit(
        fits: &[Fit],
        fit: usize,
        path: &mut Vec<usize>,
        done: &mut HashSet<usize>,
    ) -> Result<(), Error> {
        if done.contains(&fit) {
            return Ok(());
        }
        path.push(fit);
        for item in &fits[fit].items {
            let Some(reference) = item.reference else {
                continue;
            };
            if path.contains(&reference) {
                return Err(Error::at(item.line.number, "a fit cannot contain itself"));
            }
            visit(fits, reference, path, done)?;
        }
        path.pop();
        done.insert(fit);
        Ok(())
    }

    let mut done = HashSet::new();
    (0..fits.len()).try_for_each(|fit| visit(fits, fit, &mut Vec::new(), &mut done))
}

/// Assign the slots of one rack: pinned items first, then the others in order.
fn layout(items: &[Item], indexes: &[usize]) -> Result<Vec<Run>, Error> {
    let mut pinned: BTreeMap<u32, usize> = BTreeMap::new();
    for &index in indexes {
        let Some(slot) = items[index].line.index else {
            continue;
        };
        if pinned.insert(slot, index).is_some() {
            return Err(Error::at(
                items[index].line.number,
                format!("slot {slot} is pinned twice"),
            ));
        }
    }

    let mut runs: Vec<Run> = pinned
        .iter()
        .map(|(slot, index)| (*slot, 1, Some(*index)))
        .collect();
    let mut position = 1u32;
    for &index in indexes {
        let line = items[index].line;
        if line.index.is_some() {
            continue;
        }
        let mut remaining = line.count.unwrap_or(1);
        while remaining > 0 {
            while pinned.contains_key(&position) {
                position += 1;
            }
            let room = pinned
                .range(position..)
                .next()
                .map_or(remaining, |(pin, _)| pin - position);
            let take = remaining.min(room);
            runs.push((position, take, Some(index)));
            position = position.saturating_add(take);
            remaining -= take;
        }
    }
    runs.sort_by_key(|run| run.0);

    let mut filled = Vec::new();
    let mut position = 1;
    for (start, length, index) in runs {
        if start > position {
            filled.push((position, start - position, None));
        }
        filled.push((start, length, index));
        position = start + length;
    }
    Ok(filled)
}

pub(super) fn resolve<'b, I: InfoEsf>(
    blocks: &'b [Block],
    lookup: &Lookup<I>,
) -> Result<Vec<Fit<'b>>, Error> {
    let mut fits = blocks
        .iter()
        .map(|block| resolve_hull(block, lookup))
        .collect::<Result<Vec<_>, _>>()?;

    for index in 0..fits.len() {
        let items = fits[index]
            .block
            .lines
            .iter()
            .map(|line| resolve_line(line, &fits[index], &fits, lookup))
            .collect::<Result<Vec<_>, _>>()?;

        let mut plugged = HashSet::new();
        for item in items.iter().filter(|item| item.place.is_plugged()) {
            if !plugged.insert(item.type_id) {
                return Err(Error::at(
                    item.line.number,
                    format!("{:?} is plugged in twice", item.line.name),
                ));
            }
        }
        fits[index].items = items;
    }

    check_cycles(&fits)?;

    for fit in &mut fits {
        for rack in Location::ALL
            .iter()
            .copied()
            .filter(|location| location.is_rack())
        {
            let indexes: Vec<usize> = (0..fit.items.len())
                .filter(|index| fit.items[*index].place == Place::At(rack))
                .collect();
            if !indexes.is_empty() {
                fit.racks.insert(rack, layout(&fit.items, &indexes)?);
            }
        }
    }
    Ok(fits)
}
