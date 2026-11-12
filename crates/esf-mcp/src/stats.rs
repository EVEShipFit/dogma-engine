//! The stats of a fit, by name.

use std::collections::BTreeMap;

use esf_dogma_engine::{Calculation, Fit, ItemResult, Slot};
use serde_json::{Map, Value, json};

use crate::fits::{label, slot_label};
use crate::game::{Game, round};

const EFFECT_TURRET_FITTED: i32 = 42;
const EFFECT_LAUNCHER_FITTED: i32 = 40;

struct Reader<'a> {
    game: &'a Game,
    result: &'a ItemResult,
}

impl Reader<'_> {
    fn raw(&self, name: &str) -> Option<f64> {
        let attribute_id = self.game.attribute_id(name)?;
        self.result
            .attributes
            .get(&attribute_id)
            .map(|attribute| attribute.value)
    }

    fn get(&self, name: &str) -> Value {
        self.raw(name)
            .map_or(Value::Null, |value| json!(round(value)))
    }

    fn seconds(&self, name: &str) -> Value {
        self.raw(name)
            .map_or(Value::Null, |value| json!(round(value / 1000.0)))
    }

    fn resist(&self, name: &str) -> Value {
        self.raw(name)
            .map_or(Value::Null, |value| json!(round((1.0 - value) * 100.0)))
    }

    fn used(&self, used: &str, total: &str) -> Value {
        json!({"used": round(self.raw(used).unwrap_or_default()), "total": self.get(total)})
    }
}

/// Drop nulls, and objects left empty.
fn prune(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, prune(value)))
                .filter(|(_, value)| match value {
                    Value::Null => false,
                    Value::Object(map) => !map.is_empty(),
                    _ => true,
                })
                .collect(),
        ),
        value => value,
    }
}

impl Game {
    pub(crate) fn stats(&self, fit: &Fit, calculation: &Calculation) -> Value {
        let ship = Reader {
            game: self,
            result: &calculation.ship,
        };

        let depletes = ship.raw("capacitorDepletesIn");
        let stable = depletes.map(|seconds| seconds < 0.0);
        let capacitor = json!({
            "capacity_gj": ship.get("capacitorCapacity"),
            "recharge_time_s": ship.seconds("rechargeRate"),
            "stable": stable,
            "stable_at_percent": match stable {
                Some(true) => ship.get("capacitorStablePercentage"),
                _ => Value::Null,
            },
            "lasts_s": match (stable, depletes) {
                (Some(false), Some(seconds)) => json!(round(seconds)),
                _ => Value::Null,
            },
            "peak_delta_gj_s": ship.get("capacitorPeakDelta"),
        });

        let mut racks: BTreeMap<&str, u32> = BTreeMap::new();
        let mut turrets = 0;
        let mut launchers = 0;
        for item in &fit.items {
            let rack = match item.slot {
                Slot::High(_) => "high",
                Slot::Medium(_) => "medium",
                Slot::Low(_) => "low",
                Slot::Rig(_) => "rig",
                Slot::Subsystem(_) => "subsystem",
                Slot::Service(_) => "service",
                _ => continue,
            };
            *racks.entry(rack).or_default() += 1;
            turrets += u32::from(self.has_effect(item.type_id, EFFECT_TURRET_FITTED));
            launchers += u32::from(self.has_effect(item.type_id, EFFECT_LAUNCHER_FITTED));
        }
        let slots = |used: u32, total: Option<f64>| match (used, total) {
            (0, None | Some(0.0)) => Value::Null,
            (used, total) => json!({"used": used, "total": total.map(round)}),
        };
        let rack = |rack: &str, total: &str| {
            slots(
                racks.get(rack).copied().unwrap_or_default(),
                ship.raw(total),
            )
        };

        let warp_speed = ship
            .raw("baseWarpSpeed")
            .zip(ship.raw("warpSpeedMultiplier"))
            .map_or(Value::Null, |(base, multiplier)| {
                json!(round(base * multiplier))
            });

        prune(json!({
            "offense": {
                "dps": ship.get("damagePerSecondWithoutReload"),
                "dps_with_reload": ship.get("damagePerSecondWithReload"),
                "volley": ship.get("damageAlpha"),
                "drone_dps": ship.get("droneDamagePerSecond"),
                "fighter_dps": ship.get("fighterDamagePerSecond"),
            },
            "defense": {
                "ehp": ship.get("ehp"),
                "ehp_per_layer": {
                    "shield": ship.get("shieldEhp"),
                    "armor": ship.get("armorEhp"),
                    "hull": ship.get("hullEhp"),
                },
                "hp": {
                    "shield": ship.get("shieldCapacity"),
                    "armor": ship.get("armorHP"),
                    "hull": ship.get("hp"),
                },
                "resists_percent": {
                    "shield": {
                        "em": ship.resist("shieldEmDamageResonance"),
                        "thermal": ship.resist("shieldThermalDamageResonance"),
                        "kinetic": ship.resist("shieldKineticDamageResonance"),
                        "explosive": ship.resist("shieldExplosiveDamageResonance"),
                    },
                    "armor": {
                        "em": ship.resist("armorEmDamageResonance"),
                        "thermal": ship.resist("armorThermalDamageResonance"),
                        "kinetic": ship.resist("armorKineticDamageResonance"),
                        "explosive": ship.resist("armorExplosiveDamageResonance"),
                    },
                    "hull": {
                        "em": ship.resist("emDamageResonance"),
                        "thermal": ship.resist("thermalDamageResonance"),
                        "kinetic": ship.resist("kineticDamageResonance"),
                        "explosive": ship.resist("explosiveDamageResonance"),
                    },
                },
                "effective_repair_per_second": {
                    "shield_passive": ship.get("passiveShieldEffectiveRechargeRate"),
                    "shield": ship.get("shieldEffectiveBoostRate"),
                    "armor": ship.get("armorEffectiveRepairRate"),
                    "hull": ship.get("hullEffectiveRepairRate"),
                },
            },
            "capacitor": capacitor,
            "navigation": {
                "max_velocity_m_s": ship.get("maxVelocity"),
                "align_time_s": ship.get("alignTime"),
                "warp_speed_au_s": warp_speed,
                "mass_kg": ship.get("mass"),
                "agility": ship.get("agility"),
                "signature_radius_m": ship.get("signatureRadius"),
            },
            "targeting": {
                "max_target_range_m": ship.get("maxTargetRange"),
                "scan_resolution_mm": ship.get("scanResolution"),
                "max_locked_targets": ship.get("maxTargets"),
                "sensor_strength": ship.get("scanStrength"),
            },
            "fitting": {
                "cpu_tf": ship.used("cpuLoad", "cpuOutput"),
                "powergrid_mw": ship.used("powerLoad", "powerOutput"),
                "calibration": ship.used("upgradeLoad", "upgradeCapacity"),
                "drone_bandwidth_mbit_s": match ship.raw("droneBandwidth") {
                    None | Some(0.0) => Value::Null,
                    _ => ship.used("droneBandwidthLoad", "droneBandwidth"),
                },
                "drone_bay_m3": match ship.raw("droneCapacity") {
                    None | Some(0.0) => Value::Null,
                    _ => ship.used("droneCapacityLoad", "droneCapacity"),
                },
                "cargo_m3": ship.get("capacity"),
                "slots": {
                    "high": rack("high", "hiSlots"),
                    "medium": rack("medium", "medSlots"),
                    "low": rack("low", "lowSlots"),
                    "rig": rack("rig", "rigSlots"),
                    "subsystem": slots(racks.get("subsystem").copied().unwrap_or_default(), None),
                    "service": rack("service", "serviceSlots"),
                    "turret_hardpoints": slots(turrets, ship.raw("turretSlotsLeft")),
                    "launcher_hardpoints": slots(launchers, ship.raw("launcherSlotsLeft")),
                },
            },
        }))
    }

