use esf_dogma_engine::calculate::{Calculation, ItemResult};
use esf_dogma_engine::fit::{Fit, Slot, State};
use esf_dogma_engine::info::Info;

/// What the in-game fitting window shows, in its units.
pub fn dump(info: &impl Info, calculation: &Calculation) -> String {
    let hull = |name| attribute(info, &calculation.ship, name);
    let resist = |name| (1.0 - hull(name)) * 100.0;

    let statistics = serde_json::json!({
        "capacitor": {
            "stable": hull("capacitorDepletesIn") == -1.0,
            "depletes_in": hull("capacitorDepletesIn"),
            "capacity": hull("capacitorCapacity").floor(),
            "recharge": hull("rechargeRate") / 1000.0,
            "peak": hull("capacitorPeakDelta"),
            "percentage": hull("capacitorPeakDeltaPercentage"),
        },
        "offense": {
            "dps": hull("damagePerSecondWithoutReload"),
            "dps_with_reload": hull("damagePerSecondWithReload"),
            "alpha": hull("damageAlpha"),
            "drone_dps": hull("droneDamagePerSecond"),
        },
        "defense": {
            "recharge": {
                "passive": hull("passiveShieldRechargeRate"),
                "shield": hull("shieldBoostRate"),
                "armor": hull("armorRepairRate"),
                "hull": hull("hullRepairRate"),
            },
            "shield": {
                "resist": {
                    "em": resist("shieldEmDamageResonance"),
                    "therm": resist("shieldThermalDamageResonance"),
                    "kin": resist("shieldKineticDamageResonance"),
                    "expl": resist("shieldExplosiveDamageResonance"),
                },
                "hp": hull("shieldCapacity"),
                "recharge": hull("shieldRechargeRate") / 1000.0,
            },
            "armor": {
                "resist": {
                    "em": resist("armorEmDamageResonance"),
                    "therm": resist("armorThermalDamageResonance"),
                    "kin": resist("armorKineticDamageResonance"),
                    "expl": resist("armorExplosiveDamageResonance"),
                },
                "hp": hull("armorHP"),
            },
            "structure": {
                "resist": {
                    "em": resist("emDamageResonance"),
                    "therm": resist("thermalDamageResonance"),
                    "kin": resist("kineticDamageResonance"),
                    "expl": resist("explosiveDamageResonance"),
                },
                "hp": hull("hp"),
            },
            "ehp": hull("ehp"),
        },
        "targeting": {
            "lock_range": hull("maxTargetRange") / 1000.0,
            "sensor_strength": hull("scanStrength"),
            "scan_resolution": hull("scanResolution"),
            "signature_radius": hull("signatureRadius"),
            "max_locked_targets": hull("maxLockedTargets"),
        },
        "navigation": {
            "speed": hull("maxVelocity"),
            "mass": hull("mass") / 1000.0,
            "agility": hull("agility"),
            "warp_speed": hull("warpSpeedMultiplier"),
            "align_time": hull("alignTime"),
        },
        "drones": {
            "dps": hull("droneDamagePerSecond"),
            "bandwidth_load": hull("droneBandwidthLoad"),
            "bandwidth": hull("droneBandwidth"),
            "range": attribute(info, &calculation.character, "droneControlDistance") / 1000.0,
            "active": hull("droneActive"),
            "capacity_load": hull("droneCapacityLoad"),
            "capacity": hull("droneCapacity"),
        },
        "cpu": {
            "free": hull("cpuFree"),
            "capacity": hull("cpuOutput"),
        },
        "power": {
            "free": hull("powerFree"),
            "capacity": hull("powerOutput"),
        },
    });

    let mut flattened = Vec::new();
    flatten(&statistics, "", &mut flattened);

    align(&flattened)
}

/* An attribute the item lacks has its SDE default. */
fn attribute(info: &impl Info, item: &ItemResult, name: &str) -> f64 {
    let attribute_id = info.attribute_name_to_id(name);
    item.attributes.get(&attribute_id).map_or_else(
        || {
            info.get_dogma_attribute(attribute_id)
                .map_or(0.0, |attribute| attribute.default_value() as f64)
        },
        |attribute| attribute.value,
    )
}

/// Adjacent identical items in the same rack collapse into one block, like `high_1-4`.
struct Run {
    slot: &'static str,
    first: u32,
    last: u32,
    statistics: Vec<(String, String)>,
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

pub fn dump_items(info: &impl Info, fit: &Fit, calculation: &Calculation) -> String {
    let mut runs: Vec<Run> = Vec::new();
    let mut stacked = std::collections::HashMap::new();

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

        let mut statistics = Vec::new();
        push_item(
            info,
            item.type_id,
            Some(result.state),
            result,
            "",
            &mut statistics,
        );
        if let (Some(charge), Some(charge_result)) = (&item.charge, &result.charge) {
            push_item(
                info,
                charge.type_id,
                None,
                charge_result,
                "/charge",
                &mut statistics,
            );
        }

        match runs.last_mut() {
            Some(run)
                if run.slot == slot && run.last + 1 == first && run.statistics == statistics =>
            {
                run.last = last;
            }
            _ => runs.push(Run {
                slot,
                first,
                last,
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
fn push_item(
    info: &impl Info,
    type_id: i32,
    state: Option<State>,
    result: &ItemResult,
    path: &str,
    statistics: &mut Vec<(String, String)>,
) {
    let name = info
        .get_type(type_id)
        .map_or_else(|| type_id.to_string(), |r#type| r#type.name().to_string());
    statistics.push((format!("{path}/type"), name));

    if let Some(state) = state {
        let state = format!("{:?}", state).to_lowercase();
        statistics.push((format!("{path}/state"), state));
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
