use std::collections::HashMap;

use esf_dogma_engine::calculate::{Calculation, ItemResult};
use esf_dogma_engine::fit::{Fit, Slot, State};
use esf_dogma_engine::info::Info;

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

    blocks.extend(dump_items(info, fit, calculation));
    blocks.join("\n")
}

/// Adjacent identical items in the same rack collapse into one block, like `high_1-4`.
struct Run {
    slot: &'static str,
    first: u32,
    last: u32,
    lines: Vec<(String, String)>,
}

/// Unindexed slots number every item in a stack, so five drones are `dronebay_1-5`.
fn slot_name(slot: Slot) -> (&'static str, Option<u8>) {
    match slot {
        Slot::High(index) => ("high", Some(index)),
        Slot::Medium(index) => ("medium", Some(index)),
        Slot::Low(index) => ("low", Some(index)),
        Slot::Rig(index) => ("rig", Some(index)),
        Slot::Subsystem(index) => ("subsystem", Some(index)),
        Slot::Service(index) => ("service", Some(index)),
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
            Some(index) => (index as u32 + 1, index as u32 + 1),
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