    /// What the fit holds, one line per item.
    pub(crate) fn items(&self, fit: &Fit, calculation: &Calculation) -> Vec<Value> {
        fit.items
            .iter()
            .zip(&calculation.items)
            .map(|(item, result)| {
                let mut entry = Map::new();
                entry.insert("slot".into(), json!(slot_label(item.slot)));
                entry.insert("name".into(), json!(self.type_name(item.type_id)));
                if item.quantity != 1 {
                    entry.insert("quantity".into(), json!(item.quantity));
                }
                entry.insert("state".into(), json!(label(result.state)));
                if result.state < item.state {
                    entry.insert("requested_state".into(), json!(label(item.state)));
                }
                if let Some(charge) = &item.charge {
                    entry.insert("charge".into(), json!(self.type_name(charge.type_id)));
                }
                Value::Object(entry)
            })
            .collect()
    }

    pub(crate) fn violations(&self, fit: &Fit, calculation: &Calculation) -> Vec<String> {
        calculation
            .violations
            .iter()
            .flatten()
            .map(|violation| self.violation(fit, violation))
            .collect()
    }
}

/// Every number in a summary, by its path like `offense.dps`.
pub fn flatten(value: &Value) -> BTreeMap<String, Value> {
    fn walk(prefix: &str, value: &Value, out: &mut BTreeMap<String, Value>) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    let path = match prefix.is_empty() {
                        true => key.clone(),
                        false => format!("{prefix}.{key}"),
                    };
                    walk(&path, value, out);
                }
            }
            value => {
                out.insert(prefix.to_string(), value.clone());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk("", value, &mut out);
    out
}

/// The stats that differ between two summaries.
pub fn differences(before: &Value, after: &Value) -> Vec<Value> {
    let before = flatten(before);
    let after = flatten(after);
    let mut paths: Vec<&String> = before.keys().chain(after.keys()).collect();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter_map(|path| {
            let left = before.get(path).cloned().unwrap_or(Value::Null);
            let right = after.get(path).cloned().unwrap_or(Value::Null);
            if left == right {
                return None;
            }
            let mut entry = json!({"stat": path, "a": left, "b": right});
            if let (Some(left), Some(right)) = (left.as_f64(), right.as_f64()) {
                entry["change"] = json!(round(right - left));
                if left != 0.0 {
                    entry["change_percent"] = json!(round((right - left) / left.abs() * 100.0));
                }
            }
            Some(entry)
        })
        .collect()
}
