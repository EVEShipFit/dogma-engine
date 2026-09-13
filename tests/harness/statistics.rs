use esf_dogma_engine::calculate::{
    self,
    item::{Item, SlotType},
};
use esf_dogma_engine::info::Info;
use esf_dogma_engine::rust;

pub fn dump(output: &rust::Output) -> String {
    let mut output = serde_json::to_value(output).unwrap();
    /* Slot state is part of dump_items. */
    output.as_object_mut().unwrap().remove("slots");

    let mut statistics = Vec::new();
    flatten(&output, "", &mut statistics);

    align(&statistics)
}

/// Adjacent identical items in the same rack collapse into one block, like `high_1-4`.
struct Run {
    slot: String,
    first: i32,
    last: i32,
    statistics: Vec<(String, String)>,
}

pub fn dump_items(info: &impl Info, ship: &calculate::Ship) -> String {
    let mut runs: Vec<Run> = Vec::new();
    let mut drones = 0;

    for item in &ship.items {
        let slot = format!("{:?}", item.slot.r#type).to_lowercase();
        let number = item.slot.index.map_or_else(
            || {
                drones += 1;
                drones
            },
            |index| index + 1,
        );

        let mut statistics = Vec::new();
        push_item(info, item, "", &mut statistics);
        if let Some(charge) = &item.charge {
            push_item(info, charge, "/charge", &mut statistics);
        }

        match runs.last_mut() {
            Some(run)
                if run.slot == slot && run.last + 1 == number && run.statistics == statistics =>
            {
                run.last = number;
            }
            _ => runs.push(Run {
                slot,
                first: number,
                last: number,
                statistics,
            }),
        }
    }

    runs.iter()
        .map(|run| {
            let name = match run.first == run.last {
                true => format!("{}_{}", run.slot, run.first),
                false => format!("{}_{}-{}", run.slot, run.first, run.last),
            };
            let statistics: Vec<_> = run
                .statistics
                .iter()
                .map(|(key, value)| (format!("{name}{key}"), value.clone()))
                .collect();
            align(&statistics)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Only attributes an effect moved away from their base value; the rest is SDE data.
fn push_item(info: &impl Info, item: &Item, path: &str, statistics: &mut Vec<(String, String)>) {
    let name = info.get_type(item.type_id).map_or_else(
        || item.type_id.to_string(),
        |r#type| r#type.name().to_string(),
    );
    statistics.push((format!("{path}/type"), name));

    if item.slot.r#type != SlotType::Charge {
        let state = format!("{:?}", item.state).to_lowercase();
        statistics.push((format!("{path}/state"), state));
    }

    for (attribute_id, attribute) in &item.attributes {
        let value = attribute.value.get().unwrap_or(attribute.base_value);
        if value == attribute.base_value {
            continue;
        }

        let name = info.get_dogma_attribute(*attribute_id).map_or_else(
            || attribute_id.to_string(),
            |attribute| attribute.name().to_string(),
        );
        let value = match value.is_finite() {
            true => format!("{:.6}", value + 0.0),
            false => "non-finite".to_string(),
        };
        statistics.push((format!("{path}/{name}"), value));
    }
}

fn align(statistics: &[(String, String)]) -> String {
    let width = statistics
        .iter()
        .map(|(key, _)| key.len())
        .max()
        .unwrap_or(0);
    statistics
        .iter()
        .map(|(key, value)| format!("{key:width$} = {value}\n"))
        .collect()
}

fn flatten(value: &serde_json::Value, path: &str, statistics: &mut Vec<(String, String)>) {
    let value = match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                let path = match path {
                    "" => key.to_string(),
                    path => format!("{path}/{key}"),
                };
                flatten(value, &path, statistics);
            }
            return;
        }
        serde_json::Value::Number(number) => format!("{:.6}", number.as_f64().unwrap() + 0.0),
        serde_json::Value::Null => "non-finite".to_string(),
        serde_json::Value::String(text) => text.clone(),
        value => value.to_string(),
    };

    statistics.push((path.to_string(), value));
}
