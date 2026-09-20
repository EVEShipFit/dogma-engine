use std::collections::HashMap;

use esf_data::Info;
use esf_dogma_engine::{BuffSource, Calculation, Fit, ItemResult, Projection, Slot, State};

/// Only attributes an effect moved away from their base value; the rest is SDE data.
pub fn dump(info: &impl Info, fit: &Fit, calculation: &Calculation) -> String {
    let mut blocks = Vec::new();

    let mut ship = Vec::new();
    push_item(
        info,
        Some(fit.ship.type_id),
        None,
        &calculation.ship,
        "ship",
        &mut ship,
    );
    blocks.push(align(&ship));

    if let (Some(mode_type_id), Some(mode_result)) = (fit.ship.mode, &calculation.mode) {
        let mut mode = Vec::new();
        push_item(
            info,
            Some(mode_type_id),
            None,
            mode_result,
            "mode",
            &mut mode,
        );
        blocks.push(align(&mode));
    }

    let mut character = Vec::new();
    push_item(
        info,
        None,
        None,
        &calculation.character,
        "character",
        &mut character,
    );
    if !character.is_empty() {
        blocks.push(align(&character));
    }

    let incoming = dump_projection(info, &fit.incoming, "incoming");
    if !incoming.is_empty() {
        blocks.push(align(&incoming));
    }

    let buffs = dump_buffs(info, calculation);
    if !buffs.is_empty() {
        blocks.push(align(&buffs));
    }

    let outgoing = dump_projection(info, &calculation.outgoing, "outgoing");
    if !outgoing.is_empty() {
        blocks.push(align(&outgoing));
    }

    blocks.extend(dump_items(info, fit, calculation));
    blocks.join("\n")
}

