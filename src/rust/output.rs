use std::collections::BTreeMap;

use serde::Serialize;

use crate::calculate;
use crate::calculate::item::Item;
use crate::info::Info;

#[derive(Debug, Serialize)]
pub struct OutputCapacitor {
    pub stable: bool,
    pub depletes_in: f64,
    pub capacity: f64,
    pub recharge: f64,
    pub peak: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputOffense {
    pub dps: f64,
    pub dps_with_reload: f64,
    pub alpha: f64,
    pub drone_dps: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefenseRecharge {
    pub passive: f64,
    pub shield: f64,
    pub armor: f64,
    pub hull: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefenseResist {
    pub em: f64,
    pub therm: f64,
    pub kin: f64,
    pub expl: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefenseShield {
    pub resist: OutputDefenseResist,
    pub hp: f64,
    pub recharge: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefenseArmor {
    pub resist: OutputDefenseResist,
    pub hp: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefenseStructure {
    pub resist: OutputDefenseResist,
    pub hp: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDefense {
    pub recharge: OutputDefenseRecharge,
    pub shield: OutputDefenseShield,
    pub armor: OutputDefenseArmor,
    pub structure: OutputDefenseStructure,
    pub ehp: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputTargeting {
    pub lock_range: f64,
    pub sensor_strength: f64,
    pub scan_resolution: f64,
    pub signature_radius: f64,
    pub max_locked_targets: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputNavigation {
    pub speed: f64,
    pub mass: f64,
    pub agility: f64,
    pub warp_speed: f64,
    pub align_time: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputDrones {
    pub dps: f64,
    pub bandwidth_load: f64,
    pub bandwidth: f64,
    pub range: f64,
    pub active: f64,
    pub capacity_load: f64,
    pub capacity: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputCpu {
    pub free: f64,
    pub capacity: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputPower {
    pub free: f64,
    pub capacity: f64,
}

#[derive(Debug, Serialize)]
pub struct OutputSlots {
    pub hi_1: String,
    pub hi_2: String,
    pub hi_3: String,
    pub hi_4: String,
    pub hi_5: String,
    pub hi_6: String,
    pub hi_7: String,
    pub hi_8: String,
    pub med_1: String,
    pub med_2: String,
    pub med_3: String,
    pub med_4: String,
    pub med_5: String,
    pub med_6: String,
    pub med_7: String,
    pub med_8: String,
    pub lo_1: String,
    pub lo_2: String,
    pub lo_3: String,
    pub lo_4: String,
    pub lo_5: String,
    pub lo_6: String,
    pub lo_7: String,
    pub lo_8: String,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub capacitor: OutputCapacitor,
    pub offense: OutputOffense,
    pub defense: OutputDefense,
    pub targeting: OutputTargeting,
    pub navigation: OutputNavigation,
    pub drones: OutputDrones,
    pub cpu: OutputCpu,
    pub power: OutputPower,
    pub slots: OutputSlots,
}

fn get_attribute_by_name(
    info: &impl Info,
    attributes: &BTreeMap<i32, calculate::item::Attribute>,
    name: &str,
) -> f64 {
    let attribute_id = info.attribute_name_to_id(name);
    let default_value = info
        .get_dogma_attribute(attribute_id)
        .map_or(0.0, |attribute| attribute.default_value() as f64);

    let attribute = attributes
        .iter()
        .find(|attribute| *attribute.0 == attribute_id);
    if let Some(attribute) = attribute {
        attribute.1.value.get().unwrap_or(default_value)
    } else {
        default_value
    }
}

fn effect_category_to_name(
    items: &[Item],
    slot_type: calculate::item::SlotType,
    index: i32,
) -> String {
    let item = items
        .iter()
        .find(|item| item.slot.index == Some(index) && item.slot.r#type == slot_type);

    if let Some(item) = item {
        match item.state {
            calculate::item::EffectCategory::Passive => "passive",
            calculate::item::EffectCategory::Online => "online",
            calculate::item::EffectCategory::Active => "active",
            calculate::item::EffectCategory::Overload => "overload",
            _ => "unknown",
        }
    } else {
        "empty"
    }
    .to_string()
}

impl Output {
    pub fn new(info: &impl Info, statistics: &calculate::Ship) -> Output {
        Output {
            capacitor: OutputCapacitor {
                stable: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "capacitorDepletesIn",
                ) == -1.0,
                depletes_in: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "capacitorDepletesIn",
                ),
                capacity: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "capacitorCapacity",
                )
                .floor(),
                recharge: get_attribute_by_name(info, &statistics.hull.attributes, "rechargeRate")
                    / 1000.0,
                peak: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "capacitorPeakDelta",
                ),
                percentage: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "capacitorPeakDeltaPercentage",
                ),
            },
            offense: OutputOffense {
                dps: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "damagePerSecondWithoutReload",
                ),
                dps_with_reload: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "damagePerSecondWithReload",
                ),
                alpha: get_attribute_by_name(info, &statistics.hull.attributes, "damageAlpha"),
                drone_dps: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "droneDamagePerSecond",
                ),
            },
            defense: OutputDefense {
                recharge: OutputDefenseRecharge {
                    passive: get_attribute_by_name(
                        info,
                        &statistics.hull.attributes,
                        "passiveShieldRechargeRate",
                    ),
                    shield: get_attribute_by_name(
                        info,
                        &statistics.hull.attributes,
                        "shieldBoostRate",
                    ),
                    armor: get_attribute_by_name(
                        info,
                        &statistics.hull.attributes,
                        "armorRepairRate",
                    ),
                    hull: get_attribute_by_name(
                        info,
                        &statistics.hull.attributes,
                        "hullRepairRate",
                    ),
                },
                shield: OutputDefenseShield {
                    resist: OutputDefenseResist {
                        em: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "shieldEmDamageResonance",
                            ))
                            * 100.0,
                        therm: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "shieldThermalDamageResonance",
                            ))
                            * 100.0,
                        kin: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "shieldKineticDamageResonance",
                            ))
                            * 100.0,
                        expl: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "shieldExplosiveDamageResonance",
                            ))
                            * 100.0,
                    },
                    hp: get_attribute_by_name(info, &statistics.hull.attributes, "shieldCapacity"),
                    recharge: get_attribute_by_name(
                        info,
                        &statistics.hull.attributes,
                        "shieldRechargeRate",
                    ) / 1000.0,
                },
                armor: OutputDefenseArmor {
                    resist: OutputDefenseResist {
                        em: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "armorEmDamageResonance",
                            ))
                            * 100.0,
                        therm: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "armorThermalDamageResonance",
                            ))
                            * 100.0,
                        kin: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "armorKineticDamageResonance",
                            ))
                            * 100.0,
                        expl: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "armorExplosiveDamageResonance",
                            ))
                            * 100.0,
                    },
                    hp: get_attribute_by_name(info, &statistics.hull.attributes, "armorHP"),
                },
                structure: OutputDefenseStructure {
                    resist: OutputDefenseResist {
                        em: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "emDamageResonance",
                            ))
                            * 100.0,
                        therm: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "thermalDamageResonance",
                            ))
                            * 100.0,
                        kin: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "kineticDamageResonance",
                            ))
                            * 100.0,
                        expl: (1.0
                            - get_attribute_by_name(
                                info,
                                &statistics.hull.attributes,
                                "explosiveDamageResonance",
                            ))
                            * 100.0,
                    },
                    hp: get_attribute_by_name(info, &statistics.hull.attributes, "hp"),
                },
                ehp: get_attribute_by_name(info, &statistics.hull.attributes, "ehp"),
            },
            targeting: OutputTargeting {
                lock_range: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "maxTargetRange",
                ) / 1000.0,
                sensor_strength: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "scanStrength",
                ),
                scan_resolution: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "scanResolution",
                ),
                signature_radius: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "signatureRadius",
                ),
                max_locked_targets: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "maxLockedTargets",
                ),
            },
            navigation: OutputNavigation {
                speed: get_attribute_by_name(info, &statistics.hull.attributes, "maxVelocity"),
                mass: get_attribute_by_name(info, &statistics.hull.attributes, "mass") / 1000.0,
                agility: get_attribute_by_name(info, &statistics.hull.attributes, "agility"),
                warp_speed: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "warpSpeedMultiplier",
                ),
                align_time: get_attribute_by_name(info, &statistics.hull.attributes, "alignTime"),
            },
            drones: OutputDrones {
                dps: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "droneDamagePerSecond",
                ),
                bandwidth_load: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "droneBandwidthLoad",
                ),
                bandwidth: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "droneBandwidth",
                ),
                capacity_load: get_attribute_by_name(
                    info,
                    &statistics.hull.attributes,
                    "droneCapacityLoad",
                ),
                capacity: get_attribute_by_name(info, &statistics.hull.attributes, "droneCapacity"),
                range: get_attribute_by_name(
                    info,
                    &statistics.char.attributes,
                    "droneControlDistance",
                ) / 1000.0,
                active: get_attribute_by_name(info, &statistics.hull.attributes, "droneActive"),
            },
            cpu: OutputCpu {
                free: get_attribute_by_name(info, &statistics.hull.attributes, "cpuFree"),
                capacity: get_attribute_by_name(info, &statistics.hull.attributes, "cpuOutput"),
            },
            power: OutputPower {
                free: get_attribute_by_name(info, &statistics.hull.attributes, "powerFree"),
                capacity: get_attribute_by_name(info, &statistics.hull.attributes, "powerOutput"),
            },
            slots: OutputSlots {
                hi_1: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    0,
                ),
                hi_2: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    1,
                ),
                hi_3: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    2,
                ),
                hi_4: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    3,
                ),
                hi_5: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    4,
                ),
                hi_6: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    5,
                ),
                hi_7: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    6,
                ),
                hi_8: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::High,
                    7,
                ),
                med_1: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    0,
                ),
                med_2: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    1,
                ),
                med_3: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    2,
                ),
                med_4: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    3,
                ),
                med_5: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    4,
                ),
                med_6: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    5,
                ),
                med_7: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    6,
                ),
                med_8: effect_category_to_name(
                    &statistics.items,
                    calculate::item::SlotType::Medium,
                    7,
                ),
                lo_1: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 0),
                lo_2: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 1),
                lo_3: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 2),
                lo_4: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 3),
                lo_5: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 4),
                lo_6: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 5),
                lo_7: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 6),
                lo_8: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 7),
            },
        }
    }
}