/// Numbered from 1, in the order the calculation reports them.
fn dump_buffs(info: &impl Info, calculation: &Calculation) -> Vec<(String, String)> {
    let mut lines = Vec::new();

    for (index, buff) in calculation.buffs.iter().enumerate() {
        let name = format!("buff_{}", index + 1);
        let display_name = info
            .get_dbuff_collection(buff.id)
            .and_then(|collection| collection.display_name())
            .filter(|display_name| !display_name.is_empty())
            .map_or_else(|| buff.id.to_string(), str::to_string);
        let from = match buff.from {
            BuffSource::Beacon { type_id } => info
                .get_type(type_id)
                .map_or_else(|| type_id.to_string(), |r#type| r#type.name().to_string()),
        };

        lines.push((format!("{name}/type"), display_name));
        lines.push((format!("{name}/value"), format!("{:.6}", buff.value + 0.0)));
        lines.push((format!("{name}/from"), from));
        if !buff.applied {
            lines.push((format!("{name}/applied"), "false".to_string()));
        }
    }

    lines
}

/// A projection, numbered from 1 in the order it is held.
fn dump_projection(info: &impl Info, projection: &Projection, what: &str) -> Vec<(String, String)> {
    let mut lines = Vec::new();

    for (index, buff) in projection.buffs.iter().enumerate() {
        let name = format!("{what}_buff_{}", index + 1);
        lines.push((format!("{name}/id"), buff.id.to_string()));
        lines.push((format!("{name}/value"), format!("{:.6}", buff.value + 0.0)));
    }

    for (index, effect) in projection.effects.iter().enumerate() {
        let name = format!("{what}_effect_{}", index + 1);
        lines.push((format!("{name}/type"), type_name(info, effect.type_id)));
        lines.push((
            format!("{name}/effect"),
            info.get_dogma_effect(effect.effect_id).map_or_else(
                || effect.effect_id.to_string(),
                |effect| effect.name().to_string(),
            ),
        ));
        for (attribute_id, value) in &effect.attributes {
            lines.push((
                format!("{name}/{}", attribute_name(info, *attribute_id)),
                format!("{:.6}", value + 0.0),
            ));
        }
    }

    lines
}

fn type_name(info: &impl Info, type_id: i32) -> String {
    info.get_type(type_id)
        .map_or_else(|| type_id.to_string(), |r#type| r#type.name().to_string())
}

fn attribute_name(info: &impl Info, attribute_id: i32) -> String {
    info.get_dogma_attribute(attribute_id).map_or_else(
        || attribute_id.to_string(),
        |attribute| attribute.name().to_string(),
    )
}

/// Adjacent identical items in the same rack collapse into one block, like `high_1-4`.
struct Run {
    slot: &'static str,
    first: u32,
    last: u32,
    lines: Vec<(String, String)>,
}

/// Numbered from 1. Unindexed slots number every item in a stack, so five drones are `dronebay_1-5`.
fn slot_name(slot: Slot) -> (&'static str, Option<u16>) {
    match slot {
        Slot::High(index) => ("high", Some(u16::from(index) + 1)),
        Slot::Medium(index) => ("medium", Some(u16::from(index) + 1)),
        Slot::Low(index) => ("low", Some(u16::from(index) + 1)),
        Slot::Rig(index) => ("rig", Some(u16::from(index) + 1)),
        Slot::Subsystem(index) => ("subsystem", Some(u16::from(index) + 1)),
        Slot::Service(index) => ("service", Some(u16::from(index) + 1)),
        Slot::FighterTube(index) => ("fightertube", Some(u16::from(index) + 1)),
        Slot::FighterBay => ("fighterbay", None),
        Slot::Implant(index) => ("implant", Some(index.into())),
        Slot::Booster(index) => ("booster", Some(index)),
        Slot::DroneBay => ("dronebay", None),
        Slot::Cargo => ("cargo", None),
    }
}

fn dump_items(info: &impl Info, fit: &Fit, calculation: &Calculation) -> Vec<String> {
    let mut runs: Vec<Run> = Vec::new();
    let mut stacked = HashMap::new();

    for (item, result) in fit.items.iter().zip(&calculation.items) {
        let (slot, index) = slot_name(item.slot);
        let (first, last) = match index {
            Some(number) => (number as u32, number as u32),
            None => {
                let count = stacked.entry(slot).or_insert(0);
                *count += item.quantity;
                (*count - item.quantity + 1, *count)
            }
        };

        let mut lines = Vec::new();
        push_item(
            info,
            Some(item.type_id),
            Some(result.state),
            result,
            "",
            &mut lines,
        );
        if let (Some(charge), Some(charge_result)) = (&item.charge, &result.charge) {
            push_item(
                info,
                Some(charge.type_id),
                None,
                charge_result,
                "/charge",
                &mut lines,
            );
        }

        match runs.last_mut() {
            Some(run) if run.slot == slot && run.last + 1 == first && run.lines == lines => {
                run.last = last;
            }
            _ => runs.push(Run {
                slot,
                first,
                last,
                lines,
            }),
        }
    }

    runs.iter()
        .map(|run| {
            let name = match run.first == run.last {
                true => format!("{}_{}", run.slot, run.first),
                false => format!("{}_{}-{}", run.slot, run.first, run.last),
            };
            let lines: Vec<_> = run
                .lines
                .iter()
                .map(|(key, value)| (format!("{name}{key}"), value.clone()))
                .collect();
            align(&lines)
        })
        .collect()
}

fn push_item(
    info: &impl Info,
    type_id: Option<i32>,
    state: Option<State>,
    result: &ItemResult,
    path: &str,
    lines: &mut Vec<(String, String)>,
) {
    if let Some(type_id) = type_id {
        let name = info
            .get_type(type_id)
            .map_or_else(|| type_id.to_string(), |r#type| r#type.name().to_string());
        lines.push((format!("{path}/type"), name));
    }

    if let Some(state) = state {
        let state = format!("{:?}", state).to_lowercase();
        lines.push((format!("{path}/state"), state));
    }

    for (attribute_id, attribute) in &result.attributes {
        if attribute.value == attribute.base {
            continue;
        }

        let name = info.get_dogma_attribute(*attribute_id).map_or_else(
            || attribute_id.to_string(),
            |attribute| attribute.name().to_string(),
        );
        let value = match attribute.value.is_finite() {
            true => format!("{:.6}", attribute.value + 0.0),
            false => "non-finite".to_string(),
        };
        lines.push((format!("{path}/{name}"), value));
    }
}

fn align(lines: &[(String, String)]) -> String {
    let width = lines.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
    lines
        .iter()
        .map(|(key, value)| format!("{key:width$} = {value}\n"))
        .collect()
}
